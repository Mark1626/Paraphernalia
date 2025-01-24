import React, { useRef } from 'react'
import { useModel } from './hooks/useModel'
import { useLOD } from './hooks/useLOD'


// Import the model URLs using Parcel's URL import
export const treeObjUrl = new URL('../assets/models/tree.obj', import.meta.url)
  .href
export const treeMtlUrl = new URL('../assets/models/tree.mtl', import.meta.url)
  .href

export const treeSnowObjUrl = new URL('../assets/models/tree-snow-a.obj', import.meta.url)
  .href
export const treeSnowMtlUrl = new URL('../assets/models/tree-snow-a.mtl', import.meta.url)
  .href

// Simple tree for LOD
function SimpleTree() {
  return (
    <group>
      <mesh position={[0, 0.75, 0]}>
        <cylinderGeometry args={[0, 0.5, 1.5, 4]} />
        <meshBasicMaterial color="darkgreen" />
      </mesh>
      <mesh position={[0, 0.1, 0]}>
        <cylinderGeometry args={[0.1, 0.1, 0.2, 4]} />
        <meshBasicMaterial color="brown" />
      </mesh>
    </group>
  )
}

// Very simple tree for far distances
function VerySimpleTree() {
  return (
    <mesh position={[0, 0.5, 0]}>
      <coneGeometry args={[0.3, 1, 4]} />
      <meshBasicMaterial color="darkgreen" />
    </mesh>
  )
}

export function Tree({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false, snow = false }) {
  const groupRef = useRef()
  const lodLevel = useLOD(groupRef)
  
  const { groupRef: modelRef, obj } = useModel({
    objUrl: snow ? treeSnowObjUrl : treeObjUrl,
    mtlUrl: snow ? treeSnowMtlUrl : treeMtlUrl,
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
      {lodLevel === 1 && <SimpleTree />}
      {lodLevel === 2 && <VerySimpleTree />}
    </group>
  )
}
