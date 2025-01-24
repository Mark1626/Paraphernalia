import React, { useRef } from 'react'
import { useModel } from './hooks/useModel'
import { useLOD } from './hooks/useLOD'

// Import the model URLs using Parcel's URL import
export const lanternObjUrl = new URL('../assets/models/lantern.obj', import.meta.url).href
export const lanternMtlUrl = new URL('../assets/models/lantern.mtl', import.meta.url).href

// Simple lantern for LOD
function SimpleLantern() {
  return (
    <group>
      <mesh position={[0, 0.15, 0]}>
        <boxGeometry args={[0.2, 0.3, 0.2]} />
        <meshBasicMaterial color="#FFD700" />
      </mesh>
      <pointLight position={[0, 0.15, 0]} intensity={0.3} distance={3} />
    </group>
  )
}

// Very simple lantern for far distances
function VerySimpleLantern() {
  return (
    <group>
      <mesh position={[0, 0.15, 0]}>
        <boxGeometry args={[0.15, 0.2, 0.15]} />
        <meshBasicMaterial color="#FFD700" />
      </mesh>
      <pointLight position={[0, 0.15, 0]} intensity={0.2} distance={2} />
    </group>
  )
}

export function Lantern({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const groupRef = useRef()
  const lodLevel = useLOD(groupRef)
  
  const { groupRef: modelRef, obj } = useModel({
    objUrl: lanternObjUrl,
    mtlUrl: lanternMtlUrl,
    debug,
  })

  return (
    <group ref={groupRef} position={position} rotation={rotation} scale={scale}>
      {/* Debug cube, only shown if debug is true */}
      {debug && (
        <mesh position={[0, 0.5, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Render appropriate LOD level */}
      {lodLevel === 0 && (
        <primitive object={obj} ref={modelRef} scale={1} position={[0, 0, 0]} />
      )}
      {lodLevel === 1 && <SimpleLantern />}
      {lodLevel === 2 && <VerySimpleLantern />}
    </group>
  )
}
