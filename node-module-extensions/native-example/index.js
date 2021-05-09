require("./loader")

const Cube = require("../cube")

const cb = new Cube(4)
console.log(cb.area())

require("../fn")

const data = require("./test.yaml")

console.dir(data);
console.log(`Author: ${data.author}`)
console.log(`Server: ${data.server.ports[0].targetPort}`)

const port = data.server.ports[0].targetPort || 3000;
const http = require('http');

const requestListener = function (req, res) {
  res.writeHead(200);
  res.end('Hello, World!');
}

const server = http.createServer(requestListener);
server.listen(port);
