/**
 * Procedural Tents & Trees board generator.
 *
 * Produces boards with a *unique* solution:
 *   1. scatter tents (houses) so no two are 8-adjacent;
 *   2. give each tent its own orthogonally-adjacent tree (bipartite matching);
 *   3. derive the row/column clues from the tent layout;
 *   4. accept only if the solver proves exactly one solution.
 */

import { countSolutions, makeGrid } from "./solver.js";

const ORTHO = [[-1, 0], [1, 0], [0, -1], [0, 1]];

/** Difficulty presets. `density` is the fraction of cells that become houses. */
export const LEVELS = [
  { name: "Easy", size: 6, density: 0.2 },
  { name: "Medium", size: 8, density: 0.2 },
  { name: "Hard", size: 10, density: 0.2 },
  { name: "Expert", size: 12, density: 0.2 },
  { name: "Master", size: 15, density: 0.2 },
];

/**
 * Generate a uniquely-solvable board.
 *
 * @param {number} size grid dimension N (rows = cols = N)
 * @param {number} [density=0.2] target fraction of cells that are houses
 * @param {number} [maxAttempts=400]
 * @returns {{size:number, trees:boolean[][], solution:boolean[][], rowClues:number[], colClues:number[]}}
 */
export function generatePuzzle(size, density = 0.2, maxAttempts = 400) {
  const target = Math.max(1, Math.round(size * size * density));

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    const tents = scatterTents(size, target);
    if (tents.length === 0) continue;

    const trees = assignTrees(size, tents);
    if (!trees) continue; // couldn't give every tent its own tree

    const solution = makeGrid(size, false);
    for (const [r, c] of tents) solution[r][c] = true;

    const { rowClues, colClues } = deriveClues(size, solution);
    const board = { size, trees, solution, rowClues, colClues };

    if (countSolutions(board, 2) === 1) return board;
  }

  throw new Error(
    `Failed to generate a unique ${size}x${size} puzzle in ${maxAttempts} attempts`,
  );
}

/**
 * Randomly place up to `target` tents such that no two are 8-adjacent.
 * Returns the list of [row, col] tent positions actually placed.
 */
function scatterTents(size, target) {
  const occupied = makeGrid(size, false); // tent or tent-blocked neighbourhood
  const isTent = makeGrid(size, false);
  const tents = [];

  // Shuffled list of all cells = random placement order.
  const cells = [];
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) cells.push([r, c]);
  }
  shuffle(cells);

  for (const [r, c] of cells) {
    if (tents.length >= target) break;
    if (occupied[r][c]) continue;

    isTent[r][c] = true;
    tents.push([r, c]);
    // Block the 8-neighbourhood so the next tent can't touch this one.
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        const nr = r + dr, nc = c + dc;
        if (nr >= 0 && nr < size && nc >= 0 && nc < size) occupied[nr][nc] = true;
      }
    }
  }
  return tents;
}

/**
 * Give every tent its own orthogonally-adjacent tree via bipartite matching
 * (tents ↔ candidate empty cells). Returns a trees grid, or null if no perfect
 * assignment exists.
 */
function assignTrees(size, tents) {
  const isTent = makeGrid(size, false);
  for (const [r, c] of tents) isTent[r][c] = true;

  // Candidate tree cells for each tent: orthogonal neighbours that aren't tents.
  const candidates = tents.map(([r, c]) => {
    const cand = [];
    for (const [dr, dc] of ORTHO) {
      const nr = r + dr, nc = c + dc;
      if (nr < 0 || nr >= size || nc < 0 || nc >= size) continue;
      if (!isTent[nr][nc]) cand.push(nr * size + nc);
    }
    shuffle(cand); // vary which neighbour becomes the tree
    return cand;
  });

  // Kuhn's matching: tent → distinct cell id.
  const treeOfCell = new Map(); // cellId → tent index
  const augment = (tent, seen) => {
    for (const cell of candidates[tent]) {
      if (seen.has(cell)) continue;
      seen.add(cell);
      if (!treeOfCell.has(cell) || augment(treeOfCell.get(cell), seen)) {
        treeOfCell.set(cell, tent);
        return true;
      }
    }
    return false;
  };

  for (let t = 0; t < tents.length; t++) {
    if (!augment(t, new Set())) return null;
  }

  const trees = makeGrid(size, false);
  for (const cell of treeOfCell.keys()) {
    trees[(cell / size) | 0][cell % size] = true;
  }
  return trees;
}

/** Count houses per row/column to produce the edge clues. */
function deriveClues(size, solution) {
  const rowClues = new Array(size).fill(0);
  const colClues = new Array(size).fill(0);
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (solution[r][c]) {
        rowClues[r]++;
        colClues[c]++;
      }
    }
  }
  return { rowClues, colClues };
}

/** In-place Fisher–Yates shuffle. */
function shuffle(arr) {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}
