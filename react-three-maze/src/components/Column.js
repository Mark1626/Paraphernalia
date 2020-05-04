import React from "react";

export default ({ position, type }) => {
  return (
    <mesh position={position} receiveShadow castShadow>
      {type == 0 ? (
        <></>
      ) : type == 1 ? (
        <boxGeometry attach="geometry" args={[1.5, 3, 0.25]} />
      ) : type == 2 ? (
        <boxGeometry attach="geometry" args={[0.25, 3, 1.5]} />
      ) : (
        <boxGeometry attach="geometry" args={[0.5, 3, 0.5]} />
      )}
      <meshLambertMaterial attach="material" color="#ffffff" />
    </mesh>
  );
};
