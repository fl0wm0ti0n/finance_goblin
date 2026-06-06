import { Navigate, useParams } from "react-router-dom";

const EMBED_BASE = import.meta.env.VITE_GRAFANA_EMBED_BASE ?? "/analytics/grafana";

const DASHBOARDS: Record<string, { uid: string; title: string }> = {
  "platform-health": { uid: "platform-health", title: "Platform Health" },
  cashflow: { uid: "cashflow", title: "Cashflow" },
  subscriptions: { uid: "subscriptions", title: "Subscriptions" },
  budgets: { uid: "budgets", title: "Budgets" },
  portfolio: { uid: "portfolio", title: "Portfolio" },
  "forecast-horizons": { uid: "forecast-horizons", title: "Forecast Horizons" },
};

export function AnalyticsEmbedPage() {
  const { slug } = useParams<{ slug: string }>();
  const dash = slug ? DASHBOARDS[slug] : undefined;

  if (!dash) {
    return <Navigate to="/analytics/platform-health" replace />;
  }

  const src = `${EMBED_BASE}/d/${dash.uid}/${slug}?kiosk=tv`;

  return (
    <div className="analytics-embed">
      <h1 style={{ margin: "0 0 0.75rem" }}>{dash.title}</h1>
      <iframe
        title={dash.title}
        src={src}
        className="analytics-embed-frame"
        allow="fullscreen"
      />
    </div>
  );
}
