import { useSphere } from "@react-three/cannon";
import { PointerLockControls } from "@react-three/drei";
import { useFrame, useThree } from "@react-three/fiber";
import React, { useEffect, useRef, useState } from "react";
import { Vector3 } from "three";
import useKeyboard from "../useKeyboard";

const WALK_SPEED = 5;

export default () => {
  const [ref, api] = useSphere(() => ({
    mass: 1,
    position: [0, 2, 0],
  }));
  const movement = useKeyboard();
  const { camera } = useThree();
  const currentVelocity = useRef([0, 0, 0]);

  useEffect(() => {
    api.velocity.subscribe(
      (newVelocity) => (currentVelocity.current = newVelocity)
    );
  }, []);

  useFrame(() => {
    camera.position.copy(ref.current.position);
    const frontVector = new Vector3(
      0,
      0,
      (movement.backward ? 1 : 0) - (movement.forward ? 1 : 0)
    );
    const sideVector = new Vector3(
      0,
      0,
      (movement.left ? 1 : 0) - (movement.right ? 1 : 0)
    );

    const newVelocity = new Vector3()
      .subVectors(frontVector, sideVector)
      .normalize()
      .multiplyScalar(WALK_SPEED)
      .applyEuler(camera.rotation);

    api.velocity.set(newVelocity.x, currentVelocity.current[1], newVelocity.z);
  });

  return (
    <>
      <mesh ref={ref} />
      <PointerLockControls />
    </>
  );
};
