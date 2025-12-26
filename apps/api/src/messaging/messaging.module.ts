import { Module } from '@nestjs/common';
import { RabbitMQConnectionProvider } from './publishers/rabbitmq-connection.provider';
import { RabbitEventPublisher } from './publishers/rabbit-event.publisher';

@Module({
  providers: [RabbitMQConnectionProvider, RabbitEventPublisher],
  exports: [RabbitEventPublisher],
})
export class MessagingModule {}
