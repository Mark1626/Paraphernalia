import React, { useContext } from "react";
import Square from "./Square.jsx";
import { generateNestedSquareCoordinates } from "../utils";
import { CanvasContext } from "../context/CanvasProvider";

export default ({ size, x, y, xMov, yMov, steps }) => {
  const context = useContext(CanvasContext);

  const squares = generateNestedSquareCoordinates({ x, y, size, xMov, yMov, steps });

  return (
    <>
      {squares.map(({ xPos, yPos, size }, i) => (
        <Square key={`sq-${i}`} x={xPos} y={yPos} xMov={xMov} yMov={yMov} size={size} steps={steps}></Square>
      ))}
    </>
  );
};
