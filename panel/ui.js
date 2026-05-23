export function createRenderer(output, state) {
  return function render(kind, detail) {
    output.textContent = JSON.stringify({
      kind,
      state,
      detail,
    }, null, 2);
  };
}
