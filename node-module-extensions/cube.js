class Cube {
  constructor(side) {
    $.side = side
  }

  area() {
    return $.side * $.side
  }
}

module.exports = Cube
