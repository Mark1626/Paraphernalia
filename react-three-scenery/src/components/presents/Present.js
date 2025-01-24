import React, { useRef } from 'react'
import { useModel } from '../hooks/useModel'
import { useLOD } from '../hooks/useLOD'

// Simple present for LOD
function SimplePresent() {
  return (
    <group>
      <mesh position={[0, 0.15, 0]}>
        <boxGeometry args={[0.3, 0.3, 0.3]} />
        <meshBasicMaterial color="#ff4444" />
      </mesh>
      <mesh position={[0, 0.3, 0]}>
        <boxGeometry args={[0.32, 0.02, 0.32]} />
        <meshBasicMaterial color="#ffdd44" />
      </mesh>
      <mesh position={[0, 0.15, 0]} rotation={[0, 0, Math.PI / 2]}>
        <boxGeometry args={[0.3, 0.02, 0.02]} />
        <meshBasicMaterial color="#ffdd44" />
      </mesh>
      <mesh position={[0, 0.15, 0]}>
        <boxGeometry args={[0.02, 0.02, 0.3]} />
        <meshBasicMaterial color="#ffdd44" />
      </mesh>
    </group>
  )
}

// Very simple present for far distances
function VerySimplePresent() {
  return (
    <mesh position={[0, 0.15, 0]}>
      <boxGeometry args={[0.3, 0.3, 0.3]} />
      <meshBasicMaterial color="#ff4444" />
    </mesh>
  )
}

export function Present({ objUrl, mtlUrl, position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  const groupRef = useRef()
  const lodLevel = useLOD(groupRef)
  
  const { groupRef: modelRef, obj } = useModel({
    objUrl,
    mtlUrl,
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
      {lodLevel === 1 && <SimplePresent />}
      {lodLevel === 2 && <VerySimplePresent />}
    </group>
  )
}
