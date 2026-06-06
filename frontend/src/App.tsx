import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import { useAuth } from "react-oidc-context";
import { useEffect } from "react";
import { AppLayout } from "./components/AppLayout";
import { HomePage } from "./pages/HomePage";
import { SyncStatusPage } from "./pages/SyncStatusPage";
import { SettingsPage } from "./pages/SettingsPage";
import { ForecastPage } from "./pages/ForecastPage";
import { SubscriptionsPage } from "./pages/SubscriptionsPage";
import { PlanningPage } from "./pages/PlanningPage";
import { WealthPage } from "./pages/WealthPage";
import { AlertsPage } from "./pages/AlertsPage";
import { AnalyticsEmbedPage } from "./pages/AnalyticsEmbedPage";
import { ChatPage } from "./pages/ChatPage";
import { OidcCallback } from "./pages/OidcCallback";
import { isOidcConfigured } from "./auth/oidc";
import { setAccessTokenProvider } from "./lib/api";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  if (!isOidcConfigured) {
    return <>{children}</>;
  }
  return <OidcProtectedRoute>{children}</OidcProtectedRoute>;
}

function OidcProtectedRoute({ children }: { children: React.ReactNode }) {
  const auth = useAuth();
  const location = useLocation();

  useEffect(() => {
    setAccessTokenProvider(() => auth.user?.access_token ?? null);
  }, [auth.user?.access_token]);

  if (auth.isLoading) {
    return <div className="content">Loading authentication…</div>;
  }

  if (!auth.isAuthenticated) {
    auth.signinRedirect({ state: { returnTo: location.pathname } });
    return <div className="content">Redirecting to login…</div>;
  }

  return <>{children}</>;
}

export default function App() {
  return (
    <Routes>
      <Route path="/callback" element={<OidcCallback />} />
      <Route
        path="/*"
        element={
          <ProtectedRoute>
            <AppLayout>
              <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/sync" element={<SyncStatusPage />} />
                <Route path="/forecast" element={<ForecastPage />} />
                <Route path="/subscriptions" element={<SubscriptionsPage />} />
                <Route path="/planning" element={<PlanningPage />} />
                <Route path="/wealth" element={<WealthPage />} />
                <Route path="/alerts" element={<AlertsPage />} />
                <Route path="/analytics/:slug" element={<AnalyticsEmbedPage />} />
                <Route path="/chat" element={<ChatPage />} />
                <Route path="/settings" element={<SettingsPage />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </AppLayout>
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}
