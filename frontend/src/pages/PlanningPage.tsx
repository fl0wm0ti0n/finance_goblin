import { lazy, Suspense, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  apiFetch,
  CategorySavingsSuggestion,
  fetchCategorySavingsSuggestions,
  fetchGoalStats,
  PlanAdjustment,
  PlanCompare,
  PlanDetail,
  PlanListItem,
  PlanRiskScoreResponse,
  PlanVsActual,
  PlanVsActualRow,
  SavingsSuggestion,
} from "../lib/api";
import {
  formatPlanningError,
  PlanningFeedbackCard,
  usePlanningFeedback,
} from "./planningFeedback";

const CompareChart = lazy(() =>
  import("../components/planning/CompareChart").then((m) => ({ default: m.CompareChart })),
);
const PlanVsActualChart = lazy(() =>
  import("../components/planning/PlanVsActualChart").then((m) => ({
    default: m.PlanVsActualChart,
  })),
);
const CategoryFilter = lazy(() =>
  import("../components/category/CategoryFilter").then((m) => ({ default: m.CategoryFilter })),
);
const CategoryTrendChart = lazy(() =>
  import("../components/category/CategoryTrendChart").then((m) => ({
    default: m.CategoryTrendChart,
  })),
);
const GoalStatsStrip = lazy(() =>
  import("../components/plan/GoalStatsStrip").then((m) => ({ default: m.GoalStatsStrip })),
);

type Tab = "scenarios" | "compare" | "plan-vs-actual";

const TEMPLATES = [
  { id: "current", label: "Current (Ist)", desc: "Baseline forecast only" },
  { id: "leasing", label: "Leasing", desc: "+€300/month household outflow" },
  { id: "savings_mode", label: "Savings mode", desc: "Remove subscriptions + optional cut" },
  { id: "house_purchase", label: "House purchase", desc: "+€500/month savings transfer" },
  { id: "allocation_target", label: "Allocation target", desc: "50/50 ETF vs crypto weights" },
  {
    id: "goal_balance",
    label: "Goal balance",
    desc: "Target balance by date (per-plan stats)",
  },
  { id: "custom", label: "Custom", desc: "Start empty and add lines" },
];

const todayIso = () => new Date().toISOString().slice(0, 10);

type AdjustmentFormState = {
  direction: string;
  amount: string;
  frequency: string;
  target_type: string;
  label: string;
  effective_from: string;
};

const defaultAdjustmentForm = (): AdjustmentFormState => ({
  direction: "add_outflow",
  amount: "100",
  frequency: "monthly",
  target_type: "household",
  label: "",
  effective_from: todayIso(),
});

