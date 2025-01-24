import React, { useRef } from 'react'
import { useModel } from './hooks/useModel'
import { useLOD } from './hooks/useLOD'

// Import the model URLs using Parcel's URL import
export const reindeerObjUrl = new URL(
  '../assets/models/reindeer.obj',
  import.meta.url
).href
export const reindeerMtlUrl = new URL(
  '../assets/models/reindeer.mtl',
  import.meta.url
).href

// Simple reindeer for LOD
function SimpleReindeer() {
  return (
    <group>
      <mesh position={[0, 0.5, 0]}>
        <boxGeometry args={[0.8, 0.6, 0.4]} />
        <meshBasicMaterial color="brown" />
      </mesh>
      <mesh position={[0.3, 0.8, 0]}>
        <boxGeometry args={[0.3, 0.3, 0.3]} />
        <meshBasicMaterial color="brown" />
      </mesh>
      <mesh position={[0.4, 1, 0]} rotation={[0, 0, -Math.PI / 4]}>
        <cylinderGeometry args={[0.02, 0.02, 0.3, 4]} />
        <meshBasicMaterial color="brown" />
      </mesh>
    </group>
  )
}

// Very simple reindeer for far distances
function VerySimpleReindeer() {
  return (
    <mesh position={[0, 0.5, 0]}>
      <boxGeometry args={[1, 0.6, 0.4]} />
      <meshBasicMaterial color="brown" />
    </mesh>
  )
}

export function Reindeer({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const groupRef = useRef()
  const lodLevel = useLOD(groupRef)
  
  const { groupRef: modelRef, obj } = useModel({
    objUrl: reindeerObjUrl,
    mtlUrl: reindeerMtlUrl,
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
      {lodLevel === 1 && <SimpleReindeer />}
      {lodLevel === 2 && <VerySimpleReindeer />}
    </group>
  )
}
