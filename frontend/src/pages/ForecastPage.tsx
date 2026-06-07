import { lazy, Suspense, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  apiFetch,
  ForecastAccount,
  ForecastCompare,
  ForecastDaily,
  ForecastLongTerm,
  ForecastMeta,
  ForecastMonthly,
} from "../lib/api";

const DailyChart = lazy(() =>
  import("../components/forecast/DailyChart").then((m) => ({ default: m.DailyChart })),
);
const MonthlyChart = lazy(() =>
  import("../components/forecast/MonthlyChart").then((m) => ({ default: m.MonthlyChart })),
);
const LongTermChart = lazy(() =>
  import("../components/forecast/LongTermChart").then((m) => ({ default: m.LongTermChart })),
);

type Tab = "daily" | "monthly" | "long-term";
type LongTermMode = "baseline" | "ml_enhanced" | "compare";

export function ForecastPage() {
  const [tab, setTab] = useState<Tab>("daily");
  const [accountId, setAccountId] = useState<string>("");
  const [horizon, setHorizon] = useState<3 | 6 | 12 | 24>(12);
  const [longTermMode, setLongTermMode] = useState<LongTermMode>("baseline");
  const [explainOpen, setExplainOpen] = useState(false);

  const metaQuery = useQuery({
    queryKey: ["forecast-meta"],
    queryFn: () => apiFetch<ForecastMeta>("/api/v1/forecast/meta"),
  });

  const accountsQuery = useQuery({
    queryKey: ["forecast-accounts"],
    queryFn: () => apiFetch<ForecastAccount[]>("/api/v1/forecast/accounts"),
  });

  const selectedAccount =
    accountId || accountsQuery.data?.[0]?.id || "";

  const mlAvailable =
    !!metaQuery.data?.ml_computation_id &&
    metaQuery.data?.ml_status === "success" &&
    !metaQuery.data?.ml_skipped_reason;

  const mlSkippedReason = metaQuery.data?.ml_skipped_reason ?? null;
  const mlNotEnabled = mlSkippedReason === "sidecar_disabled";
  const mlSkippedOther = !!mlSkippedReason && !mlNotEnabled;

  const mlTabTooltip = mlAvailable
    ? undefined
    : mlNotEnabled
      ? "ML forecast is not enabled on this deployment"
      : mlSkippedOther
        ? `ML skipped: ${mlSkippedReason}`
        : "Baseline-only forecast — ML has not run yet";

  const mlExplainCopy = mlAvailable ? null : mlNotEnabled ? (
    <p>
      ML forecast is not enabled on this deployment. Baseline DEC-0007 forecast is authoritative
      for alerts.
    </p>
  ) : mlSkippedOther ? (
    <p>
      ML skipped: {mlSkippedReason}. Baseline DEC-0007 forecast remains authoritative for alerts.
    </p>
  ) : (
    <p>Baseline-only forecast — ML has not run yet.</p>
  );

  const balanceWarnings = metaQuery.data?.balance_warnings ?? [];

  const effectiveHorizon =
    longTermMode !== "baseline" && horizon === 3 ? 6 : horizon;

  const dailyQuery = useQuery({
    queryKey: ["forecast-daily", selectedAccount],
    queryFn: () =>
      apiFetch<ForecastDaily>(`/api/v1/forecast/daily?account_id=${encodeURIComponent(selectedAccount)}`),
    enabled: !!selectedAccount && tab === "daily" && !!metaQuery.data?.computation_id,
  });

  const monthlyQuery = useQuery({
    queryKey: ["forecast-monthly", selectedAccount],
    queryFn: () =>
      apiFetch<ForecastMonthly>(
        `/api/v1/forecast/monthly?account_id=${encodeURIComponent(selectedAccount)}`,
      ),
    enabled: !!selectedAccount && tab === "monthly" && !!metaQuery.data?.computation_id,
  });

  const longTermQuery = useQuery({
    queryKey: ["forecast-long-term", selectedAccount, effectiveHorizon, longTermMode],
    queryFn: () =>
      apiFetch<ForecastLongTerm>(
        `/api/v1/forecast/long-term?account_id=${encodeURIComponent(selectedAccount)}&horizon=${effectiveHorizon}&variant=${longTermMode === "compare" ? "baseline" : longTermMode}`,
      ),
    enabled:
      !!selectedAccount &&
      tab === "long-term" &&
      longTermMode !== "compare" &&
      !!metaQuery.data?.computation_id,
  });

  const mlLongTermQuery = useQuery({
    queryKey: ["forecast-long-term-ml", selectedAccount, effectiveHorizon],
    queryFn: () =>
      apiFetch<ForecastLongTerm>(
        `/api/v1/forecast/long-term?account_id=${encodeURIComponent(selectedAccount)}&horizon=${effectiveHorizon}&variant=ml_enhanced`,
      ),
    enabled:
      !!selectedAccount &&
      tab === "long-term" &&
      longTermMode === "ml_enhanced" &&
      mlAvailable,
  });

  const compareQuery = useQuery({
    queryKey: ["forecast-compare", selectedAccount, effectiveHorizon],
    queryFn: () =>
      apiFetch<ForecastCompare>(
        `/api/v1/forecast/compare?account_id=${encodeURIComponent(selectedAccount)}&horizon=${effectiveHorizon}`,
      ),
    enabled:
      !!selectedAccount &&
      tab === "long-term" &&
      longTermMode === "compare" &&
      mlAvailable,
  });

  const hasForecast = !!metaQuery.data?.computation_id;
  const emptyState = !hasForecast;

  const monthlySummary = useMemo(() => {
    const series = monthlyQuery.data?.series ?? [];
    if (series.length === 0) return null;
    return series[0];
  }, [monthlyQuery.data]);

  const longTermData =
    longTermMode === "ml_enhanced" ? mlLongTermQuery.data : longTermQuery.data;

  const seasonalCallout =
    metaQuery.data?.seasonal_detected &&
    metaQuery.data.seasonal_periods &&
    metaQuery.data.seasonal_periods.length > 0;

  return (
    <div>
      <div className="card" style={{ marginBottom: "1rem" }}>
        <div style={{ display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: "1rem" }}>
          <div>
            <h1>Forecast</h1>
            <p style={{ color: "#64748b", margin: 0 }}>
              Last computed:{" "}
              {metaQuery.data?.computed_at
                ? new Date(metaQuery.data.computed_at).toLocaleString()
                : "Not yet computed — run a sync first."}
              {metaQuery.data?.stale && (
                <span style={{ color: "#b45309", marginLeft: "0.5rem" }}>(stale snapshot)</span>
              )}
              {metaQuery.data?.low_confidence && (
                <span style={{ color: "#b45309", marginLeft: "0.5rem" }}>(low confidence)</span>
              )}
            </p>
            <p style={{ margin: "0.25rem 0 0" }}>
              <Link to="/sync">Sync Status</Link>
            </p>
          </div>
          <div>
            <label htmlFor="account-select" style={{ display: "block", fontSize: "0.85rem", marginBottom: "0.25rem" }}>
              Account
            </label>
            <select
              id="account-select"
              className="select"
              value={selectedAccount}
              onChange={(e) => setAccountId(e.target.value)}
              disabled={!accountsQuery.data?.length}
            >
              {(accountsQuery.data ?? []).map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                  {a.currency ? ` (${a.currency})` : ""}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {balanceWarnings.length > 0 && (
        <div className="card" style={{ marginBottom: "1rem", borderColor: "#b45309" }}>
          <strong>Starting balance warning</strong>
          <p style={{ margin: "0.5rem 0 0" }}>
            Starting balance is zero or negative — verify Firefly account balances or reconcile
            before trusting long-term forecast.
          </p>
        </div>
      )}

      {emptyState ? (
        <div className="card">
          <h2>No forecast data yet</h2>
          <p>
            Sync transactions from Firefly first. Forecasts are recomputed automatically after each
            successful sync.
          </p>
          <Link to="/sync" className="btn" style={{ display: "inline-block", marginTop: "0.5rem" }}>
            Go to Sync Status
          </Link>
        </div>
      ) : (
        <>
          <div className="tabs" style={{ marginBottom: "1rem" }}>
            {(["daily", "monthly", "long-term"] as Tab[]).map((t) => (
              <button
                key={t}
                className={`btn tab ${tab === t ? "active" : ""}`}
                onClick={() => setTab(t)}
              >
                {t === "long-term" ? "Long-term" : t.charAt(0).toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>

          {tab === "daily" && (
            <>
              <div className="grid" style={{ marginBottom: "1rem" }}>
                <div className="card">
                  <div>Tomorrow</div>
                  <div className="stat">{dailyQuery.data?.milestones.tomorrow ?? "—"}</div>
                </div>
                <div className="card">
                  <div>Next week</div>
                  <div className="stat">{dailyQuery.data?.milestones.next_week ?? "—"}</div>
                </div>
                <div className="card">
                  <div>Month end</div>
                  <div className="stat">{dailyQuery.data?.milestones.month_end ?? "—"}</div>
                </div>
              </div>
              <div className="card">
                <Suspense fallback={<p>Loading chart…</p>}>
                  <DailyChart series={dailyQuery.data?.series ?? []} />
                </Suspense>
              </div>
            </>
          )}

          {tab === "monthly" && (
            <>
              {monthlyQuery.data?.series?.[0]?.ai_mapped && (
                <div className="card" style={{ marginBottom: "1rem", borderColor: "#a855f7" }}>
                  <span
                    className="badge"
                    style={{ background: "#f3e8ff", color: "#7e22ce" }}
                    title="Some recurring forecast rows were bucket-mapped by AI after the config category map. Config-mapped categories are never overridden. Inference uses privacy-safe hashed merchant tokens and amount bands (not raw descriptions) unless allow_raw_transactions=true. Rolling discretionary residual stays Variable in MVP."
                  >
                    AI-mapped
                  </span>
                  <p style={{ margin: "0.5rem 0 0" }}>
                    AI assisted bucket mapping for ambiguous recurring patterns this month. Your{" "}
                    <code>[forecast.category_buckets]</code> config map still takes precedence;
                    rolling discretionary spend remains Variable.
                  </p>
                </div>
              )}
              {seasonalCallout && (
                <div className="card" style={{ marginBottom: "1rem", borderColor: "#3b82f6" }}>
                  <span className="badge" style={{ background: "#dbeafe", color: "#1d4ed8" }}>
                    Seasonal pattern detected
                  </span>
                  <p style={{ margin: "0.5rem 0 0" }}>
                    {metaQuery.data?.seasonal_periods?.join("-")}-month cycle detected in ML analysis
                    {metaQuery.data?.seasonal_strength != null &&
                      ` (strength ${(metaQuery.data.seasonal_strength * 100).toFixed(0)}%)`}
                    .
                  </p>
                </div>
              )}
              {monthlySummary && (
                <div className="grid" style={{ marginBottom: "1rem" }}>
                  <div className="card">
                    <div>Income</div>
                    <div className="stat">{monthlySummary.income}</div>
                  </div>
                  <div className="card">
                    <div>Fixed</div>
                    <div className="stat">{monthlySummary.fixed_costs}</div>
                  </div>
                  <div className="card">
                    <div>Variable</div>
                    <div className="stat">{monthlySummary.variable_costs}</div>
                  </div>
                  <div className="card">
                    <div>Free cashflow</div>
                    <div className="stat">{monthlySummary.free_cashflow}</div>
                  </div>
                </div>
              )}
              <div className="card">
                <Suspense fallback={<p>Loading chart…</p>}>
                  <MonthlyChart series={monthlyQuery.data?.series ?? []} />
                </Suspense>
              </div>
            </>
          )}

          {tab === "long-term" && (
            <>
              <div style={{ marginBottom: "1rem", display: "flex", gap: "0.5rem", flexWrap: "wrap", alignItems: "center" }}>
                {(["baseline", "ml_enhanced", "compare"] as LongTermMode[]).map((mode) => {
                  const disabled = mode !== "baseline" && !mlAvailable;
                  return (
                    <button
                      key={mode}
                      className={`btn tab ${longTermMode === mode ? "active" : ""}`}
                      disabled={disabled}
                      title={disabled ? mlTabTooltip : undefined}
                      onClick={() => setLongTermMode(mode)}
                    >
                      {mode === "baseline" ? "Baseline" : mode === "ml_enhanced" ? "ML-enhanced" : "Compare"}
                    </button>
                  );
                })}
              </div>

              <div style={{ marginBottom: "1rem", display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
                {([3, 6, 12, 24] as const).map((h) => {
                  const disabled = longTermMode !== "baseline" && h === 3;
                  return (
                    <button
                      key={h}
                      className={`btn tab ${horizon === h ? "active" : ""}`}
                      disabled={disabled}
                      title={disabled ? "3 mo available in baseline mode only" : undefined}
                      onClick={() => setHorizon(h)}
                    >
                      {h} mo
                    </button>
                  );
                })}
              </div>

              {longTermMode === "compare" && compareQuery.data && (
                <div className="grid" style={{ marginBottom: "1rem" }}>
                  <div className="card">
                    <div>Baseline end balance</div>
                    <div className="stat">{compareQuery.data.baseline.end_balance}</div>
                  </div>
                  <div className="card">
                    <div>ML end balance</div>
                    <div className="stat">
                      {compareQuery.data.ml_enhanced?.end_balance ?? "—"}
                    </div>
                  </div>
                  <div className="card">
                    <div>Delta at horizon</div>
                    <div className="stat">{compareQuery.data.delta_end_balance ?? "—"}</div>
                  </div>
                </div>
              )}

              {longTermMode !== "compare" && (
                <div className="card" style={{ marginBottom: "1rem" }}>
                  <div>End balance ({effectiveHorizon} months)</div>
                  <div className="stat">{longTermData?.end_balance ?? "—"}</div>
                  {longTermMode === "ml_enhanced" && longTermData?.end_balance_p10 && (
                    <p style={{ fontSize: "0.85rem", color: "#64748b", margin: "0.25rem 0 0" }}>
                      Band: {longTermData.end_balance_p10} – {longTermData.end_balance_p90}
                    </p>
                  )}
                </div>
              )}

              <div className="card" style={{ marginBottom: "1rem" }}>
                <button className="btn" onClick={() => setExplainOpen((o) => !o)}>
                  {explainOpen ? "Hide" : "How this forecast works"}
                </button>
                {explainOpen && (
                  <div style={{ marginTop: "0.75rem", fontSize: "0.9rem", color: "#475569" }}>
                    {mlAvailable ? (
                      <>
                        <p>
                          Model: {metaQuery.data?.model_family ?? longTermData?.model_family ?? "—"}
                        </p>
                        <p>
                          Seasonal periods:{" "}
                          {(metaQuery.data?.seasonal_periods ?? longTermData?.seasonal_periods)?.join(", ") ??
                            "—"}
                        </p>
                        <p>
                          Backtest WMAPE:{" "}
                          {metaQuery.data?.backtest_wmape ?? longTermData?.backtest_wmape ?? "—"}
                        </p>
                        {(longTermData?.low_confidence || metaQuery.data?.low_confidence) && (
                          <p style={{ color: "#b45309" }}>
                            Low confidence — bands shown at reduced opacity.
                          </p>
                        )}
                      </>
                    ) : (
                      mlExplainCopy
                    )}
                  </div>
                )}
              </div>

              <div className="card">
                <Suspense fallback={<p>Loading chart…</p>}>
                  {longTermMode === "compare" && compareQuery.data?.ml_enhanced ? (
                    <LongTermChart
                      compareSeries={[
                        {
                          label: "Baseline",
                          color: "#3b82f6",
                          series: compareQuery.data.baseline.series,
                        },
                        {
                          label: "ML-enhanced",
                          color: "#f97316",
                          series: compareQuery.data.ml_enhanced.series,
                        },
                      ]}
                    />
                  ) : (
                    <LongTermChart
                      series={longTermData?.series ?? []}
                      showBands={longTermMode === "ml_enhanced"}
                      lowConfidence={!!longTermData?.low_confidence}
                    />
                  )}
                </Suspense>
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}
