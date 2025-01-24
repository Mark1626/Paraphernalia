import React from 'react'
import { useModel } from './hooks/useModel'

// Import the model URLs using Parcel's URL import
export const nutcrackerObjUrl = new URL('../assets/models/nutcracker.obj', import.meta.url).href
export const nutcrackerMtlUrl = new URL('../assets/models/nutcracker.mtl', import.meta.url).href

export function Nutcracker({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const { groupRef, obj } = useModel({
    objUrl: nutcrackerObjUrl,
    mtlUrl: nutcrackerMtlUrl,
    debug,
  })

  return (
    <group ref={groupRef} position={position} rotation={rotation}>
      {/* Debug cube, only shown if debug is true */}
      {debug && (
        <mesh position={[0, 0.5, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Nutcracker model */}
      <primitive object={obj} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}
