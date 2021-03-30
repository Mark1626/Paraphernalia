import React from 'react'
import {Colors} from "../constants/color"

export default ({ position }) => {

  const verticalGeometryRail = (
    <boxGeometry attach="geometry" args={[.04,.3,.04]} />
  );
  const horizontalGeometryRail = (
    <boxGeometry attach="geometry" args={[1.2,.04,.04 ]} />
  );

  const woodBlockGeometry = (
    <boxGeometry attach="geometry" args={[.15, .02, .4]} />
  )

  return (
    <group>
      {/* Wood block */}
      {Array.from({length: 6}).map((row, i) =>
        <mesh key={`block-${i}`} position={[0+.2*i,.21,.2]} receiveShadow castShadow>
          {woodBlockGeometry}
          <meshLambertMaterial attach="material" color={Colors.brown} />
        </mesh>
      )}
      {/* Wood rail */}
      <mesh position={[-.1,.35,.4]} castShadow>
        {verticalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>

      <mesh position={[1.1,.35,.4]} castShadow>
        {verticalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>

      <mesh position={[-.1,.35,0]} castShadow>
        {verticalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>

      <mesh position={[1.1,.35,0]} castShadow>
        {verticalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>

      <mesh position={[0.5,.42,.4]} castShadow>
        {horizontalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>

      <mesh position={[0.5,.42,0]} castShadow>
        {horizontalGeometryRail}
        <meshLambertMaterial attach="material" color={Colors.brown} />
      </mesh>
      
    </group>
  );
}
