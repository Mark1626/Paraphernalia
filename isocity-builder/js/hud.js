import { getState, currentPlaceable } from "./game.js";

/**
 * Draw the game HUD (resources, AP, turn info, phase prompt).
 * Drawn in screen space (no camera transform).
 */
export function renderHUD(ctx, width, height) {
  const s = getState();

  ctx.save();
  ctx.setTransform(1, 0, 0, 1, 0, 0);

  // ── Resource bar (top-left) ──────────────────────────────
  const resources = [
    { label: "Food", value: s.resources.food, color: "#f4a" },
    { label: "Wood", value: s.resources.wood, color: "#6b4" },
    { label: "Water", value: s.resources.water, color: "#4bf" },
    { label: "Stone", value: s.resources.stone, color: "#aaa" },
  ];

  ctx.font = "bold 14px monospace";
  ctx.textBaseline = "top";
  ctx.textAlign = "left";

  // Background
  ctx.fillStyle = "rgba(0,0,0,0.5)";
  ctx.fillRect(0, 0, 360, 36);

  let x = 12;
  for (const r of resources) {
    ctx.fillStyle = r.color;
    ctx.fillText(`${r.label}: ${r.value}`, x, 10);
    x += 88;
  }

  // ── Turn / AP (top-right) ────────────────────────────────
  const turnText = `Turn ${s.turn}  |  AP: ${s.ap}`;
  ctx.textAlign = "right";
  ctx.fillStyle = "rgba(0,0,0,0.5)";
  const tw = ctx.measureText(turnText).width + 24;
  ctx.fillRect(width - tw, 0, tw, 36);
  ctx.fillStyle = "#fff";
  ctx.fillText(turnText, width - 12, 10);

  // ── Phase prompt (bottom-center) ─────────────────────────
  let prompt = "";
  if (s.phase === "place_house") {
    prompt = "Click a tile to place your house";
  } else if (s.phase === "play" && s.ap > 0) {
    prompt = "Click a tile to place a workshop  |  [Enter] End turn";
  } else if (s.phase === "turn_over") {
    prompt = "Turn over — press [Enter] for next turn";
  }

  if (prompt) {
    ctx.textAlign = "center";
    ctx.font = "bold 13px monospace";
    const pw = ctx.measureText(prompt).width + 32;
    ctx.fillStyle = "rgba(0,0,0,0.6)";
    ctx.fillRect(Math.round(width / 2 - pw / 2), height - 44, pw, 32);
    ctx.fillStyle = "#ff0";
    ctx.fillText(prompt, width / 2, height - 32);
  }

  ctx.restore();
}
