import { Module } from '@nestjs/common';
import { RabbitMQConnectionProvider } from './publishers/rabbitmq-connection.provider';
import { RabbitEventPublisher } from './publishers/rabbit-event.publisher';
import { TelemetryPublisher } from './publishers/telemetry.publisher';

const EVENT_PUBLISHER = 'IEventPublisher';

@Module({
  providers: [
    RabbitMQConnectionProvider,
    {
      provide: EVENT_PUBLISHER,
      useClass: RabbitEventPublisher,
    },
    TelemetryPublisher,
  ],
  exports: [TelemetryPublisher],
})
export class MessagingModule {}
