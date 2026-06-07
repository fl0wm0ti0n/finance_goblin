import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  apiFetch,
  SubscriptionAlert,
  SubscriptionPattern,
  SubscriptionUnreadCount,
} from "../lib/api";

const PriceHistoryChart = lazy(() =>
  import("../components/subscriptions/PriceHistoryChart").then((m) => ({
    default: m.PriceHistoryChart,
  })),
);

type Tab = "all" | "pending" | "standing";

function confidenceClass(pct: number) {
  if (pct >= 95) return "badge confidence-high";
  if (pct >= 80) return "badge confidence-medium";
  return "badge confidence-low";
}

function intervalLabel(days: number) {
  if (days <= 8) return "Weekly";
  if (days <= 16) return "Biweekly";
  if (days <= 32) return "Monthly";
  if (days <= 95) return "Quarterly";
  return `Every ${days} days`;
}

export function SubscriptionsPage() {
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab>("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [confirmKind, setConfirmKind] = useState<"subscription" | "standing_order">("subscription");
  const [toast, setToast] = useState<string | null>(null);

  const listQuery = useQuery({
    queryKey: ["subscriptions", tab],
    queryFn: () => {
      const params = new URLSearchParams();
      if (tab === "pending") params.set("status", "pending");
      if (tab === "standing") {
        params.set("status", "confirmed");
        params.set("kind", "standing_order");
      }
      const qs = params.toString();
      return apiFetch<SubscriptionPattern[]>(`/api/v1/subscriptions${qs ? `?${qs}` : ""}`);
    },
  });

  const pendingQuery = useQuery({
    queryKey: ["subscriptions", "pending-count"],
    queryFn: () => apiFetch<SubscriptionPattern[]>("/api/v1/subscriptions?status=pending"),
  });

  const unreadCountQuery = useQuery({
    queryKey: ["subscription-unread-count"],
    queryFn: () =>
      apiFetch<SubscriptionUnreadCount>("/api/v1/subscriptions/alerts/unread-count"),
    refetchInterval: 30000,
  });

  const alertsQuery = useQuery({
    queryKey: ["subscription-alerts"],
    queryFn: () => apiFetch<SubscriptionAlert[]>("/api/v1/subscriptions/alerts?unread=true"),
    refetchInterval: 30000,
  });

  const detailQuery = useQuery({
    queryKey: ["subscription-detail", selectedId],
    queryFn: () => apiFetch<SubscriptionPattern>(`/api/v1/subscriptions/${selectedId}`),
    enabled: !!selectedId,
  });

  const priceHistoryQuery = useQuery({
    queryKey: ["subscription-price-history", selectedId],
    queryFn: () =>
      apiFetch<{ events: { occurred_at: string; amount: string; event_type: string }[] }>(
        `/api/v1/subscriptions/${selectedId}/price-history`,
      ),
    enabled: !!selectedId,
  });

  useEffect(() => {
    const prev = sessionStorage.getItem("subscription-unread-new-detection");
    const current = String(unreadCountQuery.data?.unread_new_detection ?? 0);
    if (prev && Number(prev) < Number(current)) {
      setToast("New subscription alert — review pending patterns.");
    }
    sessionStorage.setItem("subscription-unread-new-detection", current);
  }, [unreadCountQuery.data?.unread_new_detection]);

  const pendingCount = pendingQuery.data?.length ?? 0;
  const unreadNewDetection = unreadCountQuery.data?.unread_new_detection ?? 0;
  const pendingFromCount = unreadCountQuery.data?.pending_patterns ?? pendingCount;
  const showReconciliationSubtitle =
    unreadCountQuery.data != null &&
    unreadCountQuery.data.unread_new_detection !== unreadCountQuery.data.pending_patterns;

  const confirmMutation = useMutation({
    mutationFn: ({ id, kind }: { id: string; kind?: string }) =>
      apiFetch<SubscriptionPattern>(`/api/v1/subscriptions/${id}/confirm`, {
        method: "POST",
        body: JSON.stringify(kind ? { kind } : {}),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-unread-count"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-alerts"] });
      setConfirmId(null);
    },
  });

  const rejectMutation = useMutation({
    mutationFn: (id: string) =>
      apiFetch<SubscriptionPattern>(`/api/v1/subscriptions/${id}/reject`, {
        method: "POST",
        body: JSON.stringify({}),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-unread-count"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-alerts"] });
    },
  });

  const markReadMutation = useMutation({
    mutationFn: (id: string) =>
      apiFetch<void>(`/api/v1/subscriptions/alerts/${id}/read`, { method: "PATCH" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["subscription-alerts"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-unread-count"] });
    },
  });

  const patterns = listQuery.data ?? [];
  const pendingPatterns = useMemo(
    () => (tab === "pending" ? patterns : patterns.filter((p) => p.status === "pending")),
    [patterns, tab],
  );
  const displayPatterns =
    tab === "pending"
      ? pendingPatterns
      : tab === "standing"
        ? patterns
        : patterns;

  const empty = !listQuery.isLoading && patterns.length === 0;
  const showPendingBanner =
    !listQuery.isLoading &&
    tab !== "pending" &&
    patterns.length === 0 &&
    pendingCount > 0;

  return (
    <div>
      {toast && (
        <div className="card" style={{ marginBottom: "1rem", background: "#ecfdf5" }}>
          {toast}
          <button className="btn" style={{ marginLeft: "1rem" }} onClick={() => setToast(null)}>
            Dismiss
          </button>
        </div>
      )}

      {unreadNewDetection > 0 && (
        <div className="card alert-banner" style={{ marginBottom: "1rem" }}>
          <strong>
            {unreadNewDetection} unread alert{unreadNewDetection === 1 ? "" : "s"}
          </strong>
          {showReconciliationSubtitle && (
            <p style={{ margin: "0.5rem 0 0" }}>
              {pendingFromCount} pattern{pendingFromCount === 1 ? "" : "s"} pending review — some
              alerts may be informational or awaiting cleanup.
            </p>
          )}
          <ul style={{ margin: "0.5rem 0 0", paddingLeft: "1.25rem" }}>
            {(alertsQuery.data ?? []).slice(0, 5).map((a) => (
              <li key={a.id} style={{ display: "flex", justifyContent: "space-between", gap: "1rem" }}>
                <span>{a.title}</span>
                <button className="btn" onClick={() => markReadMutation.mutate(a.id)}>
                  Mark read
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="card" style={{ marginBottom: "1rem" }}>
        <h1>Subscriptions</h1>
        <p>Review detected recurring patterns and confirm or reject candidates.</p>
        <div className="tabs" style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
          {(["all", "pending", "standing"] as Tab[]).map((t) => (
            <button
              key={t}
              className={`btn ${tab === t ? "active" : ""}`}
              onClick={() => setTab(t)}
            >
              {t === "all" ? "All" : t === "pending" ? "Pending review" : "Standing orders"}
            </button>
          ))}
        </div>
      </div>

      {showPendingBanner && (
        <div className="card alert-banner" style={{ marginBottom: "1rem" }}>
          <strong>
            {pendingCount} pattern{pendingCount === 1 ? "" : "s"} pending review
          </strong>
          <p style={{ margin: "0.5rem 0 0" }}>
            Confirmed and standing-order tabs only show patterns you have approved. Switch to{" "}
            <button className="btn" type="button" onClick={() => setTab("pending")}>
              Pending review
            </button>{" "}
            to confirm or reject candidates.
          </p>
        </div>
      )}

      {empty && (
        <div className="card">
          <p>No subscription patterns detected yet.</p>
          <p style={{ marginTop: "0.75rem" }}>
            Detection runs on a <strong>Full Firefly sync</strong> only (not exchange-only sync).
            Recurring expenses need at least <strong>3 matching transactions</strong> with{" "}
            <strong>≥60% confidence</strong>. Payee grouping uses the transaction description,
            then counterparty or destination names when the description is missing.
          </p>
          <p style={{ marginTop: "0.75rem" }}>
            <Link to="/sync">Go to Sync Status</Link> to run a Full sync and refresh detection.
          </p>
        </div>
      )}

      {tab === "pending" && pendingPatterns.length > 0 && (
        <div className="grid">
          {pendingPatterns.map((p) => (
            <div key={p.id} className="card">
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <strong>{p.display_name}</strong>
                <span className={confidenceClass(p.confidence_pct)}>{p.confidence_pct}%</span>
              </div>
              <p>
                {intervalLabel(p.interval_days)} · €{Math.abs(Number(p.current_amount)).toFixed(2)}
              </p>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <button className="btn" onClick={() => { setConfirmId(p.id); setConfirmKind(p.kind as "subscription" | "standing_order"); }}>
                  Confirm
                </button>
                <button className="btn" onClick={() => rejectMutation.mutate(p.id)}>
                  Reject
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {tab !== "pending" && displayPatterns.length > 0 && (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Interval</th>
                <th>Amount</th>
                <th>Kind</th>
                <th>Last seen</th>
              </tr>
            </thead>
            <tbody>
              {displayPatterns.map((p) => (
                <tr key={p.id} style={{ cursor: "pointer" }} onClick={() => setSelectedId(p.id)}>
                  <td>{p.display_name}</td>
                  <td>{intervalLabel(p.interval_days)}</td>
                  <td>€{Math.abs(Number(p.current_amount)).toFixed(2)}</td>
                  <td>
                    <span className="badge">{p.kind.replace("_", " ")}</span>
                  </td>
                  <td>{p.last_seen_at}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {confirmId && (
        <div className="modal-overlay" onClick={() => setConfirmId(null)}>
          <div className="card modal" onClick={(e) => e.stopPropagation()}>
            <h2>Confirm pattern</h2>
            <label>
              Kind override
              <select
                value={confirmKind}
                onChange={(e) => setConfirmKind(e.target.value as "subscription" | "standing_order")}
                style={{ display: "block", marginTop: "0.5rem" }}
              >
                <option value="subscription">Subscription</option>
                <option value="standing_order">Standing order</option>
              </select>
            </label>
            <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
              <button
                className="btn"
                onClick={() => confirmMutation.mutate({ id: confirmId, kind: confirmKind })}
              >
                Confirm
              </button>
              <button className="btn" onClick={() => setConfirmId(null)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {selectedId && (
        <div className="drawer-overlay" onClick={() => setSelectedId(null)}>
          <div className="drawer card" onClick={(e) => e.stopPropagation()}>
            <button className="btn" onClick={() => setSelectedId(null)} style={{ float: "right" }}>
              Close
            </button>
            <h2>{detailQuery.data?.display_name ?? "Pattern detail"}</h2>
            {detailQuery.data && (
              <>
                <p>
                  {intervalLabel(detailQuery.data.interval_days)} · €
                  {Math.abs(Number(detailQuery.data.current_amount)).toFixed(2)} ·{" "}
                  {detailQuery.data.transaction_count ?? 0} linked transactions
                </p>
                <Suspense fallback={<p>Loading chart…</p>}>
                  <PriceHistoryChart events={priceHistoryQuery.data?.events ?? []} />
                </Suspense>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
