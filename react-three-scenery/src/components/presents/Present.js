import React from 'react'
import { useModel } from '../hooks/useModel'

export function Present({
  objUrl,
  mtlUrl,
  position = [0, 0, 0],
  scale = 1,
  debug = false,
}) {
  const { groupRef, obj } = useModel({ objUrl, mtlUrl, debug })

  return (
    <group ref={groupRef} position={position}>
      {/* Debug cube, only shown if debug is true */}
      {debug && (
        <mesh position={[0, 0.5, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Present model */}
      <primitive object={obj.clone()} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}
