/**
 * Simple camera with pan (drag) and zoom (scroll) support.
 */
export class Camera {
  constructor(canvas) {
    this.x = 0;
    this.y = 0;
    this.zoom = 1;
    this.minZoom = 0.25;
    this.maxZoom = 3;

    /** When true, only middle-mouse (button 1) drags the camera. */
    this.editorMode = false;

    this._dragging = false;
    this._lastMouse = { x: 0, y: 0 };

    canvas.addEventListener("mousedown", (e) => this._onMouseDown(e));
    canvas.addEventListener("mousemove", (e) => this._onMouseMove(e));
    canvas.addEventListener("mouseup", () => this._onMouseUp());
    canvas.addEventListener("mouseleave", () => this._onMouseUp());
    // canvas.addEventListener("wheel", (e) => this._onWheel(e), { passive: false });
  }

  /** Apply camera transform to a canvas 2d context. */
  apply(ctx) {
    ctx.setTransform(this.zoom, 0, 0, this.zoom, this.x, this.y);
  }

  /** Convert a screen-space point to world-space. */
  screenToWorld(sx, sy) {
    return {
      x: (sx - this.x) / this.zoom,
      y: (sy - this.y) / this.zoom,
    };
  }

  _onMouseDown(e) {
    // In editor mode, only middle-mouse drags (left-click is for painting)
    const canDrag = this.editorMode ? (e.button === 1) : (e.button === 0 || e.button === 1);
    if (canDrag) {
      this._dragging = true;
      this._lastMouse = { x: e.clientX, y: e.clientY };
    }
  }

  _onMouseMove(e) {
    if (!this._dragging) return;
    const dx = e.clientX - this._lastMouse.x;
    const dy = e.clientY - this._lastMouse.y;
    this.x += dx;
    this.y += dy;
    this._lastMouse = { x: e.clientX, y: e.clientY };
  }

  _onMouseUp() {
    this._dragging = false;
  }

  _onWheel(e) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    const newZoom = Math.min(
      this.maxZoom,
      Math.max(this.minZoom, this.zoom * factor),
    );

    // Zoom towards cursor position
    const wx = (e.clientX - this.x) / this.zoom;
    const wy = (e.clientY - this.y) / this.zoom;
    this.zoom = newZoom;
    this.x = e.clientX - wx * this.zoom;
    this.y = e.clientY - wy * this.zoom;
  }
}
