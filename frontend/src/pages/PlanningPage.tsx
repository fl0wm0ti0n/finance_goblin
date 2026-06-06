import { lazy, Suspense, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  apiFetch,
  PlanAdjustment,
  PlanCompare,
  PlanDetail,
  PlanListItem,
  PlanRiskScoreResponse,
  PlanVsActual,
  SavingsSuggestion,
} from "../lib/api";

const CompareChart = lazy(() =>
  import("../components/planning/CompareChart").then((m) => ({ default: m.CompareChart })),
);
const PlanVsActualChart = lazy(() =>
  import("../components/planning/PlanVsActualChart").then((m) => ({
    default: m.PlanVsActualChart,
  })),
);

type Tab = "scenarios" | "compare" | "plan-vs-actual";

const TEMPLATES = [
  { id: "current", label: "Current (Ist)", desc: "Baseline forecast only" },
  { id: "leasing", label: "Leasing", desc: "+€300/month household outflow" },
  { id: "savings_mode", label: "Savings mode", desc: "Remove subscriptions + optional cut" },
  { id: "house_purchase", label: "House purchase", desc: "+€500/month savings transfer" },
  { id: "allocation_target", label: "Allocation target", desc: "50/50 ETF vs crypto weights" },
  { id: "custom", label: "Custom", desc: "Start empty and add lines" },
];

