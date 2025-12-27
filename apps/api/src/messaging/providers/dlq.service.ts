import { Injectable, Logger, NotFoundException, BadRequestException } from '@nestjs/common';
import * as amqp from 'amqplib';
import { RabbitMQConnectionProvider } from '../publishers/rabbitmq-connection.provider';
import { MetricsService } from '../../metrics/metrics.service';
import {
  DLQMessageDto,
  DLQListResponseDto,
  ReplayResponseDto,
  DLQMessage,
  ReplayConfig,
} from '../interfaces/dlq.interface';

@Injectable()
export class DLQService {
  private readonly logger = new Logger(DLQService.name);
  private readonly replayConfig: ReplayConfig = {
    maxReplayCount: 1,
  };

  constructor(
    private readonly rabbitMQConnection: RabbitMQConnectionProvider,
    private readonly metricsService: MetricsService,
  ) {}

  async listDLQMessages(queueName: string, limit: number = 50): Promise<DLQListResponseDto> {
    const dlqName = `${queueName}.dlq`;
    const channel = await this.getChannel();

    try {
      const queueInfo = await channel.checkQueue(dlqName);
      const messages: DLQMessageDto[] = [];

      const fetchCount = Math.min(limit, queueInfo.messageCount);

      for (let i = 0; i < fetchCount; i++) {
        const msg = await channel.get(dlqName, { noAck: false });

        if (!msg) break;

        const messageDto = this.mapMessageToDto(msg, i);
        messages.push(messageDto);

        await channel.nack(msg, false, true);
      }

      return {
        queue: dlqName,
        messageCount: queueInfo.messageCount,
        messages,
      };
    } catch (error) {
      this.logger.error(`Failed to list DLQ messages: ${error.message}`);
      throw new NotFoundException(`DLQ ${dlqName} not found or inaccessible`);
    } finally {
      this.metricsService.incrementInspected(queueName, 'list');
    }
  }

  async inspectMessage(queueName: string, messageId: string): Promise<DLQMessageDto> {
    const dlqName = `${queueName}.dlq`;
    const channel = await this.getChannel();

    try {
      const queueInfo = await channel.checkQueue(dlqName);

      for (let i = 0; i < queueInfo.messageCount; i++) {
        const msg = await channel.get(dlqName, { noAck: false });

        if (!msg) break;

        const dto = this.mapMessageToDto(msg, i);

        if (dto.id === messageId) {
          await channel.nack(msg, false, true);
          this.metricsService.incrementInspected(queueName, 'inspect');
          return dto;
        }

        await channel.nack(msg, false, true);
      }

      throw new NotFoundException(`Message ${messageId} not found in DLQ`);
    } catch (error) {
      if (error instanceof NotFoundException) throw error;
      this.logger.error(`Failed to inspect message: ${error.message}`);
      throw new NotFoundException(`Failed to inspect message in ${dlqName}`);
    }
  }

  async replayMessage(
    queueName: string,
    messageId: string,
    targetQueue?: string,
    operator?: string,
  ): Promise<ReplayResponseDto> {
    if (!messageId) {
      throw new BadRequestException('messageId is required for replay');
    }

    const dlqName = `${queueName}.dlq`;
    const destination = targetQueue || queueName;
    const channel = await this.getChannel();

    try {
      const queueInfo = await channel.checkQueue(dlqName);

      for (let i = 0; i < queueInfo.messageCount; i++) {
        const msg = await channel.get(dlqName, { noAck: false });

        if (!msg) break;

        const dto = this.mapMessageToDto(msg, i);

        if (dto.id === messageId) {
          // Validate message has messageId
          if (!msg.properties.messageId) {
            await channel.nack(msg, false, true);
            throw new BadRequestException('Cannot replay message without messageId');
          }

          // Check replay count guard
          const replayCount = (msg.properties.headers?.['x-replay-count'] as number) || 0;
          if (replayCount >= this.replayConfig.maxReplayCount) {
            await channel.nack(msg, false, true);
            throw new BadRequestException(
              `Message has been replayed ${replayCount} times. Maximum replay count (${this.replayConfig.maxReplayCount}) exceeded.`,
            );
          }

          const properties = {
            ...msg.properties,
            headers: {
              ...(msg.properties.headers || {}),
              'x-replayed-from-dlq': true,
              'x-replay-timestamp': Date.now(),
              'x-replay-operator': operator || 'unknown',
              'x-replay-count': replayCount + 1,
            },
          };

          await channel.publish('', destination, msg.content, properties);
          await channel.ack(msg);

          this.metricsService.incrementReplayed(queueName, destination, operator || 'unknown', 1);

          this.logger.log(
            `Replayed message ${messageId} from ${dlqName} to ${destination} (operator: ${operator || 'unknown'}, replay count: ${replayCount + 1})`,
          );

          return {
            success: true,
            message: `Message replayed to ${destination}`,
            replayed: 1,
          };
        }

        await channel.nack(msg, false, true);
      }

      throw new NotFoundException(`Message ${messageId} not found in DLQ`);
    } catch (error) {
      if (error instanceof NotFoundException || error instanceof BadRequestException) {
        throw error;
      }
      this.logger.error(`Failed to replay message: ${error.message}`);
      return {
        success: false,
        message: `Failed to replay message: ${error.message}`,
      };
    }
  }

