import ReactECharts from "echarts-for-react";

interface Point {
  date: string;
  balance: string;
}

export function DailyChart({ series }: { series: Point[] }) {
  const option = {
    tooltip: { trigger: "axis" as const },
    xAxis: {
      type: "category" as const,
      data: series.map((p) => p.date),
    },
    yAxis: { type: "value" as const },
    series: [
      {
        name: "Balance",
        type: "line" as const,
        smooth: true,
        data: series.map((p) => parseFloat(p.balance)),
        areaStyle: { opacity: 0.08 },
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 360 }} notMerge lazyUpdate />;
}
