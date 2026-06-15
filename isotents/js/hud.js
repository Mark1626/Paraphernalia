/**
 * Screen-space puzzle HUD: progress readout, controls hint, and win banner.
 * Drawn without the camera transform so it stays pinned to the viewport.
 */

export function renderHUD(ctx, width, height, info) {
  const { levelName, houseCount, treeCount, won } = info;

  ctx.save();
  ctx.setTransform(1, 0, 0, 1, 0, 0);

  // ── Status bar (top-left) ────────────────────────────────
  ctx.font = "bold 14px monospace";
  ctx.textBaseline = "top";
  ctx.textAlign = "left";
  const status = `${levelName}   Houses ${houseCount}/${treeCount}`;
  const sw = ctx.measureText(status).width + 24;
  ctx.fillStyle = "rgba(0,0,0,0.5)";
  ctx.fillRect(0, 0, sw, 36);
  ctx.fillStyle = "#fff";
  ctx.fillText(status, 12, 11);

  // ── Controls hint (bottom-center) ────────────────────────
  const hint =
    "Left-click: house   ·   Right-click / drag: mark empty   ·   N: new   ·   R: reset";
  ctx.font = "bold 12px monospace";
  ctx.textAlign = "center";
  const hw = ctx.measureText(hint).width + 28;
  ctx.fillStyle = "rgba(0,0,0,0.55)";
  ctx.fillRect(Math.round(width / 2 - hw / 2), height - 40, hw, 28);
  ctx.fillStyle = "#ddd";
  ctx.fillText(hint, width / 2, height - 31);

  // ── Win banner (center) ──────────────────────────────────
  if (won) {
    const msg = "SOLVED!";
    ctx.font = "bold 40px monospace";
    const mw = ctx.measureText(msg).width + 64;
    const bx = Math.round(width / 2 - mw / 2);
    const by = Math.round(height / 2 - 44);
    ctx.fillStyle = "rgba(0,0,0,0.7)";
    ctx.fillRect(bx, by, mw, 88);
    ctx.strokeStyle = "#5d5";
    ctx.lineWidth = 3;
    ctx.strokeRect(bx, by, mw, 88);
    ctx.fillStyle = "#5d5";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(msg, width / 2, height / 2 - 6);
    ctx.font = "bold 13px monospace";
    ctx.fillStyle = "#aaa";
    ctx.fillText("Press N for a new puzzle", width / 2, height / 2 + 26);
  }

  ctx.restore();
}
