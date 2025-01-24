import React from 'react'
import { useModel } from '../hooks/useModel'

// Import model URLs
const rocksLargeObj = new URL('../../assets/models/rocks-large.obj', import.meta.url).href
const rocksLargeMtl = new URL('../../assets/models/rocks-large.mtl', import.meta.url).href

const rocksMediumObj = new URL('../../assets/models/rocks-medium.obj', import.meta.url).href
const rocksMediumMtl = new URL('../../assets/models/rocks-medium.mtl', import.meta.url).href

const rocksSmallObj = new URL('../../assets/models/rocks-small.obj', import.meta.url).href
const rocksSmallMtl = new URL('../../assets/models/rocks-small.mtl', import.meta.url).href

function Rock({ objUrl, mtlUrl, position = [0, 0, 0], scale = 1, debug = false }) {
  const { groupRef, obj } = useModel({ objUrl, mtlUrl, debug })

  return (
    <group ref={groupRef} position={position}>
      {/* Debug cube */}
      {debug && (
        <mesh position={[0, 0.5, 0]} scale={0.2}>
          <boxGeometry />
          <meshStandardMaterial color="red" />
        </mesh>
      )}

      {/* Rock model */}
      <primitive object={obj} scale={scale} position={[0, 0, 0]} />
    </group>
  )
}

export function LargeRocks(props) {
  return <Rock objUrl={rocksLargeObj} mtlUrl={rocksLargeMtl} {...props} />
}

export function MediumRocks(props) {
  return <Rock objUrl={rocksMediumObj} mtlUrl={rocksMediumMtl} {...props} />
}

export function SmallRocks(props) {
  return <Rock objUrl={rocksSmallObj} mtlUrl={rocksSmallMtl} {...props} />
}
