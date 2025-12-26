import { EventEnvelope, EventValidator, ValidationResult, ValidationError } from '../base';

export enum Environment {
  Development = 'development',
  Staging = 'staging',
  Production = 'production',
}

export interface LogCapturedPayloadV1 {
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  serviceName: string;
  environment: Environment;
  labels?: Record<string, string>;
  context?: Record<string, unknown>;
}

export type LogCapturedEventV1 = EventEnvelope<LogCapturedPayloadV1>;

export function createLogCapturedEventV1(
  payload: LogCapturedPayloadV1,
  correlationId?: string,
): LogCapturedEventV1 {
  return {
    eventId: crypto.randomUUID(),
    eventType: 'telemetry.log.captured',
    eventVersion: 1,
    timestamp: new Date().toISOString(),
    correlationId,
    payload,
  };
}

export class LogCapturedValidatorV1 implements EventValidator<LogCapturedPayloadV1> {
  validate(event: LogCapturedEventV1): ValidationResult {
    const errors: ValidationError[] = [];

    if (!event.eventId?.trim()) {
      errors.push({ field: 'eventId', message: 'eventId is required' });
    }

    if (event.eventType !== 'telemetry.log.captured') {
      errors.push({ field: 'eventType', message: 'must be "telemetry.log.captured"' });
    }

    if (event.eventVersion !== 1) {
      errors.push({ field: 'eventVersion', message: 'must be 1' });
    }

    if (!event.timestamp || isNaN(Date.parse(event.timestamp))) {
      errors.push({ field: 'timestamp', message: 'must be valid ISO 8601 date' });
    }

    const { payload } = event;

    if (!payload) {
      errors.push({ field: 'payload', message: 'payload is required' });
      return { valid: false, errors };
    }

    const validLevels = ['debug', 'info', 'warn', 'error'];
    if (!validLevels.includes(payload.level)) {
      errors.push({
        field: 'payload.level',
        message: `must be one of: ${validLevels.join(', ')}`,
      });
    }

    if (!payload.message?.trim()) {
      errors.push({ field: 'payload.message', message: 'message is required' });
    }

    if (!payload.serviceName?.trim()) {
      errors.push({ field: 'payload.serviceName', message: 'serviceName is required' });
    }

    const validEnvironments = Object.values(Environment);
    if (!validEnvironments.includes(payload.environment)) {
      errors.push({
        field: 'payload.environment',
        message: `must be one of: ${validEnvironments.join(', ')}`,
      });
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }
}
