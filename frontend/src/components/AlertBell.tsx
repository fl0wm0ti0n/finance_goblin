import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { AlertRow, apiFetch, SubscriptionAlert } from "../lib/api";

export function AlertBell() {
  const [open, setOpen] = useState(false);

  const unreadQuery = useQuery({
    queryKey: ["alerts-unread"],
    queryFn: () => apiFetch<{ count: number }>("/api/v1/alerts/unread-count"),
    refetchInterval: 30_000,
  });

  const previewQuery = useQuery({
    queryKey: ["alerts-preview"],
    queryFn: () => apiFetch<AlertRow[]>("/api/v1/alerts?status=active"),
    enabled: open,
  });

  const subAlertsQuery = useQuery({
    queryKey: ["subscription-alerts-unread"],
    queryFn: () => apiFetch<SubscriptionAlert[]>("/api/v1/subscriptions/alerts?unread=true"),
    enabled: open,
  });

  const count = unreadQuery.data?.count ?? 0;
  const preview = (previewQuery.data ?? []).slice(0, 5);
  const subUnread = subAlertsQuery.data?.length ?? 0;

  return (
    <div style={{ position: "relative" }}>
      <button
        className="btn"
        style={{ position: "relative", padding: "0.35rem 0.75rem" }}
        onClick={() => setOpen((o) => !o)}
        aria-label="Alerts"
      >
        🔔
        {count > 0 && (
          <span
            style={{
              position: "absolute",
              top: -4,
              right: -4,
              background: "#ef4444",
              color: "white",
              borderRadius: "999px",
              fontSize: "0.7rem",
              minWidth: "1.1rem",
              textAlign: "center",
            }}
          >
            {count}
          </span>
        )}
      </button>
      {open && (
        <div
          className="card"
          style={{
            position: "absolute",
            right: 0,
            top: "100%",
            marginTop: "0.5rem",
            width: "320px",
            zIndex: 50,
            boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
          }}
        >
          <strong>Recent alerts</strong>
          {preview.length === 0 ? (
            <p style={{ margin: "0.5rem 0", fontSize: "0.9rem" }}>No active alerts</p>
          ) : (
            <ul style={{ margin: "0.5rem 0", paddingLeft: "1.25rem", fontSize: "0.9rem" }}>
              {preview.map((a) => (
                <li key={a.id}>
                  <span className={`badge severity-${a.severity}`} style={{ marginRight: "0.25rem" }}>
                    {a.severity}
                  </span>
                  {a.title}
                </li>
              ))}
            </ul>
          )}
          <Link to="/alerts" onClick={() => setOpen(false)}>
            View all alerts →
          </Link>
          {subUnread > 0 && (
            <p style={{ margin: "0.75rem 0 0", fontSize: "0.85rem" }}>
              <Link to="/subscriptions" onClick={() => setOpen(false)}>
                View subscription alerts ({subUnread})
              </Link>
            </p>
          )}
        </div>
      )}
    </div>
  );
}
