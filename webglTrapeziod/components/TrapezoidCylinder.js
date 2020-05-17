import React from 'react'

export default ({ position }) => {

  
  return (
    <mesh position={position}>
      <cylinderGeometry attach="geometry" args={[0.4 / Math.SQRT2, 1 / Math.SQRT2, 4]} />
      <meshLambertMaterial attach="material" wireframe color="#ffffff" />
    </mesh>
  )
}
