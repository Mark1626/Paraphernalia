import * as React from "react";
import Canvas from "../components/Canvas.jsx";
import Square from "../canvasComponents/Square.jsx";
import HypnoticSquare from "../canvasComponents/HypnoticSquare.jsx";
import { generateTileCoordinates } from "../utils";

export default function Home() {
  const offset = 24;
  const size = 500;
  const tileStep = (size - offset * 2) / 10;
  const startSize = tileStep - 10;
  const coordinates = generateTileCoordinates({
    size,
    offset,
    startSize,
    tileStep
  });

  return (
    <>
      <Canvas>
        {coordinates.map((coordinate, i) => (
          <HypnoticSquare key={`tile-${i}`} {...coordinate} />
        ))}
      </Canvas>
    </>
  );
}
