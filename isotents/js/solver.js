/**
 * Tents & Trees solver / verifier.
 *
 * A board is the immutable puzzle definition:
 *   { size, trees: boolean[size][size], rowClues: number[], colClues: number[] }
 *
 * A "house" layout is valid when:
 *   1. no two houses are 8-adjacent (touching, even diagonally);
 *   2. the house count of each row/column equals its clue;
 *   3. there is a perfect matching between trees and houses where each tree is
 *      orthogonally adjacent to its own house (rule 3 from PLAN.md).
 *
 * Because clues are derived from a real layout, #houses === #trees for any board
 * we generate, so rule 3 is a *perfect* bipartite matching.
 */

/** N=row-1, S=row+1, E=col+1, W=col-1 — orthogonal neighbour deltas. */
const ORTHO = [[-1, 0], [1, 0], [0, -1], [0, 1]];

/**
 * Does a perfect matching exist between every tree and a distinct orthogonally
 * adjacent house? Kuhn's augmenting-path algorithm.
 *
 * @param {number} size
 * @param {boolean[][]} trees
 * @param {boolean[][]} houses
 * @returns {boolean}
 */
export function hasPerfectMatching(size, trees, houses) {
  // Index houses; build the tree→adjacent-houses adjacency lists.
  const houseId = new Int32Array(size * size).fill(-1);
  let houseCount = 0;
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (houses[r][c]) houseId[r * size + c] = houseCount++;
    }
  }

  const treeAdj = [];
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (!trees[r][c]) continue;
      const adj = [];
      for (const [dr, dc] of ORTHO) {
        const nr = r + dr, nc = c + dc;
        if (nr < 0 || nr >= size || nc < 0 || nc >= size) continue;
        if (houses[nr][nc]) adj.push(houseId[nr * size + nc]);
      }
      treeAdj.push(adj);
    }
  }

  // Equal cardinality is necessary for a perfect matching.
  if (treeAdj.length !== houseCount) return false;

  const matchedTreeOf = new Int32Array(houseCount).fill(-1);
  const seen = new Uint8Array(houseCount);

  const augment = (tree) => {
    for (const h of treeAdj[tree]) {
      if (seen[h]) continue;
      seen[h] = 1;
      if (matchedTreeOf[h] === -1 || augment(matchedTreeOf[h])) {
        matchedTreeOf[h] = tree;
        return true;
      }
    }
    return false;
  };

  for (let t = 0; t < treeAdj.length; t++) {
    seen.fill(0);
    if (!augment(t)) return false;
  }
  return true;
}

/**
 * Count valid house layouts for a board, stopping once `limit` are found.
 * Use limit=2 to test uniqueness cheaply.
 *
 * Backtracking in row-major order with pruning:
 *   - never exceed a row/column clue;
 *   - keep each row/column clue reachable with the cells that remain;
 *   - no house 8-adjacent to an already-placed house;
 *   - every house must touch a tree orthogonally (else it can't be matched).
 * A completed layout is confirmed with hasPerfectMatching().
 *
 * @param {{size:number, trees:boolean[][], rowClues:number[], colClues:number[]}} board
 * @param {number} [limit=2]
 * @param {number} [nodeBudget=5_000_000] abort guard; returns count so far.
 * @returns {number} number of solutions found (capped at limit)
 */
export function countSolutions(board, limit = 2, nodeBudget = 5_000_000) {
  const { size, trees, rowClues, colClues } = board;
  const N = size;

  // Static helpers ------------------------------------------------------------
  // touchesTree[r][c]: cell is orthogonally adjacent to at least one tree.
  const touchesTree = makeGrid(N, false);
  for (let r = 0; r < N; r++) {
    for (let c = 0; c < N; c++) {
      for (const [dr, dc] of ORTHO) {
        const nr = r + dr, nc = c + dc;
        if (nr >= 0 && nr < N && nc >= 0 && nc < N && trees[nr][nc]) {
          touchesTree[r][c] = true;
          break;
        }
      }
    }
  }

  // rowRemain[i] / colRemain[i]: count of non-tree cells from this cell onward
  // (inclusive) in its row / column. Drives reachability pruning.
  const rowRemain = new Int32Array(N * N);
  const colRemain = new Int32Array(N * N);
  for (let r = 0; r < N; r++) {
    let acc = 0;
    for (let c = N - 1; c >= 0; c--) {
      if (!trees[r][c]) acc++;
      rowRemain[r * N + c] = acc;
    }
  }
  for (let c = 0; c < N; c++) {
    let acc = 0;
    for (let r = N - 1; r >= 0; r--) {
      if (!trees[r][c]) acc++;
      colRemain[r * N + c] = acc;
    }
  }

  // Mutable search state ------------------------------------------------------
  const houses = makeGrid(N, false);
  const rowCount = new Int32Array(N);
  const colCount = new Int32Array(N);
  let solutions = 0;
  let nodes = 0;

  const adjacentHouse = (r, c) => {
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        if (dr === 0 && dc === 0) continue;
        const nr = r + dr, nc = c + dc;
        if (nr >= 0 && nr < N && nc >= 0 && nc < N && houses[nr][nc]) return true;
      }
    }
    return false;
  };

  const recurse = (i) => {
    if (solutions >= limit || nodes > nodeBudget) return;
    nodes++;

    if (i === N * N) {
      if (hasPerfectMatching(N, trees, houses)) solutions++;
      return;
    }

    const r = (i / N) | 0;
    const c = i % N;

    if (trees[r][c]) {
      recurse(i + 1);
      return;
    }

    // Option A: place a house here.
    if (
      rowCount[r] < rowClues[r] &&
      colCount[c] < colClues[c] &&
      touchesTree[r][c] &&
      !adjacentHouse(r, c)
    ) {
      houses[r][c] = true;
      rowCount[r]++;
      colCount[c]++;
      recurse(i + 1);
      houses[r][c] = false;
      rowCount[r]--;
      colCount[c]--;
      if (solutions >= limit || nodes > nodeBudget) return;
    }

    // Option B: leave it empty — only if both clues stay reachable without it.
    if (
      rowCount[r] + (rowRemain[i] - 1) >= rowClues[r] &&
      colCount[c] + (colRemain[i] - 1) >= colClues[c]
    ) {
      recurse(i + 1);
    }
  };

  recurse(0);
  return solutions;
}

/** Allocate a size×size grid filled with `fill`. */
export function makeGrid(size, fill = false) {
  const g = new Array(size);
  for (let r = 0; r < size; r++) g[r] = new Array(size).fill(fill);
  return g;
}
