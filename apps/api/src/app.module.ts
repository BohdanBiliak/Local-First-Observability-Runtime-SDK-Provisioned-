import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { HealthModule } from './health/health.module';
import { MessagingModule } from './messaging/messaging.module';
import { MetricsModule } from './metrics/metrics.module';
import { TestModule } from './test/test.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: '.env',
    }),
    HealthModule,
    MessagingModule,
    MetricsModule,
    TestModule,
  ],
})
export class AppModule {}
