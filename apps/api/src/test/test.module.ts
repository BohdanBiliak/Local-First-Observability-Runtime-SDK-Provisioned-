import { Module } from '@nestjs/common';
import { TestController } from './test.controller';
import { MessagingModule } from '../messaging/messaging.module';

@Module({
  imports: [MessagingModule],
  controllers: [TestController],
})
export class TestModule {}
