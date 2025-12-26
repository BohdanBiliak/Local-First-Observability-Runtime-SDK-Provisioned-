import { Controller, Post, Body } from '@nestjs/common';
import { RabbitEventPublisher } from '../messaging/publishers/rabbit-event.publisher';

@Controller('test')
export class TestController {
  constructor(private readonly eventPublisher: RabbitEventPublisher) {}

  @Post('publish')
  async publishEvent(@Body() body: { exchange: string; routingKey: string; event: any }) {
    await this.eventPublisher.publish(body.exchange, body.routingKey, body.event);
    return {
      status: 'published',
      exchange: body.exchange,
      routingKey: body.routingKey,
      timestamp: new Date().toISOString(),
    };
  }
}
