import React, { Suspense, useMemo } from 'react'
import { Sky, useGLTF } from '@react-three/drei'
import { Tree } from './components/Tree'
import { Nutcracker } from './components/Nutcracker'
import { Lantern } from './components/Lantern'
import {
  PresentACube,
  PresentARectangle,
  PresentARound,
  PresentBCube,
  PresentBRectangle,
  PresentBRound,
} from './components/presents'
import { LargeRocks, MediumRocks, SmallRocks } from './components/rocks'
import { Reindeer } from './components/Reindeer'
import { DebugGrid } from './components/DebugGrid'

function LoadingBox() {
  return (
    <mesh position={[0, 0.5, 0]}>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color="gray" wireframe />
    </mesh>
  )
}

// Preload all models
useGLTF.preload('./assets/models/tree.obj')
useGLTF.preload('./assets/models/tree-snow-a.obj')
useGLTF.preload('./assets/models/lantern.obj')
useGLTF.preload('./assets/models/nutcracker.obj')

export default function Scene({ debug = false }) {
  // Memoize static elements
  const lights = useMemo(() => (
    <>
      <directionalLight
        position={[5, 5, 5]}
        intensity={1.5}
        castShadow
        shadow-mapSize-width={1024} // Reduced for performance
        shadow-mapSize-height={1024}
        shadow-camera-far={50}
        shadow-camera-left={-10}
        shadow-camera-right={10}
        shadow-camera-top={10}
        shadow-camera-bottom={-10}
      />
      <directionalLight position={[-5, 3, -5]} intensity={0.5} />
      <ambientLight intensity={0.4} />
    </>
  ), [])

  const sky = useMemo(() => (
    <Sky
      distance={450000}
      sunPosition={[0, 1, 0]}
      inclination={0}
      azimuth={0.25}
      rayleigh={2}
      turbidity={10}
    />
  ), [])

  return (
    <group>
      {sky}
      {lights}

      {/* Debug grid, only shown if debug is true */}
      {debug && <DebugGrid size={20} divisions={100} />}

      <Suspense fallback={<LoadingBox />}>
        {/* Trees */}
        <Tree position={[4, 0, 4]} scale={0.5} debug={debug} />
        <Tree position={[5, 0, 3]} scale={0.45} debug={debug} snow={true} />
        <Tree position={[3, 0, 5]} scale={0.4} debug={debug} snow={true} />

        {/* Decorative Items */}
        {/* Nutcrackers guarding the path */}
        <Nutcracker 
          position={[-2, 0, -2]} 
          rotation={[0, Math.PI/4, 0]} 
          scale={0.4} 
          debug={debug} 
        />
        <Nutcracker 
          position={[2, 0, -2]} 
          rotation={[0, -Math.PI/4, 0]} 
          scale={0.4} 
          debug={debug} 
        />
        
        {/* Lanterns lighting the path */}
        <Lantern position={[-1.5, 0, -1.5]} scale={0.3} debug={debug} />
        <Lantern position={[1.5, 0, -1.5]} scale={0.3} debug={debug} />
        <Lantern position={[-1, 0, 0]} scale={0.3} debug={debug} />
        <Lantern position={[1, 0, 0]} scale={0.3} debug={debug} />
        <Lantern position={[-0.5, 0, 1.5]} scale={0.3} debug={debug} />
        <Lantern position={[0.5, 0, 1.5]} scale={0.3} debug={debug} />

        {/* Lanterns around the tree */}
        <Lantern position={[3.5, 0, 4.5]} scale={0.25} debug={debug} />
        <Lantern position={[4.5, 0, 4.5]} scale={0.25} debug={debug} />
        <Lantern position={[4.5, 0, 3.5]} scale={0.25} debug={debug} />
        <Lantern position={[3.5, 0, 3.5]} scale={0.25} debug={debug} />

        {/* Nutcrackers guarding the tree */}
        <Nutcracker 
          position={[3.2, 0, 3.2]} 
          rotation={[0, -Math.PI/4, 0]} 
          scale={0.35} 
          debug={debug} 
        />
        <Nutcracker 
          position={[4.8, 0, 3.2]} 
          rotation={[0, Math.PI/4, 0]} 
          scale={0.35} 
          debug={debug} 
        />
        <Nutcracker 
          position={[4, 0, 5]} 
          rotation={[0, Math.PI, 0]} 
          scale={0.35} 
          debug={debug} 
        />

        {/* Nutcracker by the tree */}
        <Nutcracker 
          position={[3.5, 0, 3.5]} 
          rotation={[0, -Math.PI/3, 0]} 
          scale={0.35} 
          debug={debug} 
        />

        {/* Present Set A */}
        <PresentACube position={[3, 0, 3.5]} scale={0.5} debug={debug} />
        <PresentARectangle position={[3.5, 0, 3.8]} scale={0.5} debug={debug} />
        <PresentARound position={[4, 0, 3.2]} scale={0.5} debug={debug} />

        {/* Present Set B */}
        <PresentBCube position={[4.5, 0, 3.5]} scale={0.5} debug={debug} />
        <PresentBRectangle position={[5, 0, 3.8]} scale={0.5} debug={debug} />
        <PresentBRound position={[4.2, 0, 4.5]} scale={0.5} debug={debug} />

        {/* Rocks */}
        <LargeRocks position={[-3, 0, 2]} scale={0.4} debug={debug} />
        <MediumRocks position={[0, 0, 3]} scale={0.4} debug={debug} />
        <SmallRocks position={[3, 0, 2]} scale={0.4} debug={debug} />

        {/* Additional rock formations */}
        <LargeRocks position={[-4, 0, -3]} scale={0.3} rotation={[0, 1, 0]} debug={debug} />
        <MediumRocks position={[4, 0, -3]} scale={0.35} rotation={[0, -0.5, 0]} debug={debug} />
        <SmallRocks position={[0, 0, -4]} scale={0.3} debug={debug} />

        {/* Reindeer */}
        <Reindeer
          position={[-2.5, 0, 1]}
          rotation={[0, Math.PI / 4, 0]}
          scale={0.4}
          debug={debug}
        />
        <Reindeer
          position={[2.5, 0, 1]}
          rotation={[0, -Math.PI / 4, 0]}
          scale={0.4}
          debug={debug}
        />
        <Reindeer
          position={[0, 0, 2]}
          rotation={[0, Math.PI, 0]}
          scale={0.4}
          debug={debug}
        />

      </Suspense>

      {/* Ground plane */}
      <mesh
        rotation={[-Math.PI / 2, 0, 0]}
        position={[0, 0, 0]}
        receiveShadow
      >
        <planeGeometry args={[20, 20]} />
        <meshStandardMaterial color="#355E3B" />
      </mesh>
    </group>
  )
}
