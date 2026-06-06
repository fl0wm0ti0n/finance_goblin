import { WebStorageStateStore } from "oidc-client-ts";

const authority = import.meta.env.VITE_OIDC_AUTHORITY ?? "";
const clientId = import.meta.env.VITE_OIDC_CLIENT_ID ?? "flow-finance-ai";
const redirectUri =
  import.meta.env.VITE_OIDC_REDIRECT_URI ?? `${window.location.origin}/callback`;
const postLogoutRedirectUri =
  import.meta.env.VITE_OIDC_POST_LOGOUT_REDIRECT_URI ?? window.location.origin;

export const oidcConfig = {
  authority,
  client_id: clientId,
  redirect_uri: redirectUri,
  post_logout_redirect_uri: postLogoutRedirectUri,
  response_type: "code",
  scope: "openid profile email",
  automaticSilentRenew: true,
  userStore: new WebStorageStateStore({ store: window.localStorage }),
  onSigninCallback: () => {
    window.history.replaceState({}, document.title, window.location.pathname);
  },
};

export const isOidcConfigured = Boolean(authority);
