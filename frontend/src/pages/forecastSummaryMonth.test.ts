import { describe, expect, it } from "vitest";
import type { ForecastMonthlyPoint } from "./forecastSummaryMonth";
import {
  formatForecastMonthLabel,
  formatForecastSummarySubtitle,
  resolveForecastSummaryPoint,
} from "./forecastSummaryMonth";

const partialMonthTrap: ForecastMonthlyPoint[] = [
  {
    month: "2026-06-01",
    income: "0.00",
    fixed_costs: "86.02",
    variable_costs: "2866.57",
    free_cashflow: "-2952.59",
  },
  {
    month: "2026-07-01",
    income: "3266.16",
    fixed_costs: "86.02",
    variable_costs: "2866.57",
    free_cashflow: "313.57",
  },
];

describe("resolveForecastSummaryPoint", () => {
  it("skips partial zero-income head to first month with income", () => {
    const resolved = resolveForecastSummaryPoint(partialMonthTrap);
    expect(resolved).toBe(partialMonthTrap[1]);
    expect(resolved?.income).toBe("3266.16");
  });

  it("uses series[0] when income is positive", () => {
    const series: ForecastMonthlyPoint[] = [
      {
        month: "2026-07-01",
        income: "3266.16",
        fixed_costs: "86.02",
        variable_costs: "2866.57",
        free_cashflow: "313.57",
      },
      {
        month: "2026-08-01",
        income: "3266.16",
        fixed_costs: "86.02",
        variable_costs: "2866.57",
        free_cashflow: "313.57",
      },
    ];
    const resolved = resolveForecastSummaryPoint(series);
    expect(resolved).toBe(series[0]);
    expect(resolved?.income).toBe("3266.16");
  });

  it("uses series[0] when all months have zero income", () => {
    const series: ForecastMonthlyPoint[] = [
      {
        month: "2026-06-01",
        income: "0.00",
        fixed_costs: "86.02",
        variable_costs: "2866.57",
        free_cashflow: "-2952.59",
      },
      {
        month: "2026-07-01",
        income: "0.00",
        fixed_costs: "86.02",
        variable_costs: "2866.57",
        free_cashflow: "-2952.59",
      },
    ];
    const resolved = resolveForecastSummaryPoint(series);
    expect(resolved).toBe(series[0]);
    expect(resolved?.income).toBe("0.00");
  });

  it("uses series[0] for a single-month series", () => {
    const series: ForecastMonthlyPoint[] = [
      {
        month: "2026-06-01",
        income: "0.00",
        fixed_costs: "86.02",
        variable_costs: "2866.57",
        free_cashflow: "-2952.59",
      },
    ];
    const resolved = resolveForecastSummaryPoint(series);
    expect(resolved).toBe(series[0]);
  });

  it("returns null for an empty series", () => {
    expect(resolveForecastSummaryPoint([])).toBeNull();
  });
});

describe("formatForecastMonthLabel", () => {
  it("derives month and year from API month ISO date", () => {
    const label = formatForecastMonthLabel("2026-07-01");
    expect(label).toContain("July");
    expect(label).toContain("2026");
  });
});

describe("formatForecastSummarySubtitle", () => {
  it("returns Forecast for {Month YYYY}", () => {
    expect(formatForecastSummarySubtitle("2026-07-01")).toBe("Forecast for July 2026");
  });
});
