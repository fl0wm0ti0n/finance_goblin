import { useCallback, useRef, useState } from "react";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  toolsUsed?: { name: string; at: string }[];
  streaming?: boolean;
}

interface StreamHandlers {
  onToken?: (delta: string) => void;
  onToolStart?: (tool: string) => void;
  onToolEnd?: (tool: string, status: string) => void;
  onDone?: (toolsUsed: string[]) => void;
  onWarning?: (message: string) => void;
  onError?: (code: string, message: string) => void;
}

function parseSseChunk(buffer: string, handlers: StreamHandlers): string {
  const parts = buffer.split("\n\n");
  const remainder = parts.pop() ?? "";

  for (const block of parts) {
    let event = "message";
    let data = "";
    for (const line of block.split("\n")) {
      if (line.startsWith("event:")) event = line.slice(6).trim();
      if (line.startsWith("data:")) data = line.slice(5).trim();
    }
    if (!data) continue;
    try {
      const payload = JSON.parse(data);
      switch (event) {
        case "token":
          handlers.onToken?.(payload.delta ?? "");
          break;
        case "tool_start":
          handlers.onToolStart?.(payload.tool ?? "");
          break;
        case "tool_end":
          handlers.onToolEnd?.(payload.tool ?? "", payload.status ?? "ok");
          break;
        case "done":
          handlers.onDone?.(payload.tools_used ?? []);
          break;
        case "warning":
          handlers.onWarning?.(payload.message ?? data);
          break;
        case "error":
          handlers.onError?.(payload.code ?? "error", payload.message ?? data);
          break;
      }
    } catch {
      /* ignore malformed chunks */
    }
  }
  return remainder;
}

export function useChatStream(getToken: () => string | null) {
  const [error, setError] = useState<string | null>(null);
  const [streaming, setStreaming] = useState(false);
  const abortRef = useRef<AbortController | null>(null);

  const abort = useCallback(() => {
    abortRef.current?.abort();
    abortRef.current = null;
    setStreaming(false);
  }, []);

  const send = useCallback(
    async (messages: { role: string; content: string }[], handlers: StreamHandlers) => {
      abort();
      setError(null);
      const token = getToken();
      const apiBase = import.meta.env.VITE_API_BASE_URL ?? "";
      const controller = new AbortController();
      abortRef.current = controller;
      setStreaming(true);

      try {
        const resp = await fetch(`${apiBase}/api/v1/chat/stream`, {
          method: "POST",
          headers: {
            Accept: "text/event-stream",
            "Content-Type": "application/json",
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
          },
          body: JSON.stringify({ messages }),
          signal: controller.signal,
          credentials: "include",
        });

        if (!resp.ok) {
          const text = await resp.text();
          throw new Error(text || resp.statusText);
        }

        const reader = resp.body?.getReader();
        if (!reader) throw new Error("No response body");

        const decoder = new TextDecoder();
        let buffer = "";
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          buffer = parseSseChunk(buffer, handlers);
        }
        if (buffer) parseSseChunk(buffer + "\n\n", handlers);
      } catch (e) {
        if ((e as Error).name !== "AbortError") {
          const msg = e instanceof Error ? e.message : String(e);
          setError(msg);
          handlers.onError?.("client_error", msg);
        }
      } finally {
        setStreaming(false);
        abortRef.current = null;
      }
    },
    [abort, getToken],
  );

  return { send, abort, streaming, error, setError };
}
