import type { ChatMessage } from "./useChatStream";

export function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";
  return (
    <div
      className={`chat-bubble ${isUser ? "chat-bubble-user" : "chat-bubble-assistant"}`}
      style={{
        alignSelf: isUser ? "flex-end" : "flex-start",
        maxWidth: "90%",
        padding: "0.75rem 1rem",
        borderRadius: "0.75rem",
        background: isUser ? "#e0f2fe" : "#f1f5f9",
        marginBottom: "0.5rem",
      }}
    >
      <div style={{ whiteSpace: "pre-wrap" }}>
        {message.content}
        {message.streaming && <span className="chat-cursor">▌</span>}
      </div>
      {message.toolsUsed && message.toolsUsed.length > 0 && (
        <details style={{ marginTop: "0.5rem", fontSize: "0.8rem", color: "#64748b" }}>
          <summary>Tools used</summary>
          <ul style={{ margin: "0.25rem 0 0", paddingLeft: "1.25rem" }}>
            {message.toolsUsed.map((t, i) => (
              <li key={`${t.name}-${i}`}>
                {t.name} — {new Date(t.at).toLocaleTimeString()}
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}
