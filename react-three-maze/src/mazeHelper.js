/**
 * @param {number} x
 * @param {number} y
 */
function maze(x, y) {
  let n = x * y - 1;
  /** @type boolean[][] */
  let horiz = [];
  /** @type boolean[][] */
  let verti = [];
  let here = [Math.floor(Math.random() * x), Math.floor(Math.random() * y)]
  let path = [here];
  /** @type boolean[][] */
  let unvisited = []

  for (var j = 0; j < x + 1; j++) {
    horiz[j] = [];
    verti[j] = []
  }
  for (var j = 0; j < x + 2; j++) {
    unvisited[j] = [];
    for (var k = 0; k < y + 1; k++) {
      unvisited[j].push(
        j > 0 && j < x + 1 && k > 0 && (j != here[0] + 1 || k != here[1] + 1)
      );
    }
  }
  while (0 < n) {
    var potential = [
      [here[0] + 1, here[1]],
      [here[0], here[1] + 1],
      [here[0] - 1, here[1]],
      [here[0], here[1] - 1],
    ];
    var neighbors = [];
    for (var j = 0; j < 4; j++)
      if (unvisited[potential[j][0] + 1][potential[j][1] + 1])
        neighbors.push(potential[j]);
    if (neighbors.length) {
      n = n - 1;
      const next = neighbors[Math.floor(Math.random() * neighbors.length)];
      unvisited[next[0] + 1][next[1] + 1] = false;
      if (next[0] == here[0])
        horiz[next[0]][(next[1] + here[1] - 1) / 2] = true;
      else verti[(next[0] + here[0] - 1) / 2][next[1]] = true;
      path.push((here = next));
    // @ts-ignore
    } else here = path.pop();
  }
  return { x: x, y: y, horiz: horiz, verti: verti };
}

/**
 * "0" Empty Space
 * "1" Vertical Column
 * "2" Horizantal Column
 * "3" Pillar
 * @param {{ x: number; y: number; horiz: boolean[][]; verti: boolean[][]; }} maze
 */
function convertToCoordinates({x, y, verti, horiz}) {
  let maze = [];
  let width;
  // Two separate columns for vertical and horizontal columns
  for (var j = 0; j < x * 2 + 1; j++) {
    var row = [];
    if (0 == j % 2) {
      for (var k = 0; k < y + 1; k++) {
        row.push(3);
        if (j > 0 && verti[j / 2 - 1][k]) {
          row.push(0);
        } else {
          row.push(2);
        }
      }
    } else {
      for (var k = 0; k < y + 1; k++) {
        if (k > 0 && horiz[(j - 1) / 2][k - 1]) {
          row.push(0);
        } else {
          row.push(1);
        }
        row.push(0);
      }
    }
    //Delete the last unnecessary element
    row.pop();
    // Add entry
    if (0 == j) row[1] = 0;
    // Add exit
    if (x * 2 - 1 == j) row[row.length - 1] = 0;
    maze.push(row);

    if (j == 0) width = row.length;
  }
  return {width, height: maze.length, maze};
}

// const m = maze(3, 2)
// console.log(m)
// console.log(convertToCoordinates(maze(3, 2)))

// /**
//  * @param {{x: number, y:number}} param
//  */
export default ({ x, y }) => convertToCoordinates(maze(x, y));
