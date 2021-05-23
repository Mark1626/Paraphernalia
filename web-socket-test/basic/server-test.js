const WebSocket = require("ws");
wss = new WebSocket.Server({
  port: 8080
})

console.log("Creating Web Socket")
wss.on('connection', (ws) => {
  console.log(`Connection Created with client`)
  ws.on('message', (message) => {
    console.log(`Received message ${message}`)
    ws.send("Hello client")
  })

  ws.on('close', () => {
    console.log(`Client disconnected, shutting down`)
    wss.close()
  })
})
