import type { ForecastMonthly } from "../lib/api";

export type ForecastMonthlyPoint = ForecastMonthly["series"][number];

function parseIncome(income: string): number {
  return parseFloat(income);
}

export function resolveForecastSummaryPoint(
  series: ForecastMonthlyPoint[],
): ForecastMonthlyPoint | null {
  if (series.length === 0) return null;
  if (parseIncome(series[0].income) === 0 && series.length > 1) {
    return series.find((p) => parseIncome(p.income) > 0) ?? series[0];
  }
  return series[0];
}

export function formatForecastMonthLabel(monthIso: string): string {
  const [year, month] = monthIso.slice(0, 7).split("-").map(Number);
  return new Date(year, month - 1, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });
}

export function formatForecastSummarySubtitle(monthIso: string): string {
  return `Forecast for ${formatForecastMonthLabel(monthIso)}`;
}
