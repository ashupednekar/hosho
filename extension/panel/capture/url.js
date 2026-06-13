const SENSITIVE_QUERY_KEYS = new Set([
  "access_token",
  "api_key",
  "apikey",
  "auth",
  "client_secret",
  "code",
  "id_token",
  "key",
  "password",
  "refresh_token",
  "secret",
  "session",
  "sig",
  "signature",
  "token",
]);

export function sanitizedUrl(raw) {
  if (!raw) return raw;

  try {
    const url = new URL(raw);
    url.username = "";
    url.password = "";
    for (const key of Array.from(url.searchParams.keys())) {
      if (SENSITIVE_QUERY_KEYS.has(key.toLowerCase())) {
        url.searchParams.set(key, "REDACTED");
      }
    }
    return url.toString();
  } catch (_) {
    return raw;
  }
}
