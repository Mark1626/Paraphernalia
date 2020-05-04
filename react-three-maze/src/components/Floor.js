import React from "react";

export default ({ position, scale=1 }) => {

  const geometry = [6.5*scale, 0.2, 4.5*scale]
  return <mesh position={position} receiveShadow>
    <boxGeometry attach="geometry" args={geometry} />
    <meshLambertMaterial attach="material" color="#718093" />
  </mesh>;
};
