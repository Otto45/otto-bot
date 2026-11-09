export interface AuditEvent {
  readonly id: string;
  readonly customerId: string;
  readonly description: string;
  readonly occurredOn: Date;
}
