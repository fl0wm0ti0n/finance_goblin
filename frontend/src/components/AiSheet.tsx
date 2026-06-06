import { useState } from "react";
import { ChatPanel } from "./chat/ChatPanel";

export function AiSheet() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="btn"
        onClick={() => setOpen(true)}
        aria-label="Open AI assistant"
      >
        AI
      </button>
      {open && (
        <div
          role="dialog"
          aria-modal="true"
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 50,
            display: "flex",
            justifyContent: "flex-end",
          }}
        >
          <div
            style={{ flex: 1, background: "rgba(0,0,0,0.3)" }}
            onClick={() => setOpen(false)}
          />
          <aside
            style={{
              width: "min(400px, 100vw)",
              background: "#fff",
              boxShadow: "-4px 0 24px rgba(0,0,0,0.12)",
              display: "flex",
              flexDirection: "column",
              padding: "1rem",
              maxHeight: "100vh",
            }}
          >
            <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "0.5rem" }}>
              <strong>AI Assistant</strong>
              <button type="button" className="btn" onClick={() => setOpen(false)}>
                Close
              </button>
            </div>
            <div style={{ flex: 1, minHeight: 0 }}>
              <ChatPanel />
            </div>
          </aside>
        </div>
      )}
    </>
  );
}
