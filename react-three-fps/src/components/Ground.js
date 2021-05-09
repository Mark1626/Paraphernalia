import { usePlane } from "@react-three/cannon";
import React from "react";

export default () => {
  const [ref] = usePlane(() => ({
    rotation: [-Math.PI / 2, 0, 0],
  }));

  return (
    <mesh ref={ref} receiveShadow>
      <planeBufferGeometry attach="geometry" args={[100, 100]} />
      <meshBasicMaterial color={0x030} attach="material" />
    </mesh>
  );
};