  async replayAllMessages(
    queueName: string,
    targetQueue?: string,
    operator?: string,
  ): Promise<ReplayResponseDto> {
    const dlqName = `${queueName}.dlq`;
    const destination = targetQueue || queueName;
    const channel = await this.getChannel();

    try {
      const queueInfo = await channel.checkQueue(dlqName);
      let replayedCount = 0;
      let skippedCount = 0;

      for (let i = 0; i < queueInfo.messageCount; i++) {
        const msg = await channel.get(dlqName, { noAck: false });

        if (!msg) break;

        // Skip messages without messageId
        if (!msg.properties.messageId) {
          this.logger.warn(`Skipping message without messageId at index ${i}`);
          await channel.nack(msg, false, true);
          skippedCount++;
          continue;
        }

        // Check replay count guard
        const replayCount = (msg.properties.headers?.['x-replay-count'] as number) || 0;
        if (replayCount >= this.replayConfig.maxReplayCount) {
          this.logger.warn(
            `Skipping message ${msg.properties.messageId}: replay count ${replayCount} exceeds maximum ${this.replayConfig.maxReplayCount}`,
          );
          await channel.nack(msg, false, true);
          skippedCount++;
          continue;
        }

        const properties = {
          ...msg.properties,
          headers: {
            ...(msg.properties.headers || {}),
            'x-replayed-from-dlq': true,
            'x-replay-timestamp': Date.now(),
            'x-replay-operator': operator || 'unknown',
            'x-replay-count': replayCount + 1,
          },
        };

        await channel.publish('', destination, msg.content, properties);
        await channel.ack(msg);
        replayedCount++;
      }

      if (replayedCount > 0) {
        this.metricsService.incrementReplayed(
          queueName,
          destination,
          operator || 'unknown',
          replayedCount,
        );
      }

      this.logger.log(
        `Bulk replay from ${dlqName} to ${destination}: ${replayedCount} replayed, ${skippedCount} skipped (operator: ${operator || 'unknown'})`,
      );

      return {
        success: true,
        message: `Replayed ${replayedCount} messages to ${destination}${skippedCount > 0 ? `, skipped ${skippedCount}` : ''}`,
        replayed: replayedCount,
      };
    } catch (error) {
      this.logger.error(`Failed to replay all messages: ${error.message}`);
      return {
        success: false,
        message: `Failed to replay messages: ${error.message}`,
      };
    }
  }

  async purgeQueue(
    queueName: string,
  ): Promise<{ success: boolean; message: string; purged: number }> {
    const dlqName = `${queueName}.dlq`;
    const channel = await this.getChannel();

    try {
      const result = await channel.purgeQueue(dlqName);

      this.metricsService.incrementPurged(queueName, result.messageCount);

      this.logger.log(`Purged ${result.messageCount} messages from ${dlqName}`);

      return {
        success: true,
        message: `Purged ${result.messageCount} messages from DLQ`,
        purged: result.messageCount,
      };
    } catch (error) {
      this.logger.error(`Failed to purge queue: ${error.message}`);
      return {
        success: false,
        message: `Failed to purge queue: ${error.message}`,
        purged: 0,
      };
    }
  }

  private async getChannel(): Promise<amqp.Channel> {
    const connection = this.rabbitMQConnection.getConnection();
    return await connection.createChannel();
  }

  private mapMessageToDto(msg: amqp.GetMessage, index: number): DLQMessageDto {
    let content: any;
    try {
      content = JSON.parse(msg.content.toString());
    } catch {
      content = msg.content.toString();
    }

    const headers = msg.properties.headers || {};
    const retryCount = headers['x-retry-count'] || 0;
    const errorReason = headers['x-error-reason'];
    const errorType = headers['x-error-type'] as 'transient' | 'permanent' | undefined;
    const eventVersion = headers['x-event-version'];

    return {
      id: msg.properties.messageId || `msg-${index}`,
      content,
      originalQueue: headers['x-original-queue'] || 'unknown',
      routingKey: msg.fields.routingKey,
      timestamp: msg.properties.timestamp,
      retryCount,
      headers,
      errorReason,
      errorType,
      eventVersion,
    };
  }
}
