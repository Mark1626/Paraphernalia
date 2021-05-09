import { useBox } from "@react-three/cannon";
import React from "react";

export default ({ position }) => {
  const [ref] = useBox(() => ({
    position,
  }));

  return (
    <mesh ref={ref} castShadow>
      <boxBufferGeometry attach="geometry" />
      <meshBasicMaterial color={0xff01fb} attach="material" />
    </mesh>
  );
};
