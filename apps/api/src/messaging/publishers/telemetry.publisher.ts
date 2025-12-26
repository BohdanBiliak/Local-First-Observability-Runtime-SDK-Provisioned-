import { Injectable, Logger, Inject } from '@nestjs/common';
import {
  EventEnvelope,
  EventValidator,
  LogCapturedValidatorV1,
  LogCapturedPayloadV1,
} from '@observability/contracts';
import { ITelemetryPublisher, PublishResult } from '../interfaces/telemetry-publisher.interface';
import { IEventPublisher } from '../interfaces/event-publisher.interface';

const EVENT_PUBLISHER = 'IEventPublisher';

@Injectable()
export class TelemetryPublisher implements ITelemetryPublisher {
  private readonly logger = new Logger(TelemetryPublisher.name);
  private readonly TELEMETRY_EXCHANGE = 'telemetry.events';

  private readonly validators = new Map<string, EventValidator<any>>([
    ['telemetry.log.captured:1', new LogCapturedValidatorV1()],
  ]);

  constructor(@Inject(EVENT_PUBLISHER) private readonly eventPublisher: IEventPublisher) {}

  async publishTelemetryEvent<T>(event: EventEnvelope<T>): Promise<void> {
    const validationKey = `${event.eventType}:${event.eventVersion}`;
    const validator = this.validators.get(validationKey);

    if (!validator) {
      const error = `No validator found for ${validationKey}`;
      this.logger.error(error);
      throw new Error(error);
    }

    const validationResult = validator.validate(event);

    if (!validationResult.valid) {
      this.logger.warn(
        `Invalid event ${event.eventId}:`,
        JSON.stringify(validationResult.errors, null, 2),
      );
      throw new Error(
        `Event validation failed: ${validationResult.errors.map((e) => `${e.field}: ${e.message}`).join(', ')}`,
      );
    }

    const routingKey = this.buildRoutingKey(event);

    try {
      await this.eventPublisher.publish(this.TELEMETRY_EXCHANGE, routingKey, event);
      this.logger.log(`Published ${event.eventType} v${event.eventVersion} [${event.eventId}]`);
    } catch (error) {
      this.logger.error(`Failed to publish event ${event.eventId}:`, error);
      throw error;
    }
  }

  private buildRoutingKey(event: EventEnvelope<any>): string {
    return `${event.eventType}.v${event.eventVersion}`;
  }
}
