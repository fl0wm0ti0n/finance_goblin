import { ChatPanel } from "../components/chat/ChatPanel";

export function ChatPage() {
  return (
    <div className="card" style={{ height: "calc(100vh - 8rem)", display: "flex", flexDirection: "column" }}>
      <h1>AI Assistant</h1>
      <p style={{ color: "#64748b", marginBottom: "1rem" }}>
        Ask natural-language questions about cashflow, subscriptions, plans, and wealth. Tools use
        read-only in-process services only.
      </p>
      <div style={{ flex: 1, minHeight: 0 }}>
        <ChatPanel />
      </div>
    </div>
  );
}
