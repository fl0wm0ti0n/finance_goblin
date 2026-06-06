import { useCallback, useEffect, useRef, useState } from "react";
import { useAuth } from "react-oidc-context";
import { useQuery } from "@tanstack/react-query";
import { apiFetch, Settings } from "../../lib/api";
import { useChatStream, ChatMessage } from "./useChatStream";
import { MessageBubble } from "./MessageBubble";
import { SuggestedPrompts } from "./SuggestedPrompts";
import { PrivacyBadge } from "./PrivacyBadge";
import { ProviderBadge } from "./ProviderBadge";

function newId() {
  return crypto.randomUUID();
}

export function ChatPanel() {
  const auth = useAuth();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [streamWarning, setStreamWarning] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const toolsRef = useRef<{ name: string; at: string }[]>([]);

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => apiFetch<Settings>("/api/v1/settings"),
  });

  const getToken = useCallback(() => auth.user?.access_token ?? null, [auth.user?.access_token]);
  const { send, abort, streaming, error, setError } = useChatStream(getToken);

  const ai = settingsQuery.data?.ai;
  const providerConfigured =
    settingsQuery.data?.provider_configured ??
    settingsQuery.data?.openai_configured ??
    false;
  const allowRaw = settingsQuery.data?.privacy?.allow_raw_transactions ?? false;

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => () => abort(), [abort]);

  const submit = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed || streaming) return;

      if (!providerConfigured) {
        setError(
          "AI provider is not configured. Edit backend config.toml, set env vars, and restart.",
        );
        return;
      }

      const userMsg: ChatMessage = { id: newId(), role: "user", content: trimmed };
      const assistantId = newId();
      setStreamWarning(null);
      setMessages((m) => [
        ...m,
        userMsg,
        { id: assistantId, role: "assistant", content: "", streaming: true },
      ]);
      setInput("");
      toolsRef.current = [];

      const history = [...messages, userMsg].map((m) => ({
        role: m.role,
        content: m.content,
      }));

      await send(history, {
        onToken: (delta) => {
          setMessages((m) =>
            m.map((msg) =>
              msg.id === assistantId ? { ...msg, content: msg.content + delta } : msg,
            ),
          );
        },
        onToolStart: (tool) => {
          toolsRef.current.push({ name: tool, at: new Date().toISOString() });
        },
        onDone: (toolsUsed) => {
          const used = toolsRef.current.length
            ? toolsRef.current
            : toolsUsed.map((name) => ({ name, at: new Date().toISOString() }));
          setMessages((m) =>
            m.map((msg) =>
              msg.id === assistantId
                ? { ...msg, streaming: false, toolsUsed: used.length ? used : undefined }
                : msg,
            ),
          );
        },
        onWarning: (message) => {
          setStreamWarning(message);
        },
        onError: (_code, message) => {
          setMessages((m) =>
            m.map((msg) =>
              msg.id === assistantId
                ? { ...msg, streaming: false, content: msg.content || `Error: ${message}` }
                : msg,
            ),
          );
        },
      });
    },
    [messages, providerConfigured, send, setError, streaming],
  );

  return (
    <div className="chat-panel" style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: "0.75rem",
          flexWrap: "wrap",
          gap: "0.5rem",
        }}
      >
        <PrivacyBadge allowRaw={allowRaw} />
        {ai && <ProviderBadge label={ai.provider_label} isLocal={ai.is_local} />}
        {!providerConfigured && (
          <span className="badge" style={{ background: "#fef3c7", color: "#92400e" }}>
            Provider not configured
          </span>
        )}
      </div>

      {!providerConfigured && (
        <p
          style={{
            background: "#fef3c7",
            color: "#92400e",
            padding: "0.5rem 0.75rem",
            borderRadius: "0.5rem",
            fontSize: "0.9rem",
          }}
        >
          AI chat is disabled until the provider is configured (API key for OpenAI, or base_url for
          compatible endpoints). Edit <code>config/default.toml</code> and restart the backend.
        </p>
      )}

      <div
        style={{
          flex: 1,
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          minHeight: 200,
        }}
      >
        {messages.length === 0 && <SuggestedPrompts onSelect={(p) => submit(p)} />}
        {messages.map((m) => (
          <MessageBubble key={m.id} message={m} />
        ))}
        {streamWarning && (
          <p
            style={{
              fontSize: "0.85rem",
              color: "#92400e",
              background: "#fffbeb",
              padding: "0.5rem 0.75rem",
              borderRadius: "0.5rem",
              marginTop: "0.25rem",
            }}
          >
            {streamWarning}
          </p>
        )}
        <div ref={bottomRef} />
      </div>

      {error && (
        <p style={{ color: "#b91c1c", fontSize: "0.9rem", margin: "0.5rem 0" }}>{error}</p>
      )}

      <form
        onSubmit={(e) => {
          e.preventDefault();
          submit(input);
        }}
        style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}
      >
        <input
          className="input"
          style={{ flex: 1 }}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Ask about your finances…"
          disabled={streaming || !providerConfigured}
        />
        <button
          type="submit"
          className="btn btn-primary"
          disabled={streaming || !input.trim() || !providerConfigured}
        >
          Send
        </button>
        {streaming && (
          <button type="button" className="btn" onClick={abort}>
            Stop
          </button>
        )}
      </form>
    </div>
  );
}
