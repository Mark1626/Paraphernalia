const canvas = document.getElementById("c");
const x = canvas.getContext("2d");
const S = Math.sin;
const C = Math.cos;
const T = Math.tan;
const R = (r, g, b, a) => {
  a = a ? a : 1;
  return `rgba(${r}, ${g}, ${b}, ${a})`;
};

let t = 0;
const starttime = performance.now();

const init = () => {
  window.requestAnimationFrame(draw);
};

const draw = () => {
  u(t);
  t = (performance.now() - starttime) / 1000;
  window.requestAnimationFrame(draw);
};

init();