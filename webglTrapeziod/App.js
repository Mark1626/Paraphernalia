import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { Canvas, useThree, extend, useFrame } from "react-three-fiber";
import TrapezoidPolyhedra from "./components/TrapezoidPolyhedra";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import TrapezoidCylinder from "./components/TrapezoidCylinder";

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
  const { aspect } = useThree();
  const fov = 25;
  const near = 0.01;
  const far = 1000;
  const camera = new THREE.PerspectiveCamera(fov, aspect, near, far);

  useEffect(() => {
    camera.position.set(-45, 45, 30);
    camera.lookAt(new THREE.Vector3(1, 0, -1));
  }, [camera]);

  return (
    <Canvas
      camera={camera}
      onCreated={({ gl }) => {
        gl.setClearAlpha(1);
      }}
    >
      <Controls />
      <TrapezoidCylinder position={[5, 0.15, 1]} />
      <TrapezoidPolyhedra position={[2, 0.15, 1]} />
      <ambientLight intensity={0.25} />
      <pointLight position={[10, 10, 10]} />
    </Canvas>
  );
};
