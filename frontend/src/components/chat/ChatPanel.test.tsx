import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import {
  createDevBypassAuthValue,
  DevBypassAuthProvider,
} from "../../auth/DevBypassAuthProvider";
import { ChatPanel } from "./ChatPanel";

vi.stubGlobal("crypto", {
  randomUUID: () => "00000000-0000-4000-8000-000000000001",
});

describe("DevBypassAuthProvider", () => {
  it("createDevBypassAuthValue satisfies dev-bypass contract", () => {
    const auth = createDevBypassAuthValue();

    expect(auth.isLoading).toBe(false);
    expect(auth.isAuthenticated).toBe(false);
    expect(auth.user).toBeUndefined();
    expect(auth.settings.authority).toBe("");
  });
});

describe("ChatPanel with DevBypassAuthProvider", () => {
  it("renders without useAuth context warning", async () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

    vi.stubGlobal(
      "fetch",
      vi.fn(() =>
        Promise.resolve({
          ok: true,
          status: 200,
          json: () =>
            Promise.resolve({
              provider_configured: false,
              privacy: { allow_raw_transactions: false },
            }),
        }),
      ),
    );

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <DevBypassAuthProvider>
          <ChatPanel />
        </DevBypassAuthProvider>
      </QueryClientProvider>,
    );

    expect(
      warnSpy.mock.calls.some(([message]) =>
        String(message).includes("AuthProvider context is undefined"),
      ),
    ).toBe(false);

    expect(await screen.findByPlaceholderText(/Ask about your finances/i)).toBeInTheDocument();

    warnSpy.mockRestore();
  });
});
