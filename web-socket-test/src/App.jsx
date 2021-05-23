import React, { useEffect, useRef, useState } from 'react'
import './App.css'

function App() {
  const [disabled, setDisabled] = useState(true);
  const ws = useRef(null)

  useEffect(() => {
    ws.current = new WebSocket("ws://localhost:3001")
    ws.current.onopen = () => {
      console.log("Connection Created, sending message")
      ws.current.send("Hello Server")
    }
    ws.current.onclose = () => console.log("ws closed");

    return () => {
      ws.current.close()
    }
  }, [])

  useEffect(() => {
    if (!ws.current) return

    ws.current.onmessage = e => {
      if (!disabled) return
      const message = e.data
      console.log(`Message ${message}`)
    }
  }, [disabled])

  return (
    <div className="App">
      <button onClick={() => {
        setDisabled(!disabled)
      }}>
        {disabled ? <h1>Disabled</h1> : <h1>Enabled</h1> }
      </button>
    </div>
  )
}

export default App
