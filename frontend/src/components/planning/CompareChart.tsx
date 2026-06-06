import ReactECharts from "echarts-for-react";
import { PlanCompare } from "../../lib/api";

export function CompareChart({ data }: { data: PlanCompare }) {
  const versions = data.versions.map((v) => `v${v.version_number}`);
  const deltas = data.versions.map((v) => parseFloat(v.monthly_delta_sum));
  const balances = data.versions.map((v) => parseFloat(v.projected_month_end_balance));

  const option = {
    tooltip: { trigger: "axis" },
    legend: { data: ["Monthly delta sum", "Month-end balance"] },
    xAxis: { type: "category", data: versions },
    yAxis: { type: "value" },
    series: [
      { name: "Monthly delta sum", type: "bar", data: deltas },
      { name: "Month-end balance", type: "bar", data: balances },
    ],
  };

  return <ReactECharts option={option} style={{ height: 320 }} />;
}
