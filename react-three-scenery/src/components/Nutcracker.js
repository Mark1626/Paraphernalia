import React, { useRef } from 'react'
import { useModel } from './hooks/useModel'
import { useLOD } from './hooks/useLOD'

// Import the model URLs using Parcel's URL import
export const nutcrackerObjUrl = new URL('../assets/models/nutcracker.obj', import.meta.url).href
export const nutcrackerMtlUrl = new URL('../assets/models/nutcracker.mtl', import.meta.url).href

// Simple nutcracker for LOD
function SimpleNutcracker() {
  return (
    <group>
      <mesh position={[0, 0.5, 0]}>
        <boxGeometry args={[0.3, 1, 0.3]} />
        <meshBasicMaterial color="red" />
      </mesh>
      <mesh position={[0, 1.2, 0]}>
        <sphereGeometry args={[0.2, 4, 4]} />
        <meshBasicMaterial color="black" />
      </mesh>
    </group>
  )
}

// Very simple nutcracker for far distances
function VerySimpleNutcracker() {
  return (
    <mesh position={[0, 0.5, 0]}>
      <boxGeometry args={[0.3, 1.2, 0.3]} />
      <meshBasicMaterial color="red" />
    </mesh>
  )
}

export function Nutcracker({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const groupRef = useRef()
  const lodLevel = useLOD(groupRef)
  
  const { groupRef: modelRef, obj } = useModel({
    objUrl: nutcrackerObjUrl,
    mtlUrl: nutcrackerMtlUrl,
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
      {lodLevel === 1 && <SimpleNutcracker />}
      {lodLevel === 2 && <VerySimpleNutcracker />}
    </group>
  )
}
