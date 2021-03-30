import React from "react";
import CustomShadow from "./CustomShadow";
import {Colors} from "../constants/color"

export default ({ position }) => {
  const trunkPosition = [...position];
  trunkPosition[1] = 0.275;
  const leavesPosition = [...position];
  leavesPosition[1] = 0.2 + 0.15 + 0.4 / 2;
  const leavesGeometry = (
    <boxGeometry attach="geometry" args={[0.25, 0.4, 0.25]} />
  );

  return (
    <group>
      {/* Trunk */}
      <mesh position={trunkPosition} receiveShadow castShadow>
        <boxGeometry attach="geometry" args={[0.15, 0.15, 0.15]} />
        <meshLambertMaterial attach="material" color={Colors.brownDark} />
      </mesh>
      {/* Leaves */}
      <CustomShadow
        position={leavesPosition}
        geometry={leavesGeometry}
        alpha={0.25}
      />
      <mesh position={leavesPosition} castShadow>
        {leavesGeometry}
        <meshLambertMaterial attach="material" color={Colors.green} />
      </mesh>
    </group>
  );
};
