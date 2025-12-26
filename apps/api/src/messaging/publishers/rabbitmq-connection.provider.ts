import { Injectable, OnModuleDestroy, OnModuleInit, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as amqp from 'amqplib';

@Injectable()
export class RabbitMQConnectionProvider implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(RabbitMQConnectionProvider.name);
  private connection: amqp.ChannelModel | null = null;

  constructor(private readonly configService: ConfigService) {}

  async onModuleInit() {
    const url = this.configService.get<string>('RABBITMQ_URL');
    if (!url) {
      throw new Error('RABBITMQ_URL is not configured');
    }

    try {
      this.connection = await amqp.connect(url);
      this.logger.log('RabbitMQ connection established');

      this.connection.on('error', (err) => {
        this.logger.error('RabbitMQ connection error:', err);
      });

      this.connection.on('close', () => {
        this.logger.warn('RabbitMQ connection closed');
      });
    } catch (error) {
      this.logger.error('Failed to connect to RabbitMQ:', error);
      throw error;
    }
  }

  async onModuleDestroy() {
    if (this.connection) {
      try {
        await this.connection.close();
        this.logger.log('RabbitMQ connection closed gracefully');
      } catch (error) {
        this.logger.error('Error closing connection:', error);
      }
    }
  }

  getConnection(): amqp.ChannelModel {
    if (!this.connection) {
      throw new Error('RabbitMQ connection not established');
    }
    return this.connection;
  }
}
