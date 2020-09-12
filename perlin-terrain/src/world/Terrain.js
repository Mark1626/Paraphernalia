import React, { useState, useEffect, useCallback } from 'react'
import { pointCount, pointGap, pointSize } from './constants'
import Point from './Point'

export default () => {
  let [points, setPoints] = useState([])

  useEffect(() => {
    let p = []
    for (let i = 0; i < pointCount; i++) {
      for (let j = 0; j < pointCount; j++) {
        p.push({
          x: i * pointGap,
          y: 0,
          z: j * pointGap,
          pointSize,
          i,
          j,
        });
      }
    }
    setPoints(p);
  }, [])

  return (
    <group>
      {points.map((p, idx) => (
        <Point key={`${idx}`} {...p} />
      ))}
    </group>
  )
}
