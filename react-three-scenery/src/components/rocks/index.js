import React, { useRef } from 'react'
import { useModel } from '../hooks/useModel'
import { useLOD } from '../hooks/useLOD'
import { useThree } from '@react-three/fiber'

// Import the model URLs using Parcel's URL import
export const largeRocksObjUrl = new URL('../../assets/models/rocks-large.obj', import.meta.url).href
export const largeRocksMtlUrl = new URL('../../assets/models/rocks-large.mtl', import.meta.url).href

export const mediumRocksObjUrl = new URL('../../assets/models/rocks-medium.obj', import.meta.url).href
export const mediumRocksMtlUrl = new URL('../../assets/models/rocks-medium.mtl', import.meta.url).href

export const smallRocksObjUrl = new URL('../../assets/models/rocks-small.obj', import.meta.url).href
export const smallRocksMtlUrl = new URL('../../assets/models/rocks-small.mtl', import.meta.url).href

// Simple rocks for LOD
function SimpleRocks({ size = 'large' }) {
  const dimensions = {
    large: { width: 1.2, height: 0.6, depth: 1.2 },
    medium: { width: 0.8, height: 0.4, depth: 0.8 },
    small: { width: 0.6, height: 0.3, depth: 0.6 }
  }[size]

  return (
    <mesh position={[0, dimensions.height / 2, 0]}>
      <boxGeometry args={[dimensions.width, dimensions.height, dimensions.depth]} />
      <meshBasicMaterial color="gray" />
    </mesh>
  )
}

// Very simple rocks for far distances
function VerySimpleRocks({ size = 'large' }) {
  const dimensions = {
    large: { width: 1, height: 0.4, depth: 1 },
    medium: { width: 0.6, height: 0.3, depth: 0.6 },
    small: { width: 0.4, height: 0.2, depth: 0.4 }
  }[size]

  return (
    <mesh position={[0, dimensions.height / 2, 0]}>
      <boxGeometry args={[dimensions.width, dimensions.height, dimensions.depth]} />
      <meshBasicMaterial color="gray" />
    </mesh>
  )
}

// Base rock component with LOD
function RockBase({ objUrl, mtlUrl, position, rotation, scale, debug, size }) {
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
      {lodLevel === 1 && <SimpleRocks size={size} />}
      {lodLevel === 2 && <VerySimpleRocks size={size} />}
    </group>
  )
}

// Export rock components with different sizes
export function LargeRocks({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  return (
    <RockBase
      objUrl={largeRocksObjUrl}
      mtlUrl={largeRocksMtlUrl}
      position={position}
      rotation={rotation}
      scale={scale}
      debug={debug}
      size="large"
    />
  )
}

export function MediumRocks({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  return (
    <RockBase
      objUrl={mediumRocksObjUrl}
      mtlUrl={mediumRocksMtlUrl}
      position={position}
      rotation={rotation}
      scale={scale}
      debug={debug}
      size="medium"
    />
  )
}

export function SmallRocks({ position = [0, 0, 0], rotation = [0, 0, 0], scale = 1, debug = false }) {
  return (
    <RockBase
      objUrl={smallRocksObjUrl}
      mtlUrl={smallRocksMtlUrl}
      position={position}
      rotation={rotation}
      scale={scale}
      debug={debug}
      size="small"
    />
  )
}
