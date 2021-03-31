import React from 'react'
import { Colors } from '../constants/color'
import CustomShadow from './CustomShadow';
import Drop from './Drop';

export default () => {

  const positionRiver = [0.5,.1,0];
  const geometryRiver = <boxGeometry attach="geometry" args={[1, .1, 2]} />

  const positionRiverBed = [0.5,.025,0];
  const geometryRiverBed = <boxGeometry attach="geometry" args={[1, .05, 2]} />

  return (
    <group>
      <mesh position={positionRiver} >
        {geometryRiver}
        <meshLambertMaterial attach="material" color={Colors.blue} />
      </mesh>
      <CustomShadow
        position={positionRiver}
        geometry={geometryRiver}
        alpha={0.08}
      />
      {/* River bed */}
      <mesh position={positionRiverBed} >
        {geometryRiverBed}
        <meshLambertMaterial attach="material" color={Colors.greenLight} />
      </mesh>
      <CustomShadow
        position={positionRiverBed}
        geometry={geometryRiverBed}
        alpha={0.08}
      />
      {Array.from({length: 500}).map((a, i) => 
        <Drop
          key={`drop-${i}`}
          speed={0}
          position={[Math.random(.1,.9), -20*Math.random(),1+(Math.random()-.5)*.1]}
          lifespan={-1} />
      )}
    </group>
  )
}
