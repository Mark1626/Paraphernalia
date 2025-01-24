import React from 'react'
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader'
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader'
import { useLoader } from '@react-three/fiber'
import { useHelper } from '@react-three/drei'
import * as THREE from 'three'
import { textureUrl } from '../constants'

export function useModel({ objUrl, mtlUrl, debug }) {
  const groupRef = React.useRef()

  // Load texture and materials
  const texture = useLoader(THREE.TextureLoader, textureUrl)
  const materials = useLoader(MTLLoader, mtlUrl)
  const originalObj = useLoader(OBJLoader, objUrl, loader => {
    materials.preload()
    loader.setMaterials(materials)
  })

  // Clone the object and its materials
  const obj = React.useMemo(() => {
    const clonedObj = originalObj.clone(true)
    clonedObj.traverse(child => {
      if (child.isMesh) {
        child.castShadow = true
        child.receiveShadow = true
        if (child.material) {
          // Clone the material for each instance
          child.material = child.material.clone()
          child.material.transparent = false
          child.material.opacity = 1
          child.material.side = THREE.DoubleSide
          child.material.map = texture
          child.material.needsUpdate = true
        }
      }
    })
    return clonedObj
  }, [originalObj, texture])

  // Only apply helper if debug is true
  if (debug) {
    useHelper(groupRef, THREE.BoxHelper, 'red')
  }

  return { groupRef, obj }
}
