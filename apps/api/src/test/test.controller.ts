import { Controller, Post, Body, HttpCode } from '@nestjs/common';
import {
  createLogCapturedEventV1,
  Environment,
  LogCapturedPayloadV1,
} from '@observability/contracts';
import { TelemetryPublisher } from '../messaging/publishers/telemetry.publisher';

interface LogRequest {
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  serviceName: string;
  environment: 'development' | 'staging' | 'production';
  labels?: Record<string, string>;
}

@Controller('test')
export class TestController {
  constructor(private readonly telemetryPublisher: TelemetryPublisher) {}

  @Post('log')
  @HttpCode(200)
  async captureLog(@Body() body: LogRequest) {
    const payload: LogCapturedPayloadV1 = {
      level: body.level,
      message: body.message,
      serviceName: body.serviceName,
      environment:
        Environment[body.environment.charAt(0).toUpperCase() + body.environment.slice(1)],
      labels: body.labels,
    };

    const event = createLogCapturedEventV1(payload);

    await this.telemetryPublisher.publishTelemetryEvent(event);

    return {
      status: 'published',
      eventId: event.eventId,
      eventType: event.eventType,
      eventVersion: event.eventVersion,
      timestamp: event.timestamp,
    };
  }
}
