import React from 'react'
import { useModel } from './hooks/useModel'

// Import the model URLs using Parcel's URL import
export const treeObjUrl = new URL('../assets/models/tree.obj', import.meta.url)
  .href
export const treeMtlUrl = new URL('../assets/models/tree.mtl', import.meta.url)
  .href

export const treeSnowObjUrl = new URL('../assets/models/tree-snow-a.obj', import.meta.url)
  .href
export const treeSnowMtlUrl = new URL('../assets/models/tree-snow-a.mtl', import.meta.url)
  .href

export function Tree({ position = [0, 0, 0], scale = 1, debug = false, snow = false }) {
  const { groupRef, obj } = useModel({
    objUrl: snow ? treeSnowObjUrl : treeObjUrl,
    mtlUrl: snow ? treeSnowMtlUrl : treeMtlUrl,
    debug,
  })

  return (
    <group ref={groupRef} position={position}>
      {/* Debug cube, only shown if debug is true */}
      {debug && (
        <mesh position={[0, 0.5, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Tree model */}
      <primitive object={obj} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}
