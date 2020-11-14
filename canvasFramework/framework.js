const c = document.getElementById("c");
c.width = 1920;
c.height = 1040;
const x = c.getContext("2d");
const S = Math.sin;
const C = Math.cos;
const T = Math.tan;
const R = (r, g, b, a) => {
  a = a ? a : 1;
  return `rgba(${r}, ${g}, ${b}, ${a})`;
};

let time = 0;
let frame = 0;
let nextFrameMs = 0;
const FPS = 60;
const epsilon = 1.5;

const stats = new Stats();
document.body.appendChild(stats.dom);

const init = () => {
  requestAnimationFrame(loop);
}

const loop = (frame_time) => {
  stats.begin();
  requestAnimationFrame(loop);
  if (frame_time < nextFrameMs - epsilon) {
    return;
  }
  nextFrameMs = Math.max(nextFrameMs + 1000 / FPS, frame_time);

  time = frame/FPS;
  if(time * FPS | 0 == frame - 1){
    time += 0.000001;
  }
  frame++;

  u(time);
  stats.end();
}

let t = 0;
const starttime = performance.now();

init();
