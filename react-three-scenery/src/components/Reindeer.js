import React from 'react'
import { useModel } from './hooks/useModel'

// Import model URLs
const reindeerObj = new URL('../assets/models/reindeer.obj', import.meta.url).href
const reindeerMtl = new URL('../assets/models/reindeer.mtl', import.meta.url).href

export function Reindeer({
  position = [0, 0, 0],
  rotation = [0, 0, 0],
  scale = 1,
  debug = false,
}) {
  const { groupRef, obj } = useModel({
    objUrl: reindeerObj,
    mtlUrl: reindeerMtl,
    debug,
  })

  return (
    <group ref={groupRef} position={position} rotation={rotation}>
      {/* Debug cube */}
      {debug && (
        <mesh position={[0, 1, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Reindeer model */}
      <primitive object={obj.clone()} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}
