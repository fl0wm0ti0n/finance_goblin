import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  apiFetch,
  assignSubscriptionTags,
  confirmDiscoverCandidate,
  createOperatorTag,
  deleteOperatorTag,
  DiscoverCandidate,
  fetchCategories,
  fetchDiscover,
  fetchOperatorTags,
  ForecastAccount,
  OperatorTag,
  renameOperatorTag,
  SubscriptionAlert,
  SubscriptionPattern,
  SubscriptionUnreadCount,
} from "../lib/api";

const PriceHistoryChart = lazy(() =>
  import("../components/subscriptions/PriceHistoryChart").then((m) => ({
    default: m.PriceHistoryChart,
  })),
);

type Tab = "all" | "pending" | "standing" | "discover";

const INTERVAL_BUCKETS = [
  { label: "Weekly", days: 7 },
  { label: "Biweekly", days: 14 },
  { label: "Monthly", days: 30 },
  { label: "Quarterly", days: 90 },
] as const;

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

function categoryLabel(
  categoryId: string | null | undefined,
  catalog: Map<string, string>,
): string {
  if (!categoryId) return "Uncategorized";
  return catalog.get(categoryId) ?? categoryId;
}

export function SubscriptionsPage() {
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab>("all");
  const [tagFilter, setTagFilter] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [confirmKind, setConfirmKind] = useState<"subscription" | "standing_order">("subscription");
  const [toast, setToast] = useState<string | null>(null);
  const [tagManagerOpen, setTagManagerOpen] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [renameTagId, setRenameTagId] = useState<string | null>(null);
  const [renameTagName, setRenameTagName] = useState("");
  const [deleteTagId, setDeleteTagId] = useState<string | null>(null);
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);

  const [discoverAccountId, setDiscoverAccountId] = useState("");
  const [discoverPayee, setDiscoverPayee] = useState("");
  const [discoverInterval, setDiscoverInterval] = useState<number | "">("");
  const [discoverCustomInterval, setDiscoverCustomInterval] = useState("");
  const [discoverSearched, setDiscoverSearched] = useState(false);
  const [discoverConfirm, setDiscoverConfirm] = useState<DiscoverCandidate | null>(null);

  const accountsQuery = useQuery({
    queryKey: ["forecast-accounts"],
    queryFn: () => apiFetch<ForecastAccount[]>("/api/v1/forecast/accounts"),
  });

  const categoriesQuery = useQuery({
    queryKey: ["categories-catalog"],
    queryFn: () => fetchCategories(),
  });

  const categoryMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const c of categoriesQuery.data?.categories ?? []) {
      map.set(c.id, c.name);
    }
    return map;
  }, [categoriesQuery.data]);

  useEffect(() => {
    if (!discoverAccountId && accountsQuery.data?.[0]?.id) {
      setDiscoverAccountId(accountsQuery.data[0].id);
    }
  }, [accountsQuery.data, discoverAccountId]);

  const listQuery = useQuery({
    queryKey: ["subscriptions", tab, tagFilter],
    queryFn: () => {
      const params = new URLSearchParams();
      if (tab === "pending") params.set("status", "pending");
      if (tab === "standing") {
        params.set("status", "confirmed");
        params.set("kind", "standing_order");
      }
      if (tagFilter) params.set("tag", tagFilter);
      const qs = params.toString();
      return apiFetch<SubscriptionPattern[]>(`/api/v1/subscriptions${qs ? `?${qs}` : ""}`);
    },
    enabled: tab !== "discover",
  });

  const tagsQuery = useQuery({
    queryKey: ["operator-tags"],
    queryFn: fetchOperatorTags,
  });

  const discoverQuery = useQuery({
    queryKey: [
      "subscriptions-discover",
      discoverAccountId,
      discoverPayee,
      discoverInterval,
      discoverCustomInterval,
    ],
    queryFn: () => {
      const interval =
        discoverInterval === "custom"
          ? Number(discoverCustomInterval)
          : discoverInterval === ""
            ? undefined
            : discoverInterval;
      return fetchDiscover({
        account_id: discoverAccountId,
        payee: discoverPayee || undefined,
        interval_days: interval,
      });
    },
    enabled: discoverSearched && !!discoverAccountId && tab === "discover",
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
    if (detailQuery.data?.tags) {
      setSelectedTagIds(detailQuery.data.tags.map((t) => t.id));
    }
  }, [detailQuery.data?.tags]);

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

  const discoverConfirmMutation = useMutation({
    mutationFn: (candidate: DiscoverCandidate) =>
      confirmDiscoverCandidate({
        payee_key: candidate.payee_key,
        interval_days: candidate.interval_days,
        median_amount: candidate.median_amount,
        transaction_ids: candidate.transaction_ids,
        kind: confirmKind,
      }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["subscriptions-discover"] });
      setDiscoverConfirm(null);
      setToast(data.merged ? "Merged with existing confirmed subscription." : "Subscription confirmed.");
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

  const assignTagsMutation = useMutation({
    mutationFn: ({ id, tagIds }: { id: string; tagIds: string[] }) =>
      assignSubscriptionTags(id, tagIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-detail"] });
    },
  });

  const createTagMutation = useMutation({
    mutationFn: (name: string) => createOperatorTag(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["operator-tags"] });
      setNewTagName("");
    },
  });

  const renameTagMutation = useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) => renameOperatorTag(id, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["operator-tags"] });
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      setRenameTagId(null);
    },
  });

  const deleteTagMutation = useMutation({
    mutationFn: (id: string) => deleteOperatorTag(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["operator-tags"] });
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      setDeleteTagId(null);
      if (tagFilter) setTagFilter(null);
    },
  });

  const patterns = listQuery.data ?? [];
  const pendingPatterns = useMemo(
    () => (tab === "pending" ? patterns : patterns.filter((p) => p.status === "pending")),
    [patterns, tab],
  );
  const displayPatterns =
    tab === "pending" ? pendingPatterns : tab === "standing" ? patterns : patterns;

  const empty = tab !== "discover" && !listQuery.isLoading && patterns.length === 0;
  const showPendingBanner =
    tab !== "discover" &&
    !listQuery.isLoading &&
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
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
          <div>
            <h1>Subscriptions</h1>
            <p>Review detected recurring patterns, discover candidates manually, and organize with tags.</p>
          </div>
          <button className="btn" type="button" onClick={() => setTagManagerOpen(true)}>
            Manage tags
          </button>
        </div>
        <div className="tabs" style={{ display: "flex", gap: "0.5rem", marginTop: "1rem", flexWrap: "wrap" }}>
          {(["all", "pending", "standing", "discover"] as Tab[]).map((t) => (
            <button
              key={t}
              className={`btn ${tab === t ? "active" : ""}`}
              onClick={() => setTab(t)}
            >
              {t === "all"
                ? "All"
                : t === "pending"
                  ? "Pending review"
                  : t === "standing"
                    ? "Standing orders"
                    : "Discover"}
            </button>
          ))}
        </div>
        {(tab === "all" || tab === "standing") && (tagsQuery.data?.length ?? 0) > 0 && (
          <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
            <button
              className={`btn ${tagFilter === null ? "active" : ""}`}
              onClick={() => setTagFilter(null)}
            >
              All tags
            </button>
            {(tagsQuery.data ?? []).map((tag: OperatorTag) => (
              <button
                key={tag.id}
                className={`btn ${tagFilter === tag.slug ? "active" : ""}`}
                onClick={() => setTagFilter(tag.slug)}
              >
                {tag.name}
              </button>
            ))}
          </div>
        )}
      </div>

      {tab === "discover" && (
        <div className="card" style={{ marginBottom: "1rem" }}>
          <h2>Discover recurring candidates</h2>
          <p>Search expense transactions for recurring patterns not yet confirmed. Account is required.</p>
          <div style={{ display: "grid", gap: "1rem", marginTop: "1rem" }}>
            <label>
              Account (required)
              <select
                value={discoverAccountId}
                onChange={(e) => setDiscoverAccountId(e.target.value)}
                style={{ display: "block", marginTop: "0.25rem", minWidth: "16rem" }}
              >
                <option value="">Select account…</option>
                {(accountsQuery.data ?? []).map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Payee contains
              <input
                type="text"
                value={discoverPayee}
                onChange={(e) => setDiscoverPayee(e.target.value)}
                placeholder="e.g. netflix"
                style={{ display: "block", marginTop: "0.25rem", width: "100%", maxWidth: "24rem" }}
              />
            </label>
            <label>
              Interval bucket
              <select
                value={discoverInterval === "" ? "" : discoverInterval === "custom" ? "custom" : String(discoverInterval)}
                onChange={(e) => {
                  const v = e.target.value;
                  if (v === "") setDiscoverInterval("");
                  else if (v === "custom") setDiscoverInterval("custom");
                  else setDiscoverInterval(Number(v));
                }}
                style={{ display: "block", marginTop: "0.25rem" }}
              >
                <option value="">Any interval</option>
                {INTERVAL_BUCKETS.map((b) => (
                  <option key={b.days} value={b.days}>
                    {b.label} ({b.days}d)
                  </option>
                ))}
                <option value="custom">Custom</option>
              </select>
            </label>
            {discoverInterval === "custom" && (
              <label>
                Custom interval (days)
                <input
                  type="number"
                  min={1}
                  value={discoverCustomInterval}
                  onChange={(e) => setDiscoverCustomInterval(e.target.value)}
                  style={{ display: "block", marginTop: "0.25rem", width: "8rem" }}
                />
              </label>
            )}
            <button
              className="btn"
              disabled={!discoverAccountId}
              onClick={() => setDiscoverSearched(true)}
            >
              Search
            </button>
          </div>
          {discoverSearched && discoverQuery.isLoading && <p style={{ marginTop: "1rem" }}>Searching…</p>}
          {discoverQuery.data?.meta.truncated && (
            <p style={{ marginTop: "1rem", color: "#b45309" }}>
              Showing top {discoverQuery.data.meta.limit} candidates — refine payee or interval to narrow results.
            </p>
          )}
          {discoverSearched && !discoverQuery.isLoading && (
            <table style={{ marginTop: "1rem", width: "100%" }}>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Interval</th>
                  <th>Median</th>
                  <th>Confidence</th>
                  <th>Tx count</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {(discoverQuery.data?.candidates ?? []).map((c) => (
                  <tr key={`${c.payee_key}-${c.interval_days}-${c.median_amount}`}>
                    <td>{c.display_name}</td>
                    <td>{intervalLabel(c.interval_days)}</td>
                    <td>€{Math.abs(c.median_amount).toFixed(2)}</td>
                    <td>
                      <span className={confidenceClass(c.confidence_pct)}>{c.confidence_pct}%</span>
                    </td>
                    <td>{c.transaction_count}</td>
                    <td>
                      <button
                        className="btn"
                        onClick={() => {
                          setConfirmKind("subscription");
                          setDiscoverConfirm(c);
                        }}
                      >
                        Confirm
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {discoverSearched &&
            !discoverQuery.isLoading &&
            (discoverQuery.data?.candidates.length ?? 0) === 0 && (
              <p style={{ marginTop: "1rem" }}>No candidates match your filters.</p>
            )}
        </div>
      )}

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
            <strong>≥60% confidence</strong>. Use the <strong>Discover</strong> tab to search manually.
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
                <button
                  className="btn"
                  onClick={() => {
                    setConfirmId(p.id);
                    setConfirmKind(p.kind as "subscription" | "standing_order");
                  }}
                >
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

      {tab !== "pending" && tab !== "discover" && displayPatterns.length > 0 && (
        <div className="card">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Category</th>
                <th>Interval</th>
                <th>Amount</th>
                <th>Kind</th>
                <th>Tags</th>
                <th>Last seen</th>
              </tr>
            </thead>
            <tbody>
              {displayPatterns.map((p) => (
                <tr key={p.id} style={{ cursor: "pointer" }} onClick={() => setSelectedId(p.id)}>
                  <td>{p.display_name}</td>
                  <td>
                    <span
                      className="badge"
                      title="Majority category from linked transactions; ties broken by most recent charge."
                    >
                      {categoryLabel(p.display_category_id, categoryMap)}
                    </span>
                  </td>
                  <td>{intervalLabel(p.interval_days)}</td>
                  <td>€{Math.abs(Number(p.current_amount)).toFixed(2)}</td>
                  <td>
                    <span className="badge">{p.kind.replace("_", " ")}</span>
                  </td>
                  <td>
                    {(p.tags ?? []).map((t) => (
                      <span key={t.id} className="badge" style={{ marginRight: "0.25rem" }}>
                        {t.name}
                      </span>
                    ))}
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

      {discoverConfirm && (
        <div className="modal-overlay" onClick={() => setDiscoverConfirm(null)}>
          <div className="card modal" onClick={(e) => e.stopPropagation()}>
            <h2>Confirm discover candidate</h2>
            <p>
              {discoverConfirm.display_name} · {intervalLabel(discoverConfirm.interval_days)} · €
              {Math.abs(discoverConfirm.median_amount).toFixed(2)}
            </p>
            <label>
              Kind
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
                onClick={() => discoverConfirmMutation.mutate(discoverConfirm)}
              >
                Confirm
              </button>
              <button className="btn" onClick={() => setDiscoverConfirm(null)}>
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
                <p>
                  <span
                    className="badge"
                    title="Majority category from linked transactions; ties broken by most recent charge."
                  >
                    {categoryLabel(detailQuery.data.display_category_id, categoryMap)}
                  </span>
                </p>
                <div style={{ marginTop: "1rem" }}>
                  <strong>Tags</strong>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem", marginTop: "0.5rem" }}>
                    {(tagsQuery.data ?? []).map((tag) => (
                      <label key={tag.id} style={{ display: "flex", alignItems: "center", gap: "0.25rem" }}>
                        <input
                          type="checkbox"
                          checked={selectedTagIds.includes(tag.id)}
                          onChange={(e) => {
                            setSelectedTagIds((prev) =>
                              e.target.checked
                                ? [...prev, tag.id]
                                : prev.filter((id) => id !== tag.id),
                            );
                          }}
                        />
                        {tag.name}
                      </label>
                    ))}
                  </div>
                  <button
                    className="btn"
                    style={{ marginTop: "0.5rem" }}
                    onClick={() =>
                      assignTagsMutation.mutate({ id: selectedId, tagIds: selectedTagIds })
                    }
                  >
                    Save tags
                  </button>
                </div>
                <Suspense fallback={<p>Loading chart…</p>}>
                  <PriceHistoryChart events={priceHistoryQuery.data?.events ?? []} />
                </Suspense>
              </>
            )}
          </div>
        </div>
      )}

      {tagManagerOpen && (
        <div className="modal-overlay" onClick={() => setTagManagerOpen(false)}>
          <div className="card modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: "28rem" }}>
            <h2>Operator tags</h2>
            <p>Tags are stored in the product database only — not synced to Firefly.</p>
            <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
              <input
                type="text"
                value={newTagName}
                onChange={(e) => setNewTagName(e.target.value)}
                placeholder="New tag name"
                style={{ flex: 1 }}
              />
              <button
                className="btn"
                disabled={!newTagName.trim()}
                onClick={() => createTagMutation.mutate(newTagName.trim())}
              >
                Create
              </button>
            </div>
            <ul style={{ marginTop: "1rem", paddingLeft: 0, listStyle: "none" }}>
              {(tagsQuery.data ?? []).map((tag) => (
                <li
                  key={tag.id}
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    marginBottom: "0.5rem",
                  }}
                >
                  {renameTagId === tag.id ? (
                    <>
                      <input
                        value={renameTagName}
                        onChange={(e) => setRenameTagName(e.target.value)}
                      />
                      <button
                        className="btn"
                        onClick={() =>
                          renameTagMutation.mutate({ id: tag.id, name: renameTagName })
                        }
                      >
                        Save
                      </button>
                    </>
                  ) : (
                    <>
                      <span>
                        {tag.name} <small>({tag.slug})</small>
                      </span>
                      <span>
                        <button
                          className="btn"
                          onClick={() => {
                            setRenameTagId(tag.id);
                            setRenameTagName(tag.name);
                          }}
                        >
                          Rename
                        </button>
                        <button className="btn" onClick={() => setDeleteTagId(tag.id)}>
                          Delete
                        </button>
                      </span>
                    </>
                  )}
                </li>
              ))}
            </ul>
            <button className="btn" style={{ marginTop: "1rem" }} onClick={() => setTagManagerOpen(false)}>
              Close
            </button>
          </div>
        </div>
      )}

      {deleteTagId && (
        <div className="modal-overlay" onClick={() => setDeleteTagId(null)}>
          <div className="card modal" onClick={(e) => e.stopPropagation()}>
            <h2>Delete tag?</h2>
            <p>This removes the tag from all subscriptions. Patterns are not deleted.</p>
            <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
              <button className="btn" onClick={() => deleteTagMutation.mutate(deleteTagId)}>
                Delete
              </button>
              <button className="btn" onClick={() => setDeleteTagId(null)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
