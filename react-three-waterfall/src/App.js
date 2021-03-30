import React, { useEffect, useRef } from "react";
import { Canvas, useThree, useFrame, extend } from "@react-three/fiber";
import * as THREE from "three";
import { OrbitControls } from "@react-three/drei";
import Waterfall from "./components/Waterfall";

export default () => {
  const fov = 25;
  const near = 0.1;
  const far = 1000;
  const w = window.innerWidth
  const h = window.innerHeight
  const aspect = w / h;
  const { current: camera } = useRef(
    new THREE.PerspectiveCamera(fov, aspect, near, far)
  );

  useEffect(() => {
    camera.position.set(-5, 6, 8);
    camera.lookAt(new THREE.Vector3(0, 0, 0));
  }, [camera]);

  return (
    <Canvas
      camera={camera}
      shadows={{enabled: true, type: THREE.PCFSoftShadowMap}}
      onCreated={() => {

      }}
    >
      <OrbitControls />
      <ambientLight color={0xffffff} intensity={0.5} />
      <directionalLight color={0xffffff} position={[200, 200, 200]} castShadow />
      <directionalLight color={0xffffff} position={[-100, 200, 50]} castShadow />
      <Waterfall />
    </Canvas>
  );
};
