import React, { useRef, useEffect } from 'react'
import { useFrame, useThree } from '@react-three/fiber'

// Component to wrap models with LOD (Level of Detail)
export function LODModel({ children, distances = [0, 5, 15], ...props }) {
  const groupRef = useRef()
  const { camera } = useThree()
  const [activeIndex, setActiveIndex] = React.useState(0)

  useFrame(() => {
    if (!groupRef.current) return

    // Calculate distance to camera
    const distance = camera.position.distanceTo(groupRef.current.position)

    // Find appropriate LOD level
    let newIndex = distances.findIndex((d, i) => {
      const nextDistance = distances[i + 1]
      return nextDistance === undefined || distance < nextDistance
    })

    if (newIndex !== activeIndex) {
      setActiveIndex(newIndex)
    }
  })

  // Only render the active LOD level
  const childArray = React.Children.toArray(children)
  const activeChild = childArray[activeIndex] || childArray[childArray.length - 1]

  return (
    <group ref={groupRef} {...props}>
      {activeChild}
    </group>
  )
}
