import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GoalStatsStrip } from "./GoalStatsStrip";
import type { GoalStats } from "../../lib/api";

const baseStats: GoalStats = {
  plan_id: "p1",
  version_id: "v1",
  target_balance_eur: "10000.00",
  target_date: "2026-11-01",
  goal_account_id: "114",
  monthly_delta_vs_baseline: "-120.00",
  yearly_rollup: [{ year: 2026, planned_net_sum: "-500.00" }],
  projected_balance_at_target: "9200.00",
  gap_eur: "800.00",
  required_monthly_savings_eur: "160.00",
  on_track: false,
  beyond_horizon: false,
  computed_at: null,
  household_fallback: false,
};

describe("GoalStatsStrip", () => {
  it("renders monthly delta and gap copy when off-track", () => {
    render(<GoalStatsStrip stats={baseStats} />);
    expect(screen.getByText(/Goal progress/)).toBeInTheDocument();
    expect(screen.getByText(/Monthly delta vs baseline/)).toBeInTheDocument();
    expect(screen.getByText(/Gap €800.00/)).toBeInTheDocument();
  });

  it("shows beyond-horizon copy when flagged", () => {
    render(<GoalStatsStrip stats={{ ...baseStats, beyond_horizon: true, projected_balance_at_target: null }} />);
    expect(screen.getByText(/Beyond 730-day horizon/)).toBeInTheDocument();
  });
});
