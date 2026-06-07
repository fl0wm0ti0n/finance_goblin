import { useCallback, useEffect, useRef, useState } from "react";

export type PlanningFeedbackKind = "success" | "error";

export type PlanningFeedbackState = {
  kind: PlanningFeedbackKind;
  message: string;
} | null;

const MAX_MESSAGE_LEN = 240;
const SUCCESS_DISMISS_MS = 4000;

export function formatPlanningError(err: unknown, fallback: string): string {
  let body = "";
  if (err instanceof Error && err.message.trim()) {
    body = err.message.trim();
    try {
      const parsed = JSON.parse(body) as { message?: string; error?: string };
      if (parsed.message?.trim()) {
        body = parsed.message.trim();
      } else if (parsed.error?.trim()) {
        body = parsed.error.replace(/_/g, " ");
      }
    } catch {
      // not JSON — use raw message
    }
  }
  if (body.length > MAX_MESSAGE_LEN) {
    body = `${body.slice(0, MAX_MESSAGE_LEN)}…`;
  }
  if (!body) {
    return `${fallback}: Request failed`;
  }
  return body;
}

export function usePlanningFeedback() {
  const [feedback, setFeedback] = useState<PlanningFeedbackState>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const dismissFeedback = useCallback(() => {
    clearTimer();
    setFeedback(null);
  }, [clearTimer]);

  const showPlanningFeedback = useCallback(
    ({ kind, message }: { kind: PlanningFeedbackKind; message: string }) => {
      clearTimer();
      const trimmed =
        message.length > MAX_MESSAGE_LEN ? `${message.slice(0, MAX_MESSAGE_LEN)}…` : message;
      setFeedback({ kind, message: trimmed });
      if (kind === "success") {
        timerRef.current = setTimeout(() => setFeedback(null), SUCCESS_DISMISS_MS);
      }
    },
    [clearTimer],
  );

  useEffect(() => () => clearTimer(), [clearTimer]);

  return { feedback, showPlanningFeedback, dismissFeedback };
}

export function PlanningFeedbackCard({
  feedback,
  onDismiss,
}: {
  feedback: PlanningFeedbackState;
  onDismiss: () => void;
}) {
  if (!feedback) {
    return null;
  }

  const background = feedback.kind === "success" ? "#ecfdf5" : "#fef2f2";

  return (
    <div className="card" style={{ marginBottom: "1rem", background }}>
      <span>{feedback.message}</span>
      {feedback.kind === "error" && (
        <button className="btn" style={{ marginLeft: "1rem" }} onClick={onDismiss} type="button">
          Dismiss
        </button>
      )}
    </div>
  );
}
