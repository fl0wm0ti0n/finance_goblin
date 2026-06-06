import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { AlertRow, apiFetch } from "../lib/api";

export function AlertsPage() {
  const queryClient = useQueryClient();

  const alertsQuery = useQuery({
    queryKey: ["alerts"],
    queryFn: () => apiFetch<AlertRow[]>("/api/v1/alerts?status=active"),
  });

  const acknowledgeMut = useMutation({
    mutationFn: (id: string) =>
      apiFetch<AlertRow>(`/api/v1/alerts/${id}/acknowledge`, { method: "PATCH" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["alerts"] });
      queryClient.invalidateQueries({ queryKey: ["alerts-unread"] });
    },
  });

  const dismissMut = useMutation({
    mutationFn: (id: string) =>
      apiFetch<AlertRow>(`/api/v1/alerts/${id}/dismiss`, { method: "PATCH" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["alerts"] });
      queryClient.invalidateQueries({ queryKey: ["alerts-unread"] });
    },
  });

  const alerts = alertsQuery.data ?? [];

  return (
    <div>
      <h1>Alerts</h1>
      <p style={{ color: "#64748b" }}>
        Unified alert inbox for scarcity, budget drift, and plan viability.{" "}
        <Link to="/subscriptions">Subscription alerts</Link> remain on the Subscriptions page.
      </p>

      {alerts.length === 0 ? (
        <div className="card">
          <p>No active alerts.</p>
        </div>
      ) : (
        <div className="card">
          <table className="data-table">
            <thead>
              <tr>
                <th>Type</th>
                <th>Severity</th>
                <th>Title</th>
                <th>Message</th>
                <th>Triggered</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {alerts.map((alert) => (
                <tr
                  key={alert.id}
                  style={alert.status === "acknowledged" ? { opacity: 0.7 } : undefined}
                >
                  <td>{alert.alert_type}</td>
                  <td>
                    <span className={`badge severity-${alert.severity}`}>{alert.severity}</span>
                  </td>
                  <td>{alert.title}</td>
                  <td>{alert.message}</td>
                  <td>{new Date(alert.triggered_at).toLocaleString()}</td>
                  <td style={{ whiteSpace: "nowrap" }}>
                    {alert.status === "active" && (
                      <button
                        className="btn"
                        style={{ marginRight: "0.5rem" }}
                        onClick={() => acknowledgeMut.mutate(alert.id)}
                      >
                        Acknowledge
                      </button>
                    )}
                    {alert.status === "acknowledged" && (
                      <span style={{ marginRight: "0.5rem", fontSize: "0.85rem", color: "#64748b" }}>
                        Acknowledged — condition still active
                      </span>
                    )}
                    <button className="btn" onClick={() => dismissMut.mutate(alert.id)}>
                      Dismiss
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
