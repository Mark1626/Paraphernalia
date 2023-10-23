export const generateNestedSquareCoordinates = ({
  x,
  y,
  xMov,
  yMov,
  size,
  steps
}) => {
  let squares = [{ xPos: x, yPos: y, size }];

  const startSize = size;
  const finalSize = 10;
  let xPos = x;
  let yPos = y;

  let newSize, newX, newY;

  for (let step = steps; step > 0; step -= 1) {
    newSize = startSize * (step / steps) + finalSize;
    newX = x + (size - newSize) / 2;
    newY = y + (size - newSize) / 2;
    newX = newX - ((x - newX) / (step + 2)) * xMov;
    newY = newY - ((y - newY) / (step + 2)) * yMov;

    squares.push({ xPos: newX, yPos: newY, size: newSize });

    x = newX;
    y = newY;
    size = newSize;
  }

  return squares;
};

const directions = [-1, 0, 1];

export const generateTileCoordinates = ({
  size,
  offset,
  startSize,
  tileStep
}) => {
  let coordinates = [];
  for (var x = offset; x < size - offset - 10; x += tileStep) {
    for (var y = offset; y < size - offset - 10; y += tileStep) {
      const startSteps = 2 + Math.ceil(Math.random() * 7);
      const xMov = directions[Math.floor(Math.random() * directions.length)];
      const yMov = directions[Math.floor(Math.random() * directions.length)];

      coordinates.push({
        x,
        y,
        size: startSize,
        xMov,
        yMov,
        steps: startSteps - 1
      });
    }
  }
  return coordinates;
};
