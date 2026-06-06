import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AuthProvider } from "react-oidc-context";
import App from "./App";
import { DevBypassAuthProvider } from "./auth/DevBypassAuthProvider";
import { isOidcConfigured, oidcConfig } from "./auth/oidc";
import "./index.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

const appTree = (
  <QueryClientProvider client={queryClient}>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </QueryClientProvider>
);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isOidcConfigured ? (
      <AuthProvider {...oidcConfig}>{appTree}</AuthProvider>
    ) : (
      <DevBypassAuthProvider>{appTree}</DevBypassAuthProvider>
    )}
  </React.StrictMode>,
);
