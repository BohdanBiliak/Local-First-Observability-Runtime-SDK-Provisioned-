import { EventEnvelope, ValidationResult } from '@observability/contracts';

export interface ITelemetryPublisher {
  /**
   * Publishes a validated telemetry event.
   * @throws Error if event is invalid or publishing fails
   */
  publishTelemetryEvent<T>(event: EventEnvelope<T>): Promise<void>;
}

export interface PublishResult {
  success: boolean;
  eventId: string;
  validation?: ValidationResult;
  error?: string;
}
