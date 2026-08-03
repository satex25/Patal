import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", () => {
  const button = document.querySelector<HTMLButtonElement>("#engine-demo-button");
  const result = document.querySelector<HTMLParagraphElement>("#engine-demo-result");

  button?.addEventListener("click", async () => {
    if (!result) return;
    // The engine reports bad geometry as an error rather than returning a
    // number that looks fine, so the caller has to handle both.
    try {
      const perimeterMm = await invoke<number>("engine_demo_perimeter_mm");
      result.textContent = `Rust engine computed a ${perimeterMm}mm perimeter.`;
    } catch (error) {
      result.textContent = `Engine could not compute that outline: ${error}`;
    }
  });
});
