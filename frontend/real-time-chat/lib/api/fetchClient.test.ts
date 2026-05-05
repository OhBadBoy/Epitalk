import { beforeEach, describe, expect, it, vi } from "vitest";

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

async function createClient() {
  const { FetchClient } = await import("./fetchClient");
  return new FetchClient("http://localhost:3001/api");
}

describe("FetchClient auth refresh flow", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
    vi.resetModules();
  });

  it("refreshes token on 401 and retries the original request", async () => {
    localStorage.setItem("token", "old-token");
    localStorage.setItem(
      "auth-storage",
      JSON.stringify({
        state: {
          token: "old-token",
          isAuthenticated: true,
          user: { id: "u1", username: "old", email: "old@example.com" },
        },
      }),
    );

    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(jsonResponse(401, { error: "Unauthorized" }))
      .mockResolvedValueOnce(
        jsonResponse(200, {
          token: "new-token",
          user: { id: "u1", username: "new", email: "new@example.com" },
        }),
      )
      .mockResolvedValueOnce(jsonResponse(200, { ok: true }));

    vi.stubGlobal("fetch", fetchMock);

    const client = await createClient();
    const data = await client.get<{ ok: boolean }>("/servers");

    expect(data.ok).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(3);

    const refreshCall = fetchMock.mock.calls[1];
    expect(String(refreshCall[0])).toContain("/auth/refresh");

    const retriedCall = fetchMock.mock.calls[2];
    const retriedHeaders = (retriedCall[1]?.headers ?? {}) as Record<string, string>;
    expect(retriedHeaders.Authorization).toBe("Bearer new-token");

    expect(localStorage.getItem("token")).toBe("new-token");
    const persisted = JSON.parse(localStorage.getItem("auth-storage") || "{}") as {
      state?: { token?: string; isAuthenticated?: boolean };
    };
    expect(persisted.state?.token).toBe("new-token");
    expect(persisted.state?.isAuthenticated).toBe(true);
  });

  it("clears session when refresh fails", async () => {
    localStorage.setItem("token", "expired-token");
    localStorage.setItem(
      "auth-storage",
      JSON.stringify({
        state: { token: "expired-token", isAuthenticated: true },
      }),
    );

    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(jsonResponse(401, { error: "Unauthorized" }))
      .mockResolvedValueOnce(jsonResponse(401, { error: "Token has expired" }));

    vi.stubGlobal("fetch", fetchMock);

    const client = await createClient();
    await expect(client.get("/servers")).rejects.toMatchObject({ status: 401 });

    expect(localStorage.getItem("token")).toBeNull();
    expect(localStorage.getItem("auth-storage")).toBeNull();
  });

  it("does not trigger refresh for non-refreshable auth endpoints", async () => {
    localStorage.setItem("token", "some-token");

    const fetchMock = vi.fn().mockResolvedValueOnce(
      jsonResponse(401, {
        error: "Invalid email or password",
      }),
    );

    vi.stubGlobal("fetch", fetchMock);

    const client = await createClient();
    await expect(
      client.post("/auth/login", { email: "user@example.com", password: "wrong" }),
    ).rejects.toMatchObject({ status: 401 });

    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(String(fetchMock.mock.calls[0][0])).toContain("/auth/login");
    expect(fetchMock.mock.calls.some((call) => String(call[0]).includes("/auth/refresh"))).toBe(
      false,
    );
  });
});
