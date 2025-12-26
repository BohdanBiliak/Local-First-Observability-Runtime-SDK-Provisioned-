import {
  Controller,
  Get,
  Post,
  Delete,
  Param,
  Query,
  Body,
  HttpCode,
  HttpStatus,
} from '@nestjs/common';
import { DLQService } from '../providers/dlq.service';
import {
  DLQListResponseDto,
  DLQMessageDto,
  ReplayMessageDto,
  ReplayResponseDto,
} from '../interfaces/dlq.interface';

@Controller('dlq')
export class DLQController {
  constructor(private readonly dlqService: DLQService) {}

  @Get(':queue')
  async listMessages(
    @Param('queue') queue: string,
    @Query('limit') limit?: number,
  ): Promise<DLQListResponseDto> {
    return this.dlqService.listDLQMessages(queue, limit ? +limit : 50);
  }

  @Get(':queue/:messageId')
  async inspectMessage(
    @Param('queue') queue: string,
    @Param('messageId') messageId: string,
  ): Promise<DLQMessageDto> {
    return this.dlqService.inspectMessage(queue, messageId);
  }

  @Post(':queue/replay')
  @HttpCode(HttpStatus.OK)
  async replayMessages(
    @Param('queue') queue: string,
    @Body() body: { messageId?: string; targetQueue?: string; operator?: string },
  ): Promise<ReplayResponseDto> {
    if (body.messageId) {
      return this.dlqService.replayMessage(queue, body.messageId, body.targetQueue, body.operator);
    }
    return this.dlqService.replayAllMessages(queue, body.targetQueue, body.operator);
  }

  @Delete(':queue')
  @HttpCode(HttpStatus.OK)
  async purgeQueue(@Param('queue') queue: string): Promise<{
    success: boolean;
    message: string;
    purged: number;
  }> {
    return this.dlqService.purgeQueue(queue);
  }
}
