import React from 'react'
import { Grid, Text } from '@react-three/drei'
import * as THREE from 'three'

function GridNumber({ position, number }) {
  return (
    <Text
      position={position}
      rotation={[-Math.PI / 2, 0, 0]}
      fontSize={0.3}
      color="white"
      anchorX="center"
      anchorY="middle"
    >
      {number}
    </Text>
  )
}

export function DebugGrid({ size = 10, divisions = 10 }) {
  const numbers = React.useMemo(() => {
    const arr = []
    const step = size / divisions
    const start = -size / 2
    const end = size / 2

    // Generate numbers for both X and Z axes
    for (let i = start; i <= end; i += step) {
      // X-axis numbers
      arr.push(
        <GridNumber
          key={`x${i}`}
          position={[i, 0.01, -size / 2 - 0.3]}
          number={i.toFixed(1)}
        />
      )
      // Z-axis numbers
      arr.push(
        <GridNumber
          key={`z${i}`}
          position={[-size / 2 - 0.3, 0.01, i]}
          number={i.toFixed(1)}
          rotation={[-Math.PI / 2, 0, Math.PI / 2]}
        />
      )
    }
    return arr
  }, [size, divisions])

  return (
    <group>
      {/* Main grid */}
      <Grid
        args={[size, size, divisions, divisions]}
        position={[0, 0.01, 0]}
        cellColor="white"
        sectionColor="red"
        infiniteGrid
        fadeDistance={50}
        fadeStrength={1}
      />

      {/* Axis indicators */}
      <group position={[0, 0.02, 0]}>
        {/* X-axis (red) */}
        <mesh position={[size / 4, 0, 0]}>
          <boxGeometry args={[size / 2, 0.02, 0.02]} />
          <meshBasicMaterial color="red" />
        </mesh>
        <Text
          position={[size / 2 + 0.3, 0, 0]}
          rotation={[-Math.PI / 2, 0, 0]}
          fontSize={0.3}
          color="red"
        >
          X
        </Text>

        {/* Z-axis (blue) */}
        <mesh position={[0, 0, size / 4]}>
          <boxGeometry args={[0.02, 0.02, size / 2]} />
          <meshBasicMaterial color="blue" />
        </mesh>
        <Text
          position={[0, 0, size / 2 + 0.3]}
          rotation={[-Math.PI / 2, 0, 0]}
          fontSize={0.3}
          color="blue"
        >
          Z
        </Text>

        {/* Origin indicator */}
        <mesh position={[0, 0, 0]}>
          <sphereGeometry args={[0.05]} />
          <meshBasicMaterial color="yellow" />
        </mesh>
        <Text
          position={[0, 0.3, 0]}
          rotation={[-Math.PI / 2, 0, 0]}
          fontSize={0.3}
          color="yellow"
        >
          (0,0,0)
        </Text>
      </group>

      {/* Grid numbers */}
      {numbers}
    </group>
  )
}
