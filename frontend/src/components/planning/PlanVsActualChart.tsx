import ReactECharts from "echarts-for-react";
import { PlanVsActualRow } from "../../lib/api";

export function PlanVsActualChart({ rows }: { rows: PlanVsActualRow[] }) {
  const dates = rows.map((r) => r.date);
  const planned = rows.map((r) => (r.planned ? parseFloat(r.planned) : null));
  const actual = rows.map((r) => (r.actual ? parseFloat(r.actual) : null));
  const deviation = rows.map((r) => (r.deviation ? parseFloat(r.deviation) : null));

  const option = {
    tooltip: { trigger: "axis" },
    legend: { data: ["Planned", "Ist (actual)", "Deviation"] },
    xAxis: { type: "category", data: dates },
    yAxis: { type: "value" },
    series: [
      { name: "Planned", type: "line", data: planned, smooth: true },
      { name: "Ist (actual)", type: "line", data: actual, smooth: true },
      {
        name: "Deviation",
        type: "line",
        data: deviation,
        smooth: true,
        markLine: { data: [{ yAxis: 0 }] },
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 360 }} />;
}
