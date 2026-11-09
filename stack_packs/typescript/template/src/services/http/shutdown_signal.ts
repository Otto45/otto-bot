// Ctrl-C, or the SIGTERM a deployment stops the service with: the server stops taking requests, finishes the ones
// under way, and the facade is closed.
export class ShutdownSignal {
  public static received(): Promise<NodeJS.Signals> {
    return new Promise<NodeJS.Signals>((resolve: (signal: NodeJS.Signals) => void): void => {
      process.once("SIGINT", resolve);
      process.once("SIGTERM", resolve);
    });
  }
}