export function PlanningPage() {
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab>("scenarios");
  const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null);
  const [newPlanName, setNewPlanName] = useState("");
  const [savingsOpen, setSavingsOpen] = useState(false);
  const [savingsTemplateVersionId, setSavingsTemplateVersionId] = useState<string | null>(null);
  const [selectedPayees, setSelectedPayees] = useState<string[]>([]);
  const [discretionaryCut, setDiscretionaryCut] = useState(false);
  const [month, setMonth] = useState<string>("");

  const plansQuery = useQuery({
    queryKey: ["plans"],
    queryFn: () => apiFetch<PlanListItem[]>("/api/v1/plans"),
  });

  const activePlanId = useMemo(() => {
    const active = plansQuery.data?.find((p) => p.is_active);
    return active?.id ?? selectedPlanId ?? plansQuery.data?.[0]?.id ?? null;
  }, [plansQuery.data, selectedPlanId]);

  const detailQuery = useQuery({
    queryKey: ["plan-detail", activePlanId],
    queryFn: () => apiFetch<PlanDetail>(`/api/v1/plans/${activePlanId}`),
    enabled: !!activePlanId,
  });

  const latestVersion = useMemo(() => {
    const versions = detailQuery.data?.versions ?? [];
    return versions.find((v) => v.is_latest) ?? versions[versions.length - 1];
  }, [detailQuery.data]);

  const viewingVersionId = selectedVersionId ?? latestVersion?.id ?? null;
  const viewingFrozen = detailQuery.data?.versions.find((v) => v.id === viewingVersionId)?.frozen;

  const versionQuery = useQuery({
    queryKey: ["plan-version", activePlanId, viewingVersionId],
    queryFn: () =>
      apiFetch<{ adjustments: PlanAdjustment[]; frozen: boolean; is_latest: boolean }>(
        `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}`,
      ),
    enabled: !!activePlanId && !!viewingVersionId,
  });

  const compareQuery = useQuery({
    queryKey: ["plan-compare", activePlanId],
    queryFn: () => apiFetch<PlanCompare>(`/api/v1/plans/${activePlanId}/compare`),
    enabled: !!activePlanId && tab === "compare",
  });

  const riskQuery = useQuery({
    queryKey: ["plan-risk-score"],
    queryFn: () => apiFetch<PlanRiskScoreResponse>("/api/v1/plans/risk-score"),
    retry: false,
  });

  const riskData = riskQuery.data;
  const riskScore: Extract<PlanRiskScoreResponse, { status: "ok" }> | undefined =
    riskData?.status === "ok" ? riskData : undefined;

  const riskBadgeColor = (score: number) => {
    if (score <= 29) return "#16a34a";
    if (score <= 59) return "#d97706";
    return "#dc2626";
  };

  const pvaQuery = useQuery({
    queryKey: ["plan-vs-actual", month],
    queryFn: () => {
      const qs = month ? `?month=${month}` : "";
      return apiFetch<PlanVsActual>(`/api/v1/plans/active/plan-vs-actual${qs}`);
    },
    enabled: tab === "plan-vs-actual",
  });

  const savingsQuery = useQuery({
    queryKey: ["savings-suggestions"],
    queryFn: () =>
      apiFetch<SavingsSuggestion[]>("/api/v1/plans/templates/savings-mode/suggestions"),
    enabled: savingsOpen,
  });

  const createPlanMutation = useMutation({
    mutationFn: (payload: { name: string; template?: string }) =>
      apiFetch<PlanDetail>("/api/v1/plans", {
        method: "POST",
        body: JSON.stringify(payload),
      }),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      setSelectedPlanId(data.plan.id);
      setSelectedVersionId(null);
    },
  });

  const activateMutation = useMutation({
    mutationFn: (planId: string) =>
      apiFetch<void>(`/api/v1/plans/${planId}/activate`, { method: "POST" }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["plans"] }),
  });

  const applyTemplateMutation = useMutation({
    mutationFn: ({
      versionId,
      template,
      subscription_payee_keys,
      discretionary_cut,
    }: {
      versionId: string;
      template: string;
      subscription_payee_keys?: string[];
      discretionary_cut?: boolean;
    }) =>
      apiFetch<void>(`/api/v1/plans/${activePlanId}/versions/${versionId}/apply-template`, {
        method: "POST",
        body: JSON.stringify({ template, subscription_payee_keys, discretionary_cut }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      setSavingsOpen(false);
    },
  });

  const createVersionMutation = useMutation({
    mutationFn: () =>
      apiFetch<{ id: string }>(`/api/v1/plans/${activePlanId}/versions`, { method: "POST" }),
    onSuccess: (v) => {
      queryClient.invalidateQueries({ queryKey: ["plan-detail"] });
      setSelectedVersionId(v.id);
    },
  });

  const deleteAdjustmentMutation = useMutation({
    mutationFn: (adjustmentId: string) =>
      apiFetch<void>(
        `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}/adjustments/${adjustmentId}`,
        { method: "DELETE" },
      ),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["plan-version"] }),
  });

  const empty = !plansQuery.isLoading && (plansQuery.data?.length ?? 0) === 0;
  const planStale = plansQuery.data?.find((p) => p.id === activePlanId)?.plan_stale;

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1>Planning</h1>
        <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
          {riskScore !== undefined && (
            <span
              className="badge"
              title={`Balance stress ${riskScore.components.balance_stress.toFixed(0)} · Plan viability ${riskScore.components.plan_viability.toFixed(0)} · Crypto vol ${riskScore.components.crypto_volatility.toFixed(0)} · ML divergence ${riskScore.components.ml_divergence_modifier.toFixed(0)}`}
              style={{
                background: riskBadgeColor(riskScore.score),
                color: "#fff",
                fontWeight: 600,
              }}
            >
              Risk {riskScore.score} ({riskScore.band})
            </span>
          )}
          {(planStale || pvaQuery.data?.plan_stale || pvaQuery.data?.actuals_stale) && (
            <>
              {planStale && <span className="badge stale">Plan stale</span>}
              {pvaQuery.data?.actuals_stale && <span className="badge stale">Actuals stale</span>}
            </>
          )}
        </div>
      </div>

      {empty ? (
        <div className="card">
          <p>No plans yet. Create your first scenario from a template.</p>
          <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
            <input
              placeholder="Plan name"
              value={newPlanName}
              onChange={(e) => setNewPlanName(e.target.value)}
            />
            <button
              className="btn primary"
              disabled={!newPlanName.trim()}
              onClick={() =>
                createPlanMutation.mutate({ name: newPlanName.trim(), template: "leasing" })
              }
            >
              Create from Leasing template
            </button>
          </div>
        </div>
      ) : (
        <>
          <div className="card" style={{ marginBottom: "1rem" }}>
            <label>
              Active plan{" "}
              <select
                value={activePlanId ?? ""}
                onChange={(e) => {
                  setSelectedPlanId(e.target.value);
                  setSelectedVersionId(null);
                }}
              >
                {(plansQuery.data ?? []).map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                    {p.is_active ? " (active)" : ""}
                  </option>
                ))}
              </select>
            </label>
            {activePlanId && (
              <button
                className="btn"
                style={{ marginLeft: "1rem" }}
                onClick={() => activateMutation.mutate(activePlanId)}
              >
                Set active
              </button>
            )}
          </div>

          <div className="tabs" style={{ marginBottom: "1rem" }}>
            {(["scenarios", "compare", "plan-vs-actual"] as Tab[]).map((t) => (
              <button
                key={t}
                className={`btn ${tab === t ? "primary" : ""}`}
                onClick={() => setTab(t)}
              >
                {t === "plan-vs-actual" ? "Plan vs Actual" : t[0].toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>

          {tab === "scenarios" && (
            <div>
              <div className="card-grid" style={{ marginBottom: "1rem" }}>
                {TEMPLATES.map((t) => (
                  <div key={t.id} className="card">
                    <strong>{t.label}</strong>
                    <p style={{ fontSize: "0.9rem", color: "#64748b" }}>{t.desc}</p>
                    <button
                      className="btn"
                      disabled={!latestVersion || viewingFrozen}
                      onClick={() => {
                        if (t.id === "savings_mode" && latestVersion) {
                          setSavingsTemplateVersionId(latestVersion.id);
                          setSavingsOpen(true);
                          return;
                        }
                        if (latestVersion) {
                          applyTemplateMutation.mutate({
                            versionId: latestVersion.id,
                            template: t.id,
                          });
                        }
                      }}
                    >
                      Apply
                    </button>
                  </div>
                ))}
              </div>

              {viewingFrozen && (
                <div className="card" style={{ marginBottom: "1rem" }}>
                  Viewing frozen version.{" "}
                  <button className="btn primary" onClick={() => createVersionMutation.mutate()}>
                    Create new version
                  </button>
                </div>
              )}

              <div className="card">
                <h3>Adjustments {viewingFrozen ? "(read-only)" : ""}</h3>
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Label</th>
                      <th>Amount</th>
                      <th>Frequency</th>
                      <th>Target</th>
                      <th />
                    </tr>
                  </thead>
                  <tbody>
                    {(versionQuery.data?.adjustments ?? []).map((a) => (
                      <tr key={a.id}>
                        <td>{a.label ?? "—"}</td>
                        <td>{a.amount}</td>
                        <td>{a.frequency}</td>
                        <td>{a.target_type}</td>
                        <td>
                          {!viewingFrozen && (
                            <button
                              className="btn"
                              onClick={() => deleteAdjustmentMutation.mutate(a.id)}
                            >
                              Delete
                            </button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {tab === "compare" && compareQuery.data && (
            <div>
              {compareQuery.data.at_version_cap && (
                <div className="card" style={{ marginBottom: "1rem" }}>
                  Version cap reached (v3). Archive this plan or create a new named plan for v4.
                </div>
              )}
              <div className="card" style={{ marginBottom: "1rem" }}>
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Version</th>
                      <th>Monthly delta sum</th>
                      <th>Projected month-end balance</th>
                      <th>Risk</th>
                    </tr>
                  </thead>
                  <tbody>
                    {compareQuery.data.versions.map((v) => (
                      <tr key={v.version_id}>
                        <td>v{v.version_number}</td>
                        <td>{v.monthly_delta_sum}</td>
                        <td>{v.projected_month_end_balance}</td>
                        <td>
                          {v.version_number === compareQuery.data.versions[compareQuery.data.versions.length - 1]?.version_number && riskScore
                            ? `${riskScore.score} (${riskScore.band})`
                            : "—"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <Suspense fallback={<div>Loading chart…</div>}>
                <CompareChart data={compareQuery.data} />
              </Suspense>
            </div>
          )}

          {tab === "plan-vs-actual" && (
            <div>
              <div className="card" style={{ marginBottom: "1rem" }}>
                <label>
                  Month (YYYY-MM, optional){" "}
                  <input
                    placeholder="2026-05"
                    value={month}
                    onChange={(e) => setMonth(e.target.value)}
                  />
                </label>
                <p style={{ fontSize: "0.85rem", color: "#64748b", marginTop: "0.5rem" }}>
                  Planned series extends beyond today; Ist (actual) stops at last sync date.
                </p>
              </div>
              {pvaQuery.data && (
                <>
                  <Suspense fallback={<div>Loading chart…</div>}>
                    <PlanVsActualChart rows={pvaQuery.data.rows} />
                  </Suspense>
                  <div className="card" style={{ marginTop: "1rem" }}>
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>Planned</th>
                          <th>Ist (actual)</th>
                          <th>Deviation</th>
                        </tr>
                      </thead>
                      <tbody>
                        {pvaQuery.data.rows.map((r) => (
                          <tr key={r.date}>
                            <td>{r.date}</td>
                            <td>{r.planned ?? "—"}</td>
                            <td>{r.actual ?? "—"}</td>
                            <td>{r.deviation ?? "—"}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </>
              )}
            </div>
          )}
        </>
      )}

      {savingsOpen && (
        <div className="modal-backdrop">
          <div className="card modal">
            <h3>Savings mode — select subscriptions to remove</h3>
            <ul style={{ listStyle: "none", padding: 0 }}>
              {(savingsQuery.data ?? []).map((s) => (
                <li key={s.pattern_id} style={{ marginBottom: "0.5rem" }}>
                  <label>
                    <input
                      type="checkbox"
                      checked={selectedPayees.includes(s.payee_key)}
                      onChange={(e) => {
                        setSelectedPayees((prev) =>
                          e.target.checked
                            ? [...prev, s.payee_key]
                            : prev.filter((p) => p !== s.payee_key),
                        );
                      }}
                    />{" "}
                    {s.display_name} ({s.current_amount}/ {s.interval_days}d)
                  </label>
                </li>
              ))}
            </ul>
            <label>
              <input
                type="checkbox"
                checked={discretionaryCut}
                onChange={(e) => setDiscretionaryCut(e.target.checked)}
              />{" "}
              Apply −€100/month discretionary cut
            </label>
            <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
              <button
                className="btn primary"
                onClick={() => {
                  if (savingsTemplateVersionId) {
                    applyTemplateMutation.mutate({
                      versionId: savingsTemplateVersionId,
                      template: "savings_mode",
                      subscription_payee_keys: selectedPayees,
                      discretionary_cut: discretionaryCut,
                    });
                  }
                }}
              >
                Apply savings mode
              </button>
              <button className="btn" onClick={() => setSavingsOpen(false)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
