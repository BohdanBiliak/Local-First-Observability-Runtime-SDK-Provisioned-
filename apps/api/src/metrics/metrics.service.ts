import { Injectable, OnModuleInit } from '@nestjs/common';
import * as promClient from 'prom-client';

@Injectable()
export class MetricsService implements OnModuleInit {
  private readonly registry: promClient.Registry;

  // DLQ operation counters
  public readonly dlqMessagesInspected: promClient.Counter<string>;
  public readonly dlqMessagesReplayed: promClient.Counter<string>;
  public readonly dlqMessagesPurged: promClient.Counter<string>;

  constructor() {
    this.registry = new promClient.Registry();

    // Enable default metrics (CPU, memory, etc.)
    promClient.collectDefaultMetrics({ register: this.registry });

    // DLQ operation metrics
    this.dlqMessagesInspected = new promClient.Counter({
      name: 'dlq_messages_inspected_total',
      help: 'Total number of DLQ messages inspected',
      labelNames: ['queue', 'operation'],
      registers: [this.registry],
    });

    this.dlqMessagesReplayed = new promClient.Counter({
      name: 'dlq_messages_replayed_total',
      help: 'Total number of DLQ messages replayed',
      labelNames: ['queue', 'target_queue', 'operator'],
      registers: [this.registry],
    });

    this.dlqMessagesPurged = new promClient.Counter({
      name: 'dlq_messages_purged_total',
      help: 'Total number of DLQ messages purged',
      labelNames: ['queue'],
      registers: [this.registry],
    });
  }

  onModuleInit() {
    // Module initialized
  }

  async getMetrics(): Promise<string> {
    return this.registry.metrics();
  }

  incrementInspected(queue: string, operation: 'list' | 'inspect'): void {
    this.dlqMessagesInspected.inc({ queue, operation });
  }

  incrementReplayed(queue: string, targetQueue: string, operator: string, count: number = 1): void {
    this.dlqMessagesReplayed.inc({ queue, target_queue: targetQueue, operator }, count);
  }

  incrementPurged(queue: string, count: number): void {
    this.dlqMessagesPurged.inc({ queue }, count);
  }
}
