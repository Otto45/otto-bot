export interface AuditEventResponse {
  readonly id: string;
  readonly description: string;
  /** When it happened, in ISO 8601. */
  readonly occurredOn: string;
}
