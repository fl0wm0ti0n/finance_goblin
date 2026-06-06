import type { ReactNode } from "react";
import type { UserManagerEvents, UserManagerSettings } from "oidc-client-ts";
import {
  AuthContext,
  type AuthContextProps,
} from "react-oidc-context";

const noop = () => undefined;
const noopUnsub = () => undefined;

const devBypassEvents = new Proxy({} as UserManagerEvents, {
  get: (_target, prop) => {
    if (typeof prop === "string" && prop.startsWith("add")) {
      return () => noopUnsub;
    }
    if (typeof prop === "string" && prop.startsWith("remove")) {
      return noop;
    }
    if (typeof prop === "string" && prop.startsWith("_raise")) {
      return async () => undefined;
    }
    if (prop === "load" || prop === "unload") {
      return async () => undefined;
    }
    return noop;
  },
});

const devBypassSettings = Object.freeze({
  authority: "",
  client_id: "dev-bypass",
  redirect_uri: "",
}) as UserManagerSettings;

export function createDevBypassAuthValue(): AuthContextProps {
  return {
    isLoading: false,
    isAuthenticated: false,
    user: undefined,
    error: undefined,
    activeNavigator: undefined,
    settings: devBypassSettings,
    events: devBypassEvents,
    clearStaleState: async () => {},
    removeUser: async () => {},
    signinPopup: async () => {
      throw new Error("OIDC not configured");
    },
    signinSilent: async () => null,
    signinRedirect: async () => {},
    signinResourceOwnerCredentials: async () => {
      throw new Error("OIDC not configured");
    },
    signoutRedirect: async () => {},
    signoutPopup: async () => {},
    signoutSilent: async () => {},
    querySessionStatus: async () => null,
    revokeTokens: async () => {},
    startSilentRenew: noop,
    stopSilentRenew: noop,
  };
}

export function DevBypassAuthProvider({ children }: { children: ReactNode }) {
  return (
    <AuthContext.Provider value={createDevBypassAuthValue()}>
      {children}
    </AuthContext.Provider>
  );
}
