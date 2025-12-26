export interface BaseEvent {
  eventId: string;
  eventType: string;
  eventVersion: number;
  timestamp: string;
  correlationId?: string;
}

export interface EventEnvelope<T> extends BaseEvent {
  payload: T;
}

export interface ValidationError {
  field: string;
  message: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface EventValidator<T> {
  validate(event: EventEnvelope<T>): ValidationResult;
}
