// Vitest setup file
import "@testing-library/jest-dom";

// Ensure a WebSocket global exists for components/tests that reference WebSocket
if (
  typeof (globalThis as unknown as { WebSocket?: unknown }).WebSocket ===
  "undefined"
) {
  // Minimal mock with OPEN constant and basic methods used by components/tests
  class MockWebSocket {
    static OPEN = 1;
    readyState = MockWebSocket.OPEN;
    onopen: ((ev: Event) => void) | null = null;
    onmessage: ((ev: { data: string }) => void) | null = null;
    onerror: ((ev: Event) => void) | null = null;
    onclose: ((ev: { code: number; reason: string }) => void) | null = null;
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    constructor(..._args: unknown[]) {}
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    send(_data: unknown) {
      /* no-op for tests */
    }
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    close(_code?: number, _reason?: string) {
      /* no-op for tests */
    }
  }

  // Assign mock to global (use a non-strict cast for assignment)
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  globalThis.WebSocket = MockWebSocket;
}
