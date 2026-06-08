import { lazy, Suspense, useState } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  apiFetch,
  ExtendedWealthBreakdown,
  HoldingsAllRow,
  PortfolioForecast,
  WealthHistoryPoint,
} from "../lib/api";

const WealthChart = lazy(() =>
  import("../components/wealth/WealthChart").then((m) => ({ default: m.WealthChart })),
);
const CategoryFilter = lazy(() =>
  import("../components/category/CategoryFilter").then((m) => ({ default: m.CategoryFilter })),
);
const CategoryTrendChart = lazy(() =>
  import("../components/category/CategoryTrendChart").then((m) => ({
    default: m.CategoryTrendChart,
  })),
);

type Tab = "overview" | "crypto";

function isStale(lastSync: string | null | undefined): boolean {
  if (!lastSync) return true;
  const hours = (Date.now() - new Date(lastSync).getTime()) / (1000 * 60 * 60);
  return hours > 25;
}

export function WealthPage() {
  const [tab, setTab] = useState<Tab>("overview");
  const [categoryId, setCategoryId] = useState("");

  const breakdownQuery = useQuery({
    queryKey: ["wealth"],
    queryFn: () => apiFetch<ExtendedWealthBreakdown>("/api/v1/wealth"),
  });

  const historyQuery = useQuery({
    queryKey: ["wealth-history"],
    queryFn: () => apiFetch<WealthHistoryPoint[]>("/api/v1/wealth/history?days=90"),
  });

  const portfolioForecastQuery = useQuery({
    queryKey: ["portfolio-forecast"],
    queryFn: () => apiFetch<PortfolioForecast>("/api/v1/wealth/portfolio-forecast"),
    enabled: tab === "crypto",
    retry: false,
  });

  const data = breakdownQuery.data;
  const stale = isStale(data?.last_successful_sync_at ?? null);
  const hasAccounts = (data?.firefly.accounts.length ?? 0) > 0;
  const allZeroBalances =
    hasAccounts && data!.firefly.accounts.every((row) => row.balance === 0);
  const showZeroTotalCallout = hasAccounts && (data?.total_eur === 0 || allZeroBalances);
  const hasOverdrawn = data?.firefly.accounts.some((row) => row.is_overdrawn) ?? false;

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", marginBottom: "1rem" }}>
        <h1 style={{ margin: 0 }}>Wealth</h1>
        {stale && <span className="badge stale">Stale sync</span>}
      </div>

      <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem" }}>
        {(["overview", "crypto"] as Tab[]).map((t) => (
          <button
            key={t}
            type="button"
            className={`btn ${tab === t ? "btn-primary" : ""}`}
            onClick={() => setTab(t)}
          >
            {t === "overview" ? "Overview" : "Crypto"}
          </button>
        ))}
      </div>

      {data?.firefly.mixed_currency && (
        <div className="card" style={{ borderColor: "#f59e0b", marginBottom: "1rem" }}>
          <strong>Mixed currency warning</strong>
          <p style={{ margin: "0.5rem 0 0" }}>
            Firefly accounts use multiple currencies. Subtotals sum native balances without FX
            conversion — interpret with caution.
          </p>
        </div>
      )}

      {data?.fx_incomplete && (
        <div className="card" style={{ borderColor: "#f59e0b", marginBottom: "1rem" }}>
          <strong>FX incomplete</strong>
          <p style={{ margin: "0.5rem 0 0" }}>
            Some crypto assets could not be priced in EUR and are excluded from the crypto subtotal:{" "}
            {(data.crypto.unpriced_assets ?? []).join(", ") || "unknown assets"}.
          </p>
        </div>
      )}

      {showZeroTotalCallout && (
        <div className="card" style={{ borderColor: "#b45309", marginBottom: "1rem" }}>
          <strong>Account balances may be stale</strong>
          <p style={{ margin: "0.5rem 0 0" }}>
            Trigger a <Link to="/sync">Full Firefly sync</Link> from Settings to refresh account
            balances. If totals still look wrong, reconcile accounts in{" "}
            <a href="https://docs.firefly-iii.org/" target="_blank" rel="noreferrer">
              Firefly III
            </a>
            .
          </p>
          {hasOverdrawn && (
            <p style={{ margin: "0.5rem 0 0" }}>
              Overdrawn accounts are included in the signed total — a negative balance can reduce net
              worth after sync.
            </p>
          )}
        </div>
      )}

      {tab === "overview" && (
        <>
          <div className="grid">
            <div className="card">
              <div>Total wealth ({data?.reporting_currency ?? "EUR"})</div>
              <div className="stat">
                {data
                  ? `€${data.total_eur.toLocaleString(undefined, { minimumFractionDigits: 2 })}`
                  : "—"}
              </div>
            </div>
            <div className="card">
              <div>Firefly subtotal</div>
              <div className="stat">
                {data
                  ? `€${data.firefly.subtotal_eur.toLocaleString(undefined, { minimumFractionDigits: 2 })}`
                  : "—"}
              </div>
            </div>
            <div className="card">
              <div>Crypto subtotal</div>
              <div className="stat">
                {data?.crypto_placeholder
                  ? "—"
                  : data
                    ? `€${data.crypto.subtotal_eur.toLocaleString(undefined, { minimumFractionDigits: 2 })}`
                    : "—"}
              </div>
            </div>
            <div className="card">
              <div>Portfolio analytics (Grafana)</div>
              <Link to="/analytics/portfolio">Open portfolio dashboard →</Link>
            </div>
          </div>

          {data?.allocation_gap && (
            <div className="card" style={{ marginTop: "1rem" }}>
              <h2>Allocation vs target</h2>
              <p>
                Current: ETF/traditional {data.allocation_gap.current.etf_traditional_pct.toFixed(1)}%
                · Crypto {data.allocation_gap.current.crypto_pct.toFixed(1)}% · Cash{" "}
                {data.allocation_gap.current.cash_pct.toFixed(1)}%
              </p>
              <p>
                Target: ETF/traditional {data.allocation_gap.target.etf_traditional_pct.toFixed(1)}%
                · Crypto {data.allocation_gap.target.crypto_pct.toFixed(1)}% · Cash{" "}
                {data.allocation_gap.target.cash_pct.toFixed(1)}%
              </p>
              <Link to="/planning">Adjust on Planning →</Link>
            </div>
          )}

          <div className="card" style={{ marginTop: "1rem" }}>
            <h2>Category spending</h2>
            <p style={{ fontSize: "0.9rem", color: "#475569" }}>
              Mirror actuals for a single category — net worth and crypto totals above remain
              household-level.
            </p>
            <Suspense fallback={<p>Loading category filter…</p>}>
              <CategoryFilter value={categoryId} onChange={setCategoryId} allowAll={false} />
            </Suspense>
            {categoryId && (
              <div style={{ marginTop: "1rem" }}>
                <Suspense fallback={<p>Loading trend…</p>}>
                  <CategoryTrendChart categoryId={categoryId} title="Category spending trend" />
                </Suspense>
              </div>
            )}
          </div>

          <div className="card" style={{ marginTop: "1rem" }}>
            <h2>Account breakdown</h2>
            <table className="data-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Role</th>
                  <th>Currency</th>
                  <th>Balance</th>
                  <th>% of Firefly</th>
                </tr>
              </thead>
              <tbody>
                {data?.firefly.accounts.map((row) => (
                  <tr
                    key={row.firefly_id}
                    style={row.is_overdrawn ? { background: "#fffbeb" } : undefined}
                  >
                    <td>
                      {row.name}
                      {row.is_overdrawn && (
                        <span
                          className="badge"
                          style={{ marginLeft: "0.5rem", background: "#fef3c7", color: "#b45309" }}
                        >
                          Overdrawn
                        </span>
                      )}
                    </td>
                    <td>{row.account_role ?? "—"}</td>
                    <td>{row.currency}</td>
                    <td style={row.is_overdrawn ? { color: "#b45309" } : undefined}>
                      {row.balance.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </td>
                    <td>{row.pct_of_total != null ? `${row.pct_of_total.toFixed(1)}%` : "—"}</td>
                  </tr>
                ))}
                {data?.crypto_placeholder && (
                  <tr style={{ opacity: 0.6 }}>
                    <td colSpan={5}>
                      No exchanges connected — configure in{" "}
                      <Link to="/settings">Settings</Link>.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </>
      )}

      {tab === "crypto" && (
        <>
          {!data?.crypto_placeholder && data?.crypto.exchanges.length === 0 ? (
            <div className="card">
              <p>No exchanges connected.</p>
              <Link to="/settings">Configure exchange API keys in Settings →</Link>
            </div>
          ) : (
            <>
              {!portfolioForecastQuery.data?.skipped &&
                (portfolioForecastQuery.data?.horizons.length ?? 0) > 0 && (
                  <div className="grid" style={{ marginBottom: "1rem" }}>
                    {portfolioForecastQuery.data?.horizons.map((h) => (
                      <div key={h.months} className="card">
                        <div>Projected crypto ({h.months} mo)</div>
                        <div className="stat">€{h.value_eur}</div>
                        {h.value_p10 && h.value_p90 && (
                          <div style={{ fontSize: "0.85rem", color: "#64748b" }}>
                            Band: €{h.value_p10} – €{h.value_p90}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}

              <div className="grid">
                {data?.crypto.exchanges.map((ex) => (
                  <div key={ex.id} className="card">
                    <div style={{ textTransform: "capitalize" }}>{ex.id}</div>
                    <div className="badge">{ex.connection_state}</div>
                    <div>
                      €{ex.subtotal_eur.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </div>
                    <div style={{ fontSize: "0.85rem", color: "#64748b" }}>
                      Last sync:{" "}
                      {ex.last_sync_at ? new Date(ex.last_sync_at).toLocaleString() : "Never"}
                    </div>
                  </div>
                ))}
              </div>

              <div className="card" style={{ marginTop: "1rem" }}>
                <h2>PnL summary</h2>
                <p>
                  Realized: €
                  {(data?.pnl.realized_eur ?? 0).toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                  })}{" "}
                  · Unrealized: €
                  {(data?.pnl.unrealized_eur ?? 0).toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                  })}{" "}
                  · Total return:{" "}
                  {data?.pnl.total_return_pct != null
                    ? `${data.pnl.total_return_pct.toFixed(2)}%`
                    : "—"}
                </p>
                <p style={{ fontSize: "0.85rem", color: "#64748b" }}>
                  Wealth analytics only — not tax reporting.
                </p>
              </div>

              <div className="card" style={{ marginTop: "1rem" }}>
                <h2>Holdings</h2>
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Asset</th>
                      <th>Type</th>
                      <th>Native qty</th>
                      <th>Unit</th>
                      <th>Value EUR</th>
                      <th>Unrealized PnL</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(data?.crypto.holdings_all ?? []).map((h: HoldingsAllRow) => (
                      <tr key={`${h.asset}-${h.product_type}-${h.native_unit}`}>
                        <td>{h.asset}</td>
                        <td>{h.product_type}</td>
                        <td>
                          {h.quantity.toLocaleString(undefined, { maximumFractionDigits: 8 })}
                        </td>
                        <td>{h.native_unit}</td>
                        <td>
                          {h.value_eur != null
                            ? h.value_eur.toLocaleString(undefined, { minimumFractionDigits: 2 })
                            : "—"}
                        </td>
                        <td>
                          {h.unrealized_pnl_eur != null
                            ? h.unrealized_pnl_eur.toLocaleString(undefined, {
                                minimumFractionDigits: 2,
                              })
                            : "—"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <p style={{ fontSize: "0.85rem", color: "#64748b", marginTop: "0.75rem" }}>
                  Linear contract unrealized PnL is shown in EUR but excluded from the crypto
                  subtotal.
                </p>
              </div>
            </>
          )}
        </>
      )}

      {historyQuery.data && historyQuery.data.length > 0 && tab === "overview" && (
        <div className="card" style={{ marginTop: "1rem" }}>
          <h2>Wealth over time</h2>
          <Suspense fallback={<div>Loading chart…</div>}>
            <WealthChart data={historyQuery.data} />
          </Suspense>
        </div>
      )}
    </div>
  );
}
