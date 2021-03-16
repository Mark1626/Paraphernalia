// http://www.dirk-loss.de/sshvis/drunken_bishop.pdf
const crypto = require("crypto");
const process = require("process")

const location = (pos) => {
  if (pos == 0) return "a";
  if (pos == 16) return "b";
  if (pos == 136) return "c";
  if (pos == 152) return "d";
  if (pos > 0 && pos < 16) return "T";
  if (pos > 136 && pos < 152) return "B";
  if (pos % 17 == 0) return "L";
  if (pos % 17 == 16) return "R";
  return "M";
};

const getNewPos = (pos, mov) => {
  switch (location(pos)) {
    case "M":
      if (mov == 0) return pos - 18;
      if (mov == 1) return pos - 16;
      if (mov == 2) return pos + 16;
      if (mov == 3) return pos + 18;
    case "T":
      if (mov == 0) return pos - 1;
      if (mov == 1) return pos + 1;
      if (mov == 2) return pos + 16;
      if (mov == 3) return pos + 18;
    case "B":
      if (mov == 0) return pos - 18;
      if (mov == 1) return pos - 16;
      if (mov == 2) return pos - 1;
      if (mov == 3) return pos + 1;
    case "L":
      if (mov == 0) return pos - 17;
      if (mov == 1) return pos - 16;
      if (mov == 2) return pos + 17;
      if (mov == 3) return pos + 18;
    case "R":
      if (mov == 0) return pos - 18;
      if (mov == 1) return pos - 17;
      if (mov == 2) return pos + 16;
      if (mov == 3) return pos + 17;
    case "a":
      if (mov == 0) return pos;
      if (mov == 1) return pos + 1;
      if (mov == 2) return pos + 17;
      if (mov == 3) return pos + 18;
    case "b":
      if (mov == 0) return pos - 1;
      if (mov == 1) return pos;
      if (mov == 2) return pos + 16;
      if (mov == 3) return pos + 17;
    case "c":
      if (mov == 0) return pos - 17;
      if (mov == 1) return pos - 16;
      if (mov == 2) return pos;
      if (mov == 3) return pos + 1;
    case "d":
      if (mov == 0) return pos - 18;
      if (mov == 1) return pos - 17;
      if (mov == 2) return pos - 1;
      if (mov == 3) return pos;
  }
};

const updateBoard = ({ pos, mov, board }) => {
  const newBoard = { ...board };
  const newPos = getNewPos(pos, mov);

  newBoard[newPos] = newBoard[newPos] ? newBoard[newPos] + 1 : 1;

  return { pos: newPos, board: newBoard };
};

const printBoard = (board) => {
  console.log('+-----------------+')
  for (let j = 0; j < 9; j++) {
    let row = `|`;
    for (let i = 0; i < 17; i++) {
      let idx = j * 17 + i;
      row = row.concat(board[idx] ? " .o+=*BOX@%&#/^SE"[board[idx]] : " ");
    }
    row = row.concat('|')
    console.log(row);
  }
  console.log('+-----------------+')
};

const digest = crypto.createHash("MD5");
const msg = process.argv[2]
// const msg = "hello"

digest.update(msg);
const val = digest.digest();
// const val = Buffer.from([0xfc, 0x94, 0xb0, 0xc1, 0xe5, 0xb0, 0x98, 0x7c, 0x58, 0x43, 0x99, 0x76, 0x97, 0xee, 0x9f, 0xb7])
// const val = Buffer.from([0x73, 0x1e, 0xe5, 0x4c, 0x82, 0x23, 0x33, 0x59, 0xe3, 0xd5, 0xe9, 0xf6, 0xcc, 0xf8, 0x7e, 0x1f])

console.log(`Drunken Bishop path for ${msg}, ${val.toString("hex")}`)
let state = { pos: 76, board: {} };
val.forEach((byte) => {
  const a = byte & 0x03;
  const b = (byte >> 2) & 0x03;
  const c = (byte >> 4) & 0x03;
  const d = (byte >> 6) & 0x03;

  state = updateBoard({ ...state, mov: a });
  state = updateBoard({ ...state, mov: b });
  state = updateBoard({ ...state, mov: c });
  state = updateBoard({ ...state, mov: d });
});

state.board[76] = 15
state.board[state.pos] = 16
// console.log(state.board);

printBoard(state.board);
