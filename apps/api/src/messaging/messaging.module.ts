import { Module } from '@nestjs/common';
import { RabbitMQConnectionProvider } from './publishers/rabbitmq-connection.provider';
import { RabbitEventPublisher } from './publishers/rabbit-event.publisher';
import { TelemetryPublisher } from './publishers/telemetry.publisher';
import { DLQService } from './providers/dlq.service';
import { DLQController } from './controllers/dlq.controller';

const EVENT_PUBLISHER = 'IEventPublisher';

@Module({
  controllers: [DLQController],
  providers: [
    RabbitMQConnectionProvider,
    {
      provide: EVENT_PUBLISHER,
      useClass: RabbitEventPublisher,
    },
    TelemetryPublisher,
    DLQService,
  ],
  exports: [TelemetryPublisher, DLQService],
})
export class MessagingModule {}
