export interface IEventPublisher {
  publish(exchange: string, routingKey: string, event: unknown): Promise<void>;
  close(): Promise<void>;
}
