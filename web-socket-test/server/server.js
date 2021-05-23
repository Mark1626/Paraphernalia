const express = require("express");
const WebSocket = require("ws");

const app = express();
const wss = new WebSocket.Server({ noServer: true })

wss.on('connection', (ws) => {
  console.log(`Connection Created with client`)
  ws.on('message', (message) => {
    console.log(`Received message ${message}`)
    ws.send("Hello Client")
  })

  ws.on('close', () => {
    console.log(`Client disconnected, shutting down`)
  })
})

app.get("/api/hello", (req, res) => {
  res.json({msg: "Hello World"})
});

const port = process.env.PORT || 3001
const listener = app.listen(port, () => {
  console.log(`Server listening on ${listener.address().port}`)
})

listener.on('upgrade', (req, soc, head) => {
  console.log("Handing WS connections")
  wss.handleUpgrade(req, soc, head, (socket) => {
    wss.emit('connection', socket, req);
  })
})
