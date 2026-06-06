import ReactECharts from "echarts-for-react";

interface MonthlyPoint {
  month: string;
  income: string;
  fixed_costs: string;
  variable_costs: string;
  free_cashflow: string;
}

export function MonthlyChart({ series }: { series: MonthlyPoint[] }) {
  const months = series.map((p) => p.month.slice(0, 7));
  const option = {
    tooltip: { trigger: "axis" as const },
    legend: { data: ["Income", "Fixed", "Variable", "Free cashflow"] },
    xAxis: { type: "category" as const, data: months },
    yAxis: { type: "value" as const },
    series: [
      {
        name: "Income",
        type: "bar" as const,
        data: series.map((p) => parseFloat(p.income)),
      },
      {
        name: "Fixed",
        type: "bar" as const,
        data: series.map((p) => parseFloat(p.fixed_costs)),
      },
      {
        name: "Variable",
        type: "bar" as const,
        data: series.map((p) => parseFloat(p.variable_costs)),
      },
      {
        name: "Free cashflow",
        type: "bar" as const,
        data: series.map((p) => parseFloat(p.free_cashflow)),
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 400 }} notMerge lazyUpdate />;
}
