import { NavLink } from "react-router-dom";
import { useAuth } from "react-oidc-context";
import { useState } from "react";
import { isOidcConfigured } from "../auth/oidc";
import { AlertBell } from "./AlertBell";
import { AiSheet } from "./AiSheet";
import { StaleBanner } from "./StaleBanner";
import { useStaleDetection } from "../hooks/useStaleDetection";

const navItems = [
  { to: "/", label: "Home", enabled: true },
  { to: "/sync", label: "Sync Status", enabled: true },
  { to: "/forecast", label: "Forecast", enabled: true },
  { to: "/subscriptions", label: "Subscriptions", enabled: true },
  { to: "/settings", label: "Settings", enabled: true },
  { to: "/planning", label: "Planning", enabled: true },
  { to: "/wealth", label: "Wealth", enabled: true },
  { to: "/chat", label: "AI", enabled: true },
];

const analyticsNavItems = [
  { to: "/analytics/platform-health", label: "Platform Health" },
  { to: "/analytics/cashflow", label: "Cashflow" },
  { to: "/analytics/subscriptions", label: "Subscriptions" },
  { to: "/analytics/budgets", label: "Budgets" },
  { to: "/analytics/portfolio", label: "Portfolio" },
  { to: "/analytics/forecast-horizons", label: "Forecast Horizons" },
];

export function AppLayout({ children }: { children: React.ReactNode }) {
  const [collapsed, setCollapsed] = useState(false);
  const auth = useAuth();
  const { stale } = useStaleDetection();

  return (
    <div className={`app-shell ${collapsed ? "sidebar-collapsed" : ""}`}>
      <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          {!collapsed && <strong>Flow Finance AI</strong>}
          <button
            className="btn"
            style={{ padding: "0.25rem 0.5rem", fontSize: "0.75rem" }}
            onClick={() => setCollapsed((c) => !c)}
            aria-label="Toggle sidebar"
          >
            {collapsed ? "»" : "«"}
          </button>
        </div>
        <nav style={{ marginTop: "1rem" }}>
          {navItems.map((item) =>
            item.enabled ? (
              <NavLink
                key={item.label}
                to={item.to}
                className={({ isActive }) => (isActive ? "active" : undefined)}
                end={item.to === "/"}
              >
                {collapsed ? item.label[0] : item.label}
              </NavLink>
            ) : (
              <span key={item.label} className="disabled" title="Coming soon">
                {collapsed ? item.label[0] : `${item.label} (Coming soon)`}
              </span>
            ),
          )}
          {!collapsed && (
            <div style={{ marginTop: "1rem", fontSize: "0.75rem", color: "#64748b" }}>
              Analytics
            </div>
          )}
          {analyticsNavItems.map((item) => (
            <NavLink
              key={item.label}
              to={item.to}
              className={({ isActive }) => (isActive ? "active" : undefined)}
            >
              {collapsed ? item.label[0] : item.label}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-footer">
          {isOidcConfigured && auth.user?.profile?.name && (
            <div>{auth.user.profile.name as string}</div>
          )}
          {isOidcConfigured && (
            <button
              className="btn"
              style={{ marginTop: "0.5rem", width: "100%" }}
              onClick={() => auth.signoutRedirect()}
            >
              Logout
            </button>
          )}
          <div
            style={{
              marginTop: "0.5rem",
              fontSize: "0.7rem",
              color: "#94a3b8",
              fontFamily: "monospace",
            }}
            title={`Release: ${__RELEASE_TAG__}\nBuild: ${__BUILD_ID__}\nTimestamp: ${new Date().toISOString()}`}
          >
            {__BUILD_ID__.slice(0, 7)}
          </div>
        </div>
      </aside>
      <div className="main">
        <header className="header">
          <div>
            <strong>Flow Finance AI</strong>
            <span style={{ marginLeft: "0.75rem", color: "#64748b", fontSize: "0.9rem" }}>
              Read-only Firefly analytics
            </span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
            <span className="badge read-only">Read-only</span>
            <AiSheet />
            <AlertBell />
          </div>
        </header>
        <StaleBanner stale={stale} />
        <main className="content">{children}</main>
      </div>
    </div>
  );
}
