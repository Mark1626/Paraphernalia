import { useState } from 'react'
import { useFrame, useThree } from '@react-three/fiber'

export function useLOD(ref) {
  const { camera } = useThree()
  const [lodLevel, setLodLevel] = useState(0)

  useFrame(() => {
    if (!ref.current) return
    
    const distance = camera.position.distanceTo(ref.current.position)
    
    // Update LOD based on distance - using much smaller thresholds
    if (distance > 8) {
      setLodLevel(2) // Very simple
    } else if (distance > 3) {
      setLodLevel(1) // Simple
    } else {
      setLodLevel(0) // Detailed
    }
  })

  return lodLevel
}
