import React, { useEffect, useRef } from "react";
import { Canvas, useThree, useFrame, extend } from "react-three-fiber";
import * as THREE from "three";
import Floor from "./components/Floor";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import Maze from "./components/Maze";
import Column from "./components/Column";

extend({ OrbitControls });

const Controls = () => {
  const { camera, gl } = useThree();
  const controls = useRef();
  useFrame(() => controls.current.update());
  return (
    <orbitControls
      ref={controls}
      target={[0, 0, 0]}
      enableDamping
      autoRotate
      autoRotateSpeed={[-0.5]}
      args={[camera, gl.domElement]}
    />
  );
};

export default () => {
  const fov = 25;
  const near = 0.01;
  const far = 1000;
  const { aspect } = useThree();
  const { current: camera } = useRef(
    new THREE.PerspectiveCamera(fov, aspect, near, far)
  );

  useEffect(() => {
    camera.position.set(-45, 45, 30);
    camera.lookAt(new THREE.Vector3(1, 0.3, 1));
  }, [camera]);

  return (
    <Canvas
      camera={camera}
      vr
      onCreated={({ gl, camera }) => {
        gl.toneMapping = THREE.Uncharted2ToneMapping;
        gl.setClearAlpha(1);
      }}
    >
      <Controls />
      <ambientLight intensity={0.25} />
      <pointLight position={[10, 10, 10]} />
      <Floor position={[0, -1.5, 1.05]} scale={4} />
      {/* <Column position={[0, 0, 1.05]} type={3} /> */}
      <Maze x={11} y={7} position={[-12, 0.1, -6.0]} />
    </Canvas>
  );
};
