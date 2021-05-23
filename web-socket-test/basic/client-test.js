const WebSocket = require("ws")
const ws = new WebSocket("ws://localhost:8080")

ws.on('open', () => {
  ws.send("Hello");
  ws.close()
})

ws.on('message', (msg) => {
  console.log(msg)
})
