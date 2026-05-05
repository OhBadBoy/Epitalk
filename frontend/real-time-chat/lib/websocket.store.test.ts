import { beforeEach, describe, expect, it, vi } from "vitest";

const hoisted = vi.hoisted(() => {
  class MockApiError extends Error {
    status: number;

    constructor(message: string, status: number) {
      super(message);
      this.status = status;
    }
  }

  return {
    meMock: vi.fn(),
    logoutMock: vi.fn(),
    MockApiError,
  };
});

vi.mock("@/lib/api", () => ({
  authApi: {
    me: hoisted.meMock,
  },
}));

vi.mock("@/lib/api/errors", () => ({
  ApiError: hoisted.MockApiError,
}));

vi.mock("@/store/auth.store", () => ({
  useAuthStore: {
    getState: () => ({
      token: "token-from-store",
      user: { id: "u1", username: "alice", email: "alice@example.test" },
      logout: hoisted.logoutMock,
    }),
  },
}));

vi.mock("@/store/dm.store", () => ({
  useDmStore: {
    getState: () => ({}),
  },
}));

class FakeWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  static instances: FakeWebSocket[] = [];

  url: string;
  readyState = FakeWebSocket.CONNECTING;
  onopen: ((ev: Event) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  send = vi.fn();

  constructor(url: string) {
    this.url = url;
    FakeWebSocket.instances.push(this);
  }

  close(code = 1000, reason = "") {
    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.({ code, reason } as CloseEvent);
  }
}

async function flushAsync() {
  await Promise.resolve();
  await Promise.resolve();
}

async function loadStore() {
  const mod = await import("../store/websocket.store");
  return mod.useWebSocketStore;
}

describe("websocket.store reconnect behavior", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    vi.resetModules();
    FakeWebSocket.instances = [];
    hoisted.meMock.mockResolvedValue({ id: "u1" });
    localStorage.clear();
    localStorage.setItem("token", "token-from-local-storage");
    vi.stubGlobal("WebSocket", FakeWebSocket as unknown as typeof WebSocket);
  });

  it("enters backoff with exponential delay after abnormal close", async () => {
    const useWebSocketStore = await loadStore();

    useWebSocketStore.getState().connect("token");
    const ws = FakeWebSocket.instances.at(-1);
    expect(ws).toBeDefined();

    ws!.onclose?.({ code: 1006, reason: "network drop" } as CloseEvent);
    await flushAsync();

    const state = useWebSocketStore.getState();
    expect(state.connectionState).toBe("backoff");
    expect(state.reconnectAttempt).toBe(1);
    expect(state.nextRetryDelayMs).toBe(1000);
    expect(hoisted.logoutMock).not.toHaveBeenCalled();
  });

  it("switches to degraded mode after too many reconnect attempts", async () => {
    const useWebSocketStore = await loadStore();

    for (let i = 0; i < 6; i += 1) {
      useWebSocketStore.getState().connect("token");
      const ws = FakeWebSocket.instances.at(-1);
      expect(ws).toBeDefined();
      ws!.onclose?.({ code: 1006, reason: "network drop" } as CloseEvent);
      await flushAsync();
      vi.clearAllTimers();
    }

    const state = useWebSocketStore.getState();
    expect(state.connectionState).toBe("degraded");
    expect(state.reconnectAttempt).toBe(6);
    expect(state.nextRetryDelayMs).toBe(30000);
  });

  it("forces auth_invalid and logout on policy close", async () => {
    const useWebSocketStore = await loadStore();

    useWebSocketStore.getState().connect("token");
    const ws = FakeWebSocket.instances.at(-1);
    expect(ws).toBeDefined();

    ws!.onclose?.({ code: 1008, reason: "Unauthorized" } as CloseEvent);
    await flushAsync();

    const state = useWebSocketStore.getState();
    expect(state.connectionState).toBe("auth_invalid");
    expect(state.reconnectAttempt).toBe(0);
    expect(state.nextRetryDelayMs).toBeNull();
    expect(state.error).toContain("Session expirée");
    expect(hoisted.logoutMock).toHaveBeenCalledTimes(1);
  });

  it("sends MessageSchedule payload when scheduling a message", async () => {
    const useWebSocketStore = await loadStore();

    useWebSocketStore.getState().connect("token");
    const ws = FakeWebSocket.instances.at(-1);
    expect(ws).toBeDefined();

    ws!.readyState = FakeWebSocket.OPEN;
    ws!.onopen?.({} as Event);

    useWebSocketStore
      .getState()
      .sendScheduledMessage("channel-1", "hello later", 5, 14, 30);

    const scheduleCall = ws!.send.mock.calls
      .map((call) => JSON.parse(call[0] as string))
      .find((payload) => payload.type === "MessageSchedule");

    expect(scheduleCall).toBeDefined();
    const payload = scheduleCall as {
      type: string;
      payload: {
        channel_id: string;
        day_of_week: number;
        hour: number;
        minute: number;
      };
    };
    expect(payload.type).toBe("MessageSchedule");
    expect(payload.payload.channel_id).toBe("channel-1");
    expect(payload.payload.day_of_week).toBe(5);
    expect(payload.payload.hour).toBe(14);
    expect(payload.payload.minute).toBe(30);
  });
});
