import React from 'react'

export default ({position, geometry, alpha}) => {
  return (
    <mesh position={position} receiveShadow>
      {geometry}
      <shadowMaterial opacity={alpha} />
    </mesh>
  )
}
