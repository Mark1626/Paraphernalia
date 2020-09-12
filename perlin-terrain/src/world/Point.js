import React, { useRef } from "react";
import * as THREE from "three";
import { noise, remap } from "./rng";
import { pointCount, pointGap, gridHeight } from "./constants";

export default ({ x, y, z, pointSize, i, j }) => {
  const forest = {
    x: 821.231,
    y: 832.2231,
    peakColor: { x: 0.45, y: 0.85, z: 0.45 },
    valleyColor: { x: 0.4, y: 0.4, z: 0.05 },
    cityColor: { x: 0.45, y: 0.45, z: 0 },
    waterColor: { x: 0, y: 0.6, z: 0.9 },
    waterHeight: -5,
    heightMul: 40,
    cityChance: 0.25,
    cloudChance: 0.4,
  };

  const forestX = forest.x + 0.1 * 50 * (i + 1);
  const forestY = forest.y + 0.1 * 50 * (j + 1);
  const terrainNoise = noise.perlin2(forestX, forestY) * -forest.heightMul;

  // console.log(terrainNoise)
  const position = [
    x - (pointCount * pointGap) / 2 + pointGap,
    terrainNoise,
    z - (pointCount * pointGap) / 2,
  ];
  const color =
    position[1] < -100
      ? new THREE.Color(
          forest.waterColor.x,
          forest.waterColor.y,
          forest.waterColor.z
        )
      : new THREE.Color(
          remap(
            position[1],
            gridHeight - forest.heightMul * 0.8,
            gridHeight - forest.heightMul * 0.1,
            forest.peakColor.x,
            forest.valleyColor.x,
            true
          ),
          remap(
            position[1],
            gridHeight - forest.heightMul * 0.8,
            gridHeight - forest.heightMul * 0.1,
            forest.peakColor.y,
            forest.valleyColor.y,
            true
          ),
          remap(
            position[1],
            gridHeight - forest.heightMul * 0.8,
            gridHeight - forest.heightMul * 0.1,
            forest.peakColor.z,
            forest.valleyColor.z,
            true
          )
        );

  const pointRef = useRef({
    position: { x: position[0], y: position[1], z: position[2] },
  });

  return (
    <mesh ref={pointRef} position={position}>
      <circleGeometry attach="geometry" args={[pointSize, 4]} />
      {/* <boxGeometry attach="geometry" args={[pointGap, pointGap, pointGap]} /> */}
      <meshBasicMaterial attach="material" color={color} />
    </mesh>
  );
};
