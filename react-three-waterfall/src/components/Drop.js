import { useFrame } from '@react-three/fiber';
import React, { useRef } from 'react'
import { Colors } from '../constants/color'

export default ({lifespan, speed, position}) => {
  const mesh = useRef()
  useFrame(() => {
    if (lifespan < 0) {
      speed = 0
      lifespan = (Math.random()*50)+50
      mesh.current.position.x = Math.random(.1,.9)
      mesh.current.position.y = 0.1
      mesh.current.position.z = 1+(Math.random()-.5)*.1
    }
    
    speed += 0.007
    lifespan -= 1
    mesh.current.position.x += (0.5 - mesh.current.position.x)/70;
    mesh.current.position.y -= speed
  })
  
  return (
      <mesh ref={mesh} position={position} >
        <boxGeometry attach="geometry" args={[0.1, 0.1, 0.1]} />
        <meshLambertMaterial attach="material" color={Colors.blue} />
      </mesh>
  )
}
