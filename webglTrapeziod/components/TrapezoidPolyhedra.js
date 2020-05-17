import React from 'react'

export default ({ position }) => {

  var verticesOfCube = [
    -1, -1, -1.2, 1, -1.2, -1, 1.6, 2.5, -1, -1, 1, -1,
    -1, -1, 1, 1, -1, 1, 1, 1, 1, -1, 1, 1,
  ];

  var indicesOfFaces = [
    2, 1, 0, 0, 3, 2,
    0, 4, 7, 7, 3, 0,
    0, 1, 5, 5, 4, 0,
    1, 2, 6, 6, 5, 1,
    2, 3, 7, 7, 6, 2,
    4, 5, 6, 6, 7, 4
  ];

  return (
    <mesh position={position}>
      <polyhedronBufferGeometry attach="geometry" args={[verticesOfCube, indicesOfFaces, 1, 0]} />
      <meshLambertMaterial attach="material" wireframe color="#ffffff" />
    </mesh>
  )
}
