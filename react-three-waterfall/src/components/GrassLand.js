import React from 'react'
import { Colors } from '../constants/color'
import CustomShadow from './CustomShadow';

export default () => {

  const positionLeft = [-1, 0.1, 0];
  const geometryLeft = <boxGeometry attach="geometry" args={[2, 0.2, 2]} />

  const positionRight = [1.5, 0.1, 0];
  const geometryRight = <boxGeometry attach="geometry" args={[1, 0.2, 2]} />

  return (
    <group>
      <mesh position={positionLeft} >
        {geometryLeft}
        <meshLambertMaterial attach="material" color={Colors.greenLight} />
      </mesh>
      <CustomShadow
        position={positionLeft}
        geometry={geometryLeft}
        alpha={0.25}
      />
      <mesh position={positionRight} >
        {geometryRight}
        <meshLambertMaterial attach="material" color={Colors.greenLight} />
      </mesh>
      <CustomShadow
        position={positionRight}
        geometry={geometryRight}
        alpha={0.25}
      />
    </group>
  )
}
