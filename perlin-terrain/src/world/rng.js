import Noise from "../../vendor/noisejs";

export const noise = new Noise(50);

export const clamp = function (n, low, high) {
  return Math.max(Math.min(n, high), low);
};

export const remap = (n, start1, stop1, start2, stop2, withinBounds = false) => {
  const newval = ((n - start1) / (stop1 - start1)) * (stop2 - start2) + start2;
  if (!withinBounds) {
    return newval;
  }
  if (start2 < stop2) {
    return clamp(newval, start2, stop2);
  } else {
    return clamp(newval, stop2, start2);
  }
};
