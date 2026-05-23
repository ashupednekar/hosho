export function createTransport({ apiBase, render }) {
  return {
    async send(path, payload) {
      try {
        await fetch(`${apiBase}${path}`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
      } catch (error) {
        render("send failed", {
          path,
          message: error instanceof Error ? error.message : String(error),
        });
      }
    },
  };
}
