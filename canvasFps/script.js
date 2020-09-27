
const canvas = document.getElementById("c");
canvas.width = 1920;
canvas.height = 1080;
const x = canvas.getContext("2d");

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

  x.fillRect(10, 10, 50, 50);
  stats.end();
}

init();
