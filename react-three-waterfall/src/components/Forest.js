import React from 'react'
import Tree from './Tree'

export default () => {
  return (
    <group>
      {/* left */}
      <Tree position={[-1.75, 0, -.85]} />
      <Tree position={[-1.75, 0, -.15]} />
      <Tree position={[-1.5, 0,-.5]} />
      <Tree position={[-1.5, 0,.4]} />
      <Tree position={[-1.25, 0,-.85]} />
      <Tree position={[-1.25, 0,.75]} />
      <Tree position={[-.75, 0,-.85]} />
      <Tree position={[-.75, 0,-.25]} />
      <Tree position={[-.25, 0,-.85]} />
      {/* right */}
      <Tree position={[1.25, 0,-.85]} />
      <Tree position={[1.25, 0,.75]} />
      <Tree position={[1.5, 0,-.5]} />
      <Tree position={[1.75, 0,-.85]} />
      <Tree position={[1.75, 0,.35]} />
    </group>
  )
}
