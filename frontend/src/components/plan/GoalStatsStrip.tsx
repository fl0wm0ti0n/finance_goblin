import { GoalStats } from "../../lib/api";

export function GoalStatsStrip({ stats }: { stats: GoalStats }) {
  return (
    <div className="card" style={{ marginBottom: "1rem" }}>
      <h3 style={{ marginTop: 0 }}>Goal progress</h3>
      <p style={{ fontSize: "0.9rem", color: "#64748b" }}>
        Target €{stats.target_balance_eur} by {stats.target_date}
        {stats.goal_account_id ? ` · account ${stats.goal_account_id}` : ""}
        {stats.household_fallback ? " · household balance (no account selected)" : ""}
      </p>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(10rem, 1fr))",
          gap: "0.75rem",
          marginTop: "0.75rem",
        }}
      >
        <div>
          <strong>Monthly delta vs baseline</strong>
          <div>€{stats.monthly_delta_vs_baseline}</div>
        </div>
        {stats.yearly_rollup.map((y) => (
          <div key={y.year}>
            <strong>{y.year} rollup</strong>
            <div>€{y.planned_net_sum}</div>
          </div>
        ))}
        <div>
          <strong>Projected at target</strong>
          <div>
            {stats.beyond_horizon
              ? "Beyond 730-day horizon"
              : stats.projected_balance_at_target
                ? `€${stats.projected_balance_at_target}`
                : "—"}
          </div>
        </div>
      </div>
      {!stats.on_track && stats.gap_eur && !stats.beyond_horizon && (
        <p style={{ marginTop: "0.75rem", color: "#b45309" }}>
          Gap €{stats.gap_eur}
          {stats.required_monthly_savings_eur
            ? ` · save ~€${stats.required_monthly_savings_eur}/mo (0% interest estimate)`
            : ""}
        </p>
      )}
      {stats.on_track && !stats.beyond_horizon && (
        <p style={{ marginTop: "0.75rem", color: "#16a34a" }}>On track for target balance.</p>
      )}
    </div>
  );
}
