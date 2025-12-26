import { Injectable, OnModuleInit, OnModuleDestroy, Logger } from '@nestjs/common';
import { IEventPublisher } from '../interfaces/event-publisher.interface';
import { RabbitMQConnectionProvider } from './rabbitmq-connection.provider';
import * as amqp from 'amqplib';

@Injectable()
export class RabbitEventPublisher implements IEventPublisher, OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(RabbitEventPublisher.name);
  private channel: amqp.ConfirmChannel | null = null;

  constructor(private readonly connectionProvider: RabbitMQConnectionProvider) {}

  async onModuleInit() {
    setTimeout(async () => {
      try {
        const connection = this.connectionProvider.getConnection();
        this.channel = await connection.createConfirmChannel();
        this.logger.log('RabbitMQ confirm channel created');
      } catch (error) {
        this.logger.error('Failed to create channel:', error);
      }
    }, 100);
  }

  async onModuleDestroy() {
    await this.close();
  }

  async publish(exchange: string, routingKey: string, event: unknown): Promise<void> {
    if (!this.channel) {
      throw new Error('Channel not initialized');
    }

    try {
      await this.channel.assertExchange(exchange, 'topic', { durable: true });
      const buffer = Buffer.from(JSON.stringify(event));

      return new Promise((resolve, reject) => {
        this.channel.publish(exchange, routingKey, buffer, { persistent: true }, (err) => {
          if (err) {
            this.logger.error(`Failed to publish to ${exchange}:${routingKey}`, err);
            reject(err);
          } else {
            resolve();
          }
        });
      });
    } catch (error) {
      this.logger.error('Publish error:', error);
      throw error;
    }
  }

  async close(): Promise<void> {
    if (this.channel) {
      await this.channel.close();
      this.logger.log('RabbitMQ channel closed');
      this.channel = null;
    }
  }
}
