import React, { useEffect, useRef, useState, useContext } from "react";
import * as THREE from "three";
import { Canvas, useFrame, extend, useThree } from "react-three-fiber";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { Stats } from "drei";
import Terrain from "./world/Terrain";

extend({ OrbitControls });

const Controls = () => {
  const { camera, gl } = useThree();
  const controls = useRef();
  useFrame(() => {
    controls.current.update();
  });
  return (
    <orbitControls
      ref={controls}
      target={[0, 0, 0]}
      enableDamping
      args={[camera, gl.domElement]}
    />
  );
};

export default () => {
  const { current: viewPos } = useRef({ lat: 0.0, lon: 0.0 });
  const [cameraPos, setCameraPos] = useState();

  const { current: camera } = useRef(
    new THREE.PerspectiveCamera(75, 1, 0.1, 1000)
  );

  useEffect(() => {
    camera.lookAt(new THREE.Vector3(0, 0, 0));
    camera.position.set(-83, 108, 138);
  }, [camera]);

  return (
    <div>
      <Canvas
        camera={camera}
        onCreated={({ gl }) => {
          gl.setClearAlpha(1);
        }}
      >
          <Stats showPanel={0} />
          <ambientLight intensity={0.25} />
          <pointLight position={[-150, 120, 30]} />
          <Controls />
          <Terrain />
      </Canvas>
    </div>
  );
};
