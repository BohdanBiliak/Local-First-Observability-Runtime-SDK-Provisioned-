export interface DLQMessage {
  content: string;
  fields: {
    deliveryTag: number;
    redelivered: boolean;
    exchange: string;
    routingKey: string;
  };
  properties: {
    contentType?: string;
    contentEncoding?: string;
    headers?: Record<string, any>;
    deliveryMode?: number;
    priority?: number;
    correlationId?: string;
    replyTo?: string;
    expiration?: string;
    messageId?: string;
    timestamp?: number;
    type?: string;
    userId?: string;
    appId?: string;
  };
}

export interface ReplayConfig {
  maxReplayCount: number;
}

export class DLQMessageDto {
  id: string;
  content: any;
  originalQueue: string;
  routingKey: string;
  timestamp?: number;
  retryCount?: number;
  headers?: Record<string, any>;
  errorReason?: string;
}

export class DLQListResponseDto {
  queue: string;
  messageCount: number;
  messages: DLQMessageDto[];
}

export class ReplayMessageDto {
  messageId: string;
  targetQueue?: string;
  operator?: string;
}

export class ReplayResponseDto {
  success: boolean;
  message: string;
  replayed?: number;
}
