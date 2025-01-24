import React, { useState, useEffect, useRef } from 'react'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { 
  PointerLockControls, 
  Stats, 
  KeyboardControls, 
  useKeyboardControls,
  Preload,
  AdaptiveDpr,
  AdaptiveEvents
} from '@react-three/drei'
import Scene from './Scene'
import * as THREE from 'three'
import './App.css'

function FPSControls() {
  const { camera } = useThree()
  const [, get] = useKeyboardControls()
  const moveSpeed = 5
  const direction = new THREE.Vector3()
  const frontVector = new THREE.Vector3()
  const sideVector = new THREE.Vector3()

  // Set initial camera rotation to look straight ahead
  useEffect(() => {
    camera.rotation.set(0, 0, 0)
  }, [camera])

  useFrame((state, delta) => {
    const { forward, backward, left, right } = get()

    // Calculate movement direction
    frontVector.set(0, 0, Number(backward) - Number(forward))
    sideVector.set(Number(left) - Number(right), 0, 0)
    direction
      .subVectors(frontVector, sideVector)
      .normalize()
      .multiplyScalar(moveSpeed * delta)
      .applyEuler(camera.rotation)

    // Move camera
    camera.position.add(direction)
  })

  return null
}

function App() {
  const [isLocked, setIsLocked] = useState(false)

  useEffect(() => {
    const handleLockChange = () => {
      setIsLocked(document.pointerLockElement !== null)
    }

    document.addEventListener('pointerlockchange', handleLockChange)
    return () => document.removeEventListener('pointerlockchange', handleLockChange)
  }, [])

  return (
    <div className="App">
      <KeyboardControls map={[
        { name: 'forward', keys: ['ArrowUp', 'KeyW'] },
        { name: 'backward', keys: ['ArrowDown', 'KeyS'] },
        { name: 'left', keys: ['ArrowLeft', 'KeyA'] },
        { name: 'right', keys: ['ArrowRight', 'KeyD'] },
      ]}>
        <Canvas
          shadows="soft"
          dpr={[1, 2]}
          performance={{ min: 0.5 }}
          camera={{
            fov: 75,
            near: 0.1,
            far: 1000,
            position: [-2, 0.8, -2],
            rotation: [0, Math.PI / 4, 0],
          }}
          gl={{
            antialias: false,
            powerPreference: "high-performance",
            alpha: false,
            stencil: false,
            depth: true,
          }}
        >
          <Stats showPanel={0} />
          <AdaptiveDpr pixelated />
          <AdaptiveEvents />
          <Preload all />
          <ambientLight intensity={0.8} />
          <PointerLockControls makeDefault />
          <FPSControls />
          <Scene debug={false} />
        </Canvas>
      </KeyboardControls>
      <div className={`instructions ${isLocked ? 'hidden' : ''}`}>
        Click to start<br />
        WASD to move<br />
        Mouse to look<br />
        ESC to exit
      </div>
    </div>
  )
}

export default App
