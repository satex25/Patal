import { invoke } from "@tauri-apps/api/core";

window.addEventListener("DOMContentLoaded", () => {
  const button = document.querySelector<HTMLButtonElement>("#engine-demo-button");
  const result = document.querySelector<HTMLParagraphElement>("#engine-demo-result");

  button?.addEventListener("click", async () => {
    if (!result) return;
    const perimeterMm = await invoke<number>("engine_demo_perimeter_mm");
    result.textContent = `Rust engine computed a ${perimeterMm}mm perimeter.`;
  });
});
