import { invoke } from "@tauri-apps/api/core";

/// A 120Hz frame, in microseconds. The number the drag-loop benchmark is
/// measured against, shown live here so the two agree in public.
const FRAME_BUDGET_US = 8333;

type Point = [number, number];

interface CutPreview {
  outline: Point[];
  cut: Point[] | null;
  error: string | null;
  outline_vertices: number;
  elapsed_micros: number;
}

interface SaveReport {
  path: string;
  bytes: number;
  schema_version: number;
  round_tripped: boolean;
  material_name: string | null;
}

const el = <T extends HTMLElement>(id: string): T => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
};

/// Fit every point on screen with a margin, preserving aspect ratio and
/// flipping y — pattern space is y-up, canvas space is y-down, and getting
/// that wrong silently mirrors the piece.
function fitTransform(points: Point[], width: number, height: number) {
  const xs = points.map((p) => p[0]);
  const ys = points.map((p) => p[1]);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);

  const margin = 32;
  const scale = Math.min(
    (width - margin * 2) / Math.max(maxX - minX, 1e-9),
    (height - margin * 2) / Math.max(maxY - minY, 1e-9),
  );

  const offsetX = (width - (maxX - minX) * scale) / 2 - minX * scale;
  const offsetY = (height - (maxY - minY) * scale) / 2 + maxY * scale;

  return (p: Point): Point => [p[0] * scale + offsetX, offsetY - p[1] * scale];
}

function strokeClosed(
  ctx: CanvasRenderingContext2D,
  points: Point[],
  project: (p: Point) => Point,
  colour: string,
  lineWidth: number,
) {
  if (points.length === 0) return;
  ctx.beginPath();
  points.forEach((point, index) => {
    const [x, y] = project(point);
    if (index === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.closePath();
  ctx.strokeStyle = colour;
  ctx.lineWidth = lineWidth;
  ctx.stroke();
}

function draw(preview: CutPreview) {
  const canvas = el<HTMLCanvasElement>("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  // Match the backing store to the CSS size so lines are not blurry on a
  // high-DPI display — a blurry preview would undermine the one thing this
  // window is for.
  const ratio = window.devicePixelRatio || 1;
  const cssWidth = canvas.clientWidth || 900;
  const cssHeight = canvas.clientHeight || 620;
  canvas.width = Math.round(cssWidth * ratio);
  canvas.height = Math.round(cssHeight * ratio);
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, cssWidth, cssHeight);

  const all = preview.cut ? [...preview.outline, ...preview.cut] : preview.outline;
  const project = fitTransform(all, cssWidth, cssHeight);

  if (preview.cut) strokeClosed(ctx, preview.cut, project, "#0ea5e9", 2);
  strokeClosed(ctx, preview.outline, project, "#a3a3a3", 1.5);

  // Vertices, but only when there are few enough to mean anything. At 800
  // points they merge into a smear that reads as a thick line.
  if (preview.outline.length <= 160) {
    ctx.fillStyle = "#737373";
    for (const point of preview.outline) {
      const [x, y] = project(point);
      ctx.beginPath();
      ctx.arc(x, y, 1.8, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

async function refresh() {
  const tolerance = Math.pow(10, Number(el<HTMLInputElement>("tolerance").value));
  const allowance = Number(el<HTMLInputElement>("allowance").value);

  el("tolerance-value").textContent = `${tolerance.toFixed(3)} mm`;
  el("allowance-value").textContent = `${allowance.toFixed(1)} mm`;

  const errorBox = el("engine-error");

  try {
    const preview = await invoke<CutPreview>("cut_preview", {
      toleranceMm: tolerance,
      allowanceMm: allowance,
    });

    draw(preview);

    el("stat-vertices").textContent = String(preview.outline_vertices);
    el("stat-time").textContent = `${preview.elapsed_micros} µs`;
    const share = (preview.elapsed_micros / FRAME_BUDGET_US) * 100;
    el("stat-budget").textContent = `${share.toFixed(1)}% of 120Hz`;

    if (preview.error) {
      // The engine's own words, verbatim. Paraphrasing them here would
      // start exactly the second error vocabulary this project spent a
      // whole commit deleting.
      errorBox.textContent = preview.error;
      errorBox.classList.remove("hidden");
    } else {
      errorBox.classList.add("hidden");
    }
  } catch (error) {
    errorBox.textContent = String(error);
    errorBox.classList.remove("hidden");
  }
}

window.addEventListener("DOMContentLoaded", () => {
  el("tolerance").addEventListener("input", refresh);
  el("allowance").addEventListener("input", refresh);
  window.addEventListener("resize", refresh);

  el("save-button").addEventListener("click", async () => {
    const result = el("save-result");
    const tolerance = Math.pow(10, Number(el<HTMLInputElement>("tolerance").value));
    try {
      const report = await invoke<SaveReport>("save_demo_document", {
        directory: ".",
        toleranceMm: tolerance,
      });
      result.textContent =
        `Wrote ${report.bytes} bytes to ${report.path} (schema v${report.schema_version}). ` +
        `Round trip ${report.round_tripped ? "matched" : "DID NOT MATCH"}; ` +
        `material resolved to ${report.material_name ?? "none"}.`;
    } catch (error) {
      result.textContent = `Save failed: ${error}`;
    }
  });

  void refresh();
});
