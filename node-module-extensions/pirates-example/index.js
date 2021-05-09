const addHook = require("pirates").addHook

const revert = addHook(
  (code, fileName) => {
    code = code.replace(/\$/g, 'this')
    code = code.replace(/fn \(/g, 'function (')
    return code
  },
  { exts: '.js', matcher: () => true }
)

const Cube = require("../cube")
const cb = new Cube(4)
console.log(cb.area())

require("../fn")

// Revert hook change
revert()

