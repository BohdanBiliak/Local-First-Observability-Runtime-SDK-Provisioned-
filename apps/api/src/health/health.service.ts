import { Injectable, OnModuleDestroy } from '@nestjs/common';

@Injectable()
export class HealthService implements OnModuleDestroy {
  private isShuttingDown = false;

  check() {
    return {
      status: this.isShuttingDown ? 'shutting_down' : 'ok',
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
    };
  }

  onModuleDestroy() {
    this.isShuttingDown = true;
    console.log('Health service: graceful shutdown initiated');
  }
}
