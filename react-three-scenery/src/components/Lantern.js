import React from 'react'
import { useModel } from './hooks/useModel'

// Import the model URLs using Parcel's URL import
export const lanternObjUrl = new URL('../assets/models/lantern.obj', import.meta.url).href
export const lanternMtlUrl = new URL('../assets/models/lantern.mtl', import.meta.url).href

export function Lantern({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const { groupRef, obj } = useModel({
    objUrl: lanternObjUrl,
    mtlUrl: lanternMtlUrl,
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

      {/* Lantern model */}
      <primitive object={obj} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}