export function PlanningPage() {
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab>("scenarios");
  const [compareCategoryId, setCompareCategoryId] = useState("");
  const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null);
  const [newPlanName, setNewPlanName] = useState("");
  const [savingsOpen, setSavingsOpen] = useState(false);
  const [savingsTemplateVersionId, setSavingsTemplateVersionId] = useState<string | null>(null);
  const [selectedPayees, setSelectedPayees] = useState<string[]>([]);
  const [discretionaryCut, setDiscretionaryCut] = useState(false);
  const [month, setMonth] = useState<string>("");
  const { feedback, showPlanningFeedback, dismissFeedback } = usePlanningFeedback();
  const [showSetActiveBanner, setShowSetActiveBanner] = useState(false);
  const [addForm, setAddForm] = useState<AdjustmentFormState>(defaultAdjustmentForm);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<AdjustmentFormState>(defaultAdjustmentForm);
  const [deleteConfirmPlan, setDeleteConfirmPlan] = useState<PlanListItem | null>(null);
  const [goalTargetBalance, setGoalTargetBalance] = useState("10000");
  const [goalTargetDate, setGoalTargetDate] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() + 5);
    return d.toISOString().slice(0, 10);
  });
  const [goalAccountId, setGoalAccountId] = useState("");
  const [categorySavingsOpen, setCategorySavingsOpen] = useState(false);
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);

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
    retry: false,
  });

  const pvaData = pvaQuery.data?.status === "ok" ? pvaQuery.data : undefined;

  const savingsQuery = useQuery({
    queryKey: ["savings-suggestions"],
    queryFn: () =>
      apiFetch<SavingsSuggestion[]>("/api/v1/plans/templates/savings-mode/suggestions"),
    enabled: savingsOpen,
  });

  const templateLabel = (id: string) => TEMPLATES.find((t) => t.id === id)?.label ?? id;

  const activePlanTemplate = detailQuery.data?.plan.template;
  const isGoalPlan = activePlanTemplate === "goal_balance";

  const goalStatsQuery = useQuery({
    queryKey: ["goal-stats", activePlanId, viewingVersionId],
    queryFn: () => fetchGoalStats(activePlanId!, viewingVersionId ?? undefined),
    enabled: !!activePlanId && isGoalPlan,
    retry: false,
  });

  const categorySavingsQuery = useQuery({
    queryKey: ["category-savings", activePlanId],
    queryFn: () => fetchCategorySavingsSuggestions(activePlanId!),
    enabled: categorySavingsOpen && !!activePlanId && isGoalPlan,
  });

  const createPlanMutation = useMutation({
    mutationFn: (payload: {
      name: string;
      template?: string;
      target_balance_eur?: string;
      target_date?: string;
      goal_account_id?: string;
    }) =>
      apiFetch<PlanDetail>("/api/v1/plans", {
        method: "POST",
        body: JSON.stringify({
          ...payload,
          goal_account_id: payload.goal_account_id || undefined,
        }),
      }),
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      setSelectedPlanId(data.plan.id);
      setSelectedVersionId(null);
      if (!data.plan.is_active) {
        setShowSetActiveBanner(true);
      }
      setTab("scenarios");
      if (variables.template && variables.template !== "custom") {
        showPlanningFeedback({
          kind: "success",
          message: `Plan "${variables.name}" created from ${templateLabel(variables.template)}`,
        });
      } else {
        showPlanningFeedback({
          kind: "success",
          message: `Plan "${variables.name}" created`,
        });
      }
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not create plan"),
      }),
  });

  const activateMutation = useMutation({
    mutationFn: (planId: string) =>
      apiFetch<void>(`/api/v1/plans/${planId}/activate`, { method: "POST" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      setShowSetActiveBanner(false);
      showPlanningFeedback({ kind: "success", message: "Plan set as active" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not set active plan"),
      }),
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
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-detail"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      setSavingsOpen(false);
      if (variables.template === "custom") {
        showPlanningFeedback({
          kind: "success",
          message: "Custom plan ready — add lines below",
        });
      } else {
        showPlanningFeedback({
          kind: "success",
          message: `Template applied (${templateLabel(variables.template)})`,
        });
      }
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not apply template"),
      }),
  });

  const createVersionMutation = useMutation({
    mutationFn: () =>
      apiFetch<{ id: string }>(`/api/v1/plans/${activePlanId}/versions`, { method: "POST" }),
    onSuccess: (v) => {
      queryClient.invalidateQueries({ queryKey: ["plan-detail"] });
      setSelectedVersionId(v.id);
      showPlanningFeedback({ kind: "success", message: "New version created" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not create version"),
      }),
  });

  const addAdjustmentMutation = useMutation({
    mutationFn: (payload: AdjustmentFormState) =>
      apiFetch<PlanAdjustment>(
        `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}/adjustments`,
        {
          method: "POST",
          body: JSON.stringify({
            direction: payload.direction,
            amount: Number(payload.amount),
            frequency: payload.frequency,
            target_type: payload.target_type,
            label: payload.label || null,
            effective_from: payload.effective_from,
          }),
        },
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      setAddForm(defaultAdjustmentForm());
      showPlanningFeedback({ kind: "success", message: "Adjustment added" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not add adjustment"),
      }),
  });

  const updateAdjustmentMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: AdjustmentFormState }) =>
      apiFetch<PlanAdjustment>(
        `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}/adjustments/${id}`,
        {
          method: "PATCH",
          body: JSON.stringify({
            direction: payload.direction,
            amount: Number(payload.amount),
            frequency: payload.frequency,
            target_type: payload.target_type,
            label: payload.label || null,
            effective_from: payload.effective_from,
          }),
        },
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      setEditingId(null);
      showPlanningFeedback({ kind: "success", message: "Adjustment updated" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not update adjustment"),
      }),
  });

  const deletePlanMutation = useMutation({
    mutationFn: (planId: string) =>
      apiFetch<void>(`/api/v1/plans/${planId}`, { method: "DELETE" }),
    onSuccess: (_data, planId) => {
      queryClient.invalidateQueries({ queryKey: ["plans"] });
      queryClient.invalidateQueries({ queryKey: ["plan-detail"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      if (selectedPlanId === planId) {
        setSelectedPlanId(null);
        setSelectedVersionId(null);
      }
      setDeleteConfirmPlan(null);
      showPlanningFeedback({ kind: "success", message: "Plan deleted" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not delete plan"),
      }),
  });

  const applyCategorySavingsMutation = useMutation({
    mutationFn: async (suggestions: CategorySavingsSuggestion[]) => {
      if (!activePlanId || !viewingVersionId) {
        throw new Error("No plan version selected");
      }
      for (const s of suggestions) {
        await apiFetch<PlanAdjustment>(
          `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}/adjustments`,
          {
            method: "POST",
            body: JSON.stringify({
              direction: "remove_outflow",
              amount: Number(s.suggested_reduction_eur),
              frequency: "monthly",
              target_type: "category",
              target_key: s.category_id,
              label: `Reduce ${s.category_name}`,
              effective_from: todayIso(),
            }),
          },
        );
      }
    },
    onSuccess: (_data, suggestions) => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      queryClient.invalidateQueries({ queryKey: ["goal-stats"] });
      queryClient.invalidateQueries({ queryKey: ["category-savings"] });
      setCategorySavingsOpen(false);
      setSelectedCategories([]);
      showPlanningFeedback({
        kind: "success",
        message: `Applied ${suggestions.length} category savings line(s) — recompute pending`,
      });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not apply category savings"),
      }),
  });

  const deleteAdjustmentMutation = useMutation({
    mutationFn: (adjustmentId: string) =>
      apiFetch<void>(
        `/api/v1/plans/${activePlanId}/versions/${viewingVersionId}/adjustments/${adjustmentId}`,
        { method: "DELETE" },
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["plan-version"] });
      queryClient.invalidateQueries({ queryKey: ["plan-compare"] });
      queryClient.invalidateQueries({ queryKey: ["plan-vs-actual"] });
      showPlanningFeedback({ kind: "success", message: "Adjustment removed" });
    },
    onError: (err) =>
      showPlanningFeedback({
        kind: "error",
        message: formatPlanningError(err, "Could not delete adjustment"),
      }),
  });

  const createGoalPlan = (name: string) =>
    createPlanMutation.mutate({
      name,
      template: "goal_balance",
      target_balance_eur: goalTargetBalance,
      target_date: goalTargetDate,
      goal_account_id: goalAccountId.trim() || undefined,
    });

  const renderGoalFields = () => (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem", marginTop: "0.5rem" }}>
      <input
        type="number"
        step="0.01"
        placeholder="Target €"
        value={goalTargetBalance}
        onChange={(e) => setGoalTargetBalance(e.target.value)}
        style={{ width: "7rem" }}
      />
      <input
        type="date"
        value={goalTargetDate}
        onChange={(e) => setGoalTargetDate(e.target.value)}
      />
      <input
        placeholder="Goal account id (optional)"
        value={goalAccountId}
        onChange={(e) => setGoalAccountId(e.target.value)}
        style={{ minWidth: "12rem" }}
      />
    </div>
  );

  const empty = !plansQuery.isLoading && (plansQuery.data?.length ?? 0) === 0;
  const planStale = plansQuery.data?.find((p) => p.id === activePlanId)?.plan_stale;
  const activePlanIsSelected = plansQuery.data?.find((p) => p.id === activePlanId)?.is_active;

  const startEdit = (a: PlanAdjustment) => {
    setEditingId(a.id);
    setEditForm({
      direction: a.direction,
      amount: a.amount,
      frequency: a.frequency,
      target_type: a.target_type,
      label: a.label ?? "",
      effective_from: a.effective_from,
    });
  };

  const renderAdjustmentFields = (
    form: AdjustmentFormState,
    onChange: (next: AdjustmentFormState) => void,
  ) => (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem", alignItems: "center" }}>
      <select value={form.direction} onChange={(e) => onChange({ ...form, direction: e.target.value })}>
        <option value="add_outflow">Add outflow</option>
        <option value="remove_outflow">Remove outflow</option>
        <option value="add_inflow">Add inflow</option>
        <option value="remove_inflow">Remove inflow</option>
      </select>
      <input
        type="number"
        step="0.01"
        placeholder="Amount"
        value={form.amount}
        onChange={(e) => onChange({ ...form, amount: e.target.value })}
        style={{ width: "6rem" }}
      />
      <select value={form.frequency} onChange={(e) => onChange({ ...form, frequency: e.target.value })}>
        <option value="monthly">Monthly</option>
        <option value="weekly">Weekly</option>
        <option value="quarterly">Quarterly</option>
        <option value="one_time">One-time</option>
      </select>
      <select
        value={form.target_type}
        onChange={(e) => onChange({ ...form, target_type: e.target.value })}
      >
        <option value="household">Household</option>
        <option value="subscription">Subscription</option>
        <option value="category">Category</option>
        <option value="custom_label">Custom label</option>
        <option value="allocation_target">Allocation target</option>
      </select>
      <input
        placeholder="Label"
        value={form.label}
        onChange={(e) => onChange({ ...form, label: e.target.value })}
      />
      <input
        type="date"
        value={form.effective_from}
        onChange={(e) => onChange({ ...form, effective_from: e.target.value })}
      />
      <p style={{ flexBasis: "100%", fontSize: "0.85rem", color: "#64748b", margin: "0.25rem 0 0" }}>
        Household applies across all linked accounts; Subscription matches recurring payees;
        Category scopes to a budget category; Custom label and Allocation target are advanced
        options templates may pre-fill.
      </p>
    </div>
  );

  return (
    <div>
      <PlanningFeedbackCard feedback={feedback} onDismiss={dismissFeedback} />

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
          {(planStale ||
            (pvaData?.status === "ok" && (pvaData.plan_stale || pvaData.actuals_stale))) && (
            <>
              {planStale && <span className="badge stale">Plan stale</span>}
              {pvaData?.status === "ok" && pvaData.actuals_stale && (
                <span className="badge stale">Actuals stale</span>
              )}
            </>
          )}
        </div>
      </div>

      {empty ? (
        <div>
          <div className="card" style={{ marginBottom: "1rem" }}>
            <p>No plans yet. Create your first scenario from a template or start empty.</p>
            <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem", flexWrap: "wrap" }}>
              <input
                placeholder="Plan name"
                value={newPlanName}
                onChange={(e) => setNewPlanName(e.target.value)}
              />
              <button
                className="btn primary"
                disabled={!newPlanName.trim() || createPlanMutation.isPending}
                onClick={() =>
                  createPlanMutation.mutate({ name: newPlanName.trim(), template: "custom" })
                }
              >
                Create empty plan
              </button>
            </div>
          </div>
          <div className="card-grid">
            {TEMPLATES.map((t) => (
              <div key={t.id} className="card">
                <strong>{t.label}</strong>
                <p style={{ fontSize: "0.9rem", color: "#64748b" }}>{t.desc}</p>
                {t.id === "goal_balance" && renderGoalFields()}
                <button
                  className="btn"
                  disabled={!newPlanName.trim() || createPlanMutation.isPending}
                  onClick={() =>
                    t.id === "goal_balance"
                      ? createGoalPlan(newPlanName.trim())
                      : createPlanMutation.mutate({ name: newPlanName.trim(), template: t.id })
                  }
                >
                  Create from {t.label}
                </button>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <>
          {showSetActiveBanner && !activePlanIsSelected && activePlanId && (
            <div className="card" style={{ marginBottom: "1rem", background: "#fffbeb" }}>
              Plan created. Click <strong>Set active</strong> below so Plan vs Actual and Grafana
              Dashboard 3 (<strong>Budgets</strong>) use this scenario.
            </div>
          )}

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
              <>
                <button
                  className="btn"
                  style={{ marginLeft: "1rem" }}
                  onClick={() => activateMutation.mutate(activePlanId)}
                >
                  Set active
                </button>
                <button
                  className="btn"
                  style={{ marginLeft: "0.5rem" }}
                  disabled={activePlanIsSelected}
                  title={
                    activePlanIsSelected
                      ? "Set another plan active before deleting the active plan"
                      : "Delete this plan"
                  }
                  onClick={() => {
                    const plan = plansQuery.data?.find((p) => p.id === activePlanId);
                    if (plan) {
                      setDeleteConfirmPlan(plan);
                    }
                  }}
                >
                  Delete plan
                </button>
              </>
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
                    {t.id === "goal_balance" && renderGoalFields()}
                    <button
                      className="btn"
                      disabled={
                        t.id === "goal_balance"
                          ? !newPlanName.trim() || createPlanMutation.isPending
                          : !latestVersion || viewingFrozen
                      }
                      onClick={() => {
                        if (t.id === "goal_balance") {
                          createGoalPlan(newPlanName.trim() || `Goal ${goalTargetDate}`);
                          return;
                        }
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
                      {t.id === "goal_balance" ? "Create goal plan" : "Apply"}
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

              {isGoalPlan && goalStatsQuery.data && (
                <Suspense fallback={<p>Loading goal stats…</p>}>
                  <GoalStatsStrip stats={goalStatsQuery.data} />
                </Suspense>
              )}

              {isGoalPlan && !viewingFrozen && (
                <div className="card" style={{ marginBottom: "1rem" }}>
                  <button className="btn" onClick={() => setCategorySavingsOpen(true)}>
                    Category savings suggestions
                  </button>
                  <p style={{ fontSize: "0.85rem", color: "#64748b", margin: "0.5rem 0 0" }}>
                    Ranked mirror aggregates only — select lines to apply; no auto-apply.
                  </p>
                </div>
              )}

              <div className="card">
                <h3>Adjustments {viewingFrozen ? "(read-only)" : ""}</h3>
                {!viewingFrozen && viewingVersionId && (
                  <div style={{ marginBottom: "1rem" }}>
                    <h4 style={{ marginTop: 0 }}>Add adjustment</h4>
                    {renderAdjustmentFields(addForm, setAddForm)}
                    <button
                      className="btn primary"
                      style={{ marginTop: "0.5rem" }}
                      disabled={!addForm.amount || addAdjustmentMutation.isPending}
                      onClick={() => addAdjustmentMutation.mutate(addForm)}
                    >
                      Add
                    </button>
                  </div>
                )}
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
                        {editingId === a.id && !viewingFrozen ? (
                          <>
                            <td colSpan={4}>
                              {renderAdjustmentFields(editForm, setEditForm)}
                            </td>
                            <td>
                              <button
                                className="btn primary"
                                onClick={() =>
                                  updateAdjustmentMutation.mutate({ id: a.id, payload: editForm })
                                }
                              >
                                Save
                              </button>{" "}
                              <button className="btn" onClick={() => setEditingId(null)}>
                                Cancel
                              </button>
                            </td>
                          </>
                        ) : (
                          <>
                            <td>{a.label ?? "—"}</td>
                            <td>{a.amount}</td>
                            <td>{a.frequency}</td>
                            <td>{a.target_type}</td>
                            <td>
                              {!viewingFrozen && (
                                <>
                                  <button className="btn" onClick={() => startEdit(a)}>
                                    Edit
                                  </button>{" "}
                                  <button
                                    className="btn"
                                    onClick={() => deleteAdjustmentMutation.mutate(a.id)}
                                  >
                                    Delete
                                  </button>
                                </>
                              )}
                            </td>
                          </>
                        )}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {tab === "compare" && compareQuery.data && (
            <div>
              {isGoalPlan && goalStatsQuery.data && (
                <Suspense fallback={<p>Loading goal stats…</p>}>
                  <GoalStatsStrip stats={goalStatsQuery.data} />
                </Suspense>
              )}
              <div className="card" style={{ marginBottom: "1rem" }}>
                <Suspense fallback={<p>Loading category filter…</p>}>
                  <CategoryFilter
                    value={compareCategoryId}
                    onChange={setCompareCategoryId}
                    label="Category (actual spending preview)"
                  />
                </Suspense>
                <p style={{ fontSize: "0.85rem", color: "#64748b", margin: "0.5rem 0 0" }}>
                  Actual spending trend only — plan compare metrics and version table below are
                  household-level and unaffected by this filter.
                </p>
              </div>
              {compareCategoryId && (
                <div style={{ marginBottom: "1rem" }}>
                  <Suspense fallback={<p>Loading actuals trend…</p>}>
                    <CategoryTrendChart
                      categoryId={compareCategoryId}
                      title="Actual spending trend"
                    />
                  </Suspense>
                </div>
              )}
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
                          {v.version_number ===
                            compareQuery.data.versions[compareQuery.data.versions.length - 1]
                              ?.version_number && riskScore
                            ? `${riskScore.score} (${riskScore.band})`
                            : "—"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <p style={{ fontSize: "0.85rem", color: "#64748b", marginTop: "0.75rem" }}>
                  Monthly delta = scenario adjustments only; projected month-end balance includes
                  the baseline household forecast and may be negative even when delta is zero.
                </p>
              </div>
              <Suspense fallback={<div>Loading chart…</div>}>
                <CompareChart data={compareQuery.data} />
              </Suspense>
            </div>
          )}

          {tab === "plan-vs-actual" && (
            <div>
              {pvaQuery.data?.status === "no_active_plan" ? (
                <div className="card">
                  <h3>No active plan</h3>
                  <p>
                    Plan vs Actual needs an active plan. Create a scenario on the Scenarios tab,
                    then click <strong>Set active</strong> above.
                  </p>
                  <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem" }}>
                    <button className="btn primary" onClick={() => setTab("scenarios")}>
                      Go to Scenarios
                    </button>
                    {activePlanId && (
                      <button
                        className="btn"
                        onClick={() => activateMutation.mutate(activePlanId)}
                      >
                        Set active now
                      </button>
                    )}
                  </div>
                </div>
              ) : (
                <>
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
                  {pvaData?.status === "ok" && (
                    <>
                      <Suspense fallback={<div>Loading chart…</div>}>
                        <PlanVsActualChart rows={pvaData.rows} />
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
                            {pvaData.rows.map((r: PlanVsActualRow) => (
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
                </>
              )}
            </div>
          )}
        </>
      )}

      {deleteConfirmPlan && (
        <div className="modal-backdrop">
          <div className="card modal">
            <h3>Delete plan?</h3>
            <p>
              Permanently delete <strong>{deleteConfirmPlan.name}</strong> and all versions and
              adjustments? This cannot be undone.
            </p>
            <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
              <button
                className="btn primary"
                disabled={deletePlanMutation.isPending}
                onClick={() => deletePlanMutation.mutate(deleteConfirmPlan.id)}
              >
                Delete
              </button>
              <button className="btn" onClick={() => setDeleteConfirmPlan(null)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {categorySavingsOpen && (
        <div className="modal-backdrop">
          <div className="card modal">
            <h3>Category savings — select reductions to apply</h3>
            <ul style={{ listStyle: "none", padding: 0 }}>
              {(categorySavingsQuery.data?.suggestions ?? []).map((s) => (
                <li key={s.category_id} style={{ marginBottom: "0.75rem" }}>
                  <label>
                    <input
                      type="checkbox"
                      checked={selectedCategories.includes(s.category_id)}
                      onChange={(e) => {
                        setSelectedCategories((prev) =>
                          e.target.checked
                            ? [...prev, s.category_id]
                            : prev.filter((id) => id !== s.category_id),
                        );
                      }}
                    />{" "}
                    <strong>{s.category_name}</strong> — reduce €{s.suggested_reduction_eur}/mo
                    <div style={{ fontSize: "0.85rem", color: "#64748b" }}>{s.evidence_summary}</div>
                  </label>
                </li>
              ))}
            </ul>
            <div style={{ marginTop: "1rem", display: "flex", gap: "0.5rem" }}>
              <button
                className="btn primary"
                disabled={
                  selectedCategories.length === 0 || applyCategorySavingsMutation.isPending
                }
                onClick={() => {
                  const picked =
                    categorySavingsQuery.data?.suggestions.filter((s) =>
                      selectedCategories.includes(s.category_id),
                    ) ?? [];
                  applyCategorySavingsMutation.mutate(picked);
                }}
              >
                Apply selected
              </button>
              <button className="btn" onClick={() => setCategorySavingsOpen(false)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
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
