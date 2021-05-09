import { Physics } from "@react-three/cannon";
import { Sky, softShadows } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";

import React from "react";
import Cube from "./components/Cube";
import Ground from "./components/Ground";
import Player from "./components/Player";

softShadows();

export default () => {
  return (
    <Canvas shadowMap>
      <Sky distance={300} turbidity={8} inclination={0.49} />
      <ambientLight intensity={0.5} />
      <directionalLight
        castShadow
        receiveShadow
        position={[10, 20, 15]}
        intensity={1}
      />
      <Physics>
        <Ground />
        <Cube position={[-2, 1, -6]} />
        <Player />
      </Physics>
    </Canvas>
  );
};
