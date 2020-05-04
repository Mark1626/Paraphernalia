import React from "react";
import GenerateMaze from "../mazeHelper";
import Column from "./Column";

export default ({ position: basePosition, x, y }) => {
  const { maze } = GenerateMaze({ x, y });

  return (
    <group>
      {maze.map((row, x) =>
        row.map((col, y) => {
          const position = [...basePosition];
          position[0] += x;
          position[2] += y;
          return <Column position={position} type={col} />;
        })
      )}
    </group>
  );
};
