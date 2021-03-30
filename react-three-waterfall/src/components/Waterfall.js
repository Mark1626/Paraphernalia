import React from 'react'
import Bridge from './Bridge'
import GrassLand from './GrassLand'
import Forest from './Forest'
import River from './River'

export default ({position}) => {
  return (
    <group>
      <GrassLand />
      <River />
      <Forest />
      <Bridge />

    </group>
  )
}
