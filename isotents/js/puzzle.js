/**
 * Runtime puzzle state: the active board, the player's placement grid, and
 * live validation (rule violations + win detection).
 *
 * Cell placement values (only meaningful where the board has no tree):
 *   EMPTY  — undecided
 *   HOUSE  — player placed a house
 *   MARKED — player flagged "definitely not a house"
 */

import { generatePuzzle } from "./generator.js";
import { hasPerfectMatching } from "./solver.js";

export const EMPTY = 0;
export const HOUSE = 1;
export const MARKED = 2;

const TREE_SPRITES = ["tree1", "tree2", "tree3", "tree4"];
const HOUSE_SPRITES = [
  "white_blue_house1", "white_red_house1", "white_green_house1", "wood_blue_house1",
];

/** @type {{board:object, placement:number[][], treeSprites:string[][], houseSprites:string[][], won:boolean}} */
let state = null;

/** Start a fresh puzzle of the given size / density. */
export function newPuzzle(size, density = 0.2) {
  const board = generatePuzzle(size, density);
  const placement = grid(size, EMPTY);
  // Stable per-cell sprite choices so re-renders don't flicker.
  const treeSprites = grid(size, null);
  const houseSprites = grid(size, null);
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      treeSprites[r][c] = TREE_SPRITES[(r * 7 + c * 3) % TREE_SPRITES.length];
      houseSprites[r][c] = HOUSE_SPRITES[(r * 5 + c) % HOUSE_SPRITES.length];
    }
  }
  state = { board, placement, treeSprites, houseSprites, won: false };
  return state;
}

export function getState() {
  return state;
}

/** Clear all of the player's placements (keeps the same board). */
export function reset() {
  const { size } = state.board;
  state.placement = grid(size, EMPTY);
  state.won = false;
}

// ── Player actions ───────────────────────────────────────────
// All actions ignore tree cells and re-evaluate the win condition.

/** Left-click: toggle a house on/off (overwrites a MARKED cell). */
export function toggleHouse(r, c) {
  if (!editable(r, c)) return;
  state.placement[r][c] = state.placement[r][c] === HOUSE ? EMPTY : HOUSE;
  refreshWin();
}

/** Right-click: toggle a "not a house" mark (overwrites a HOUSE cell). */
export function mark(r, c) {
  if (!editable(r, c)) return;
  state.placement[r][c] = state.placement[r][c] === MARKED ? EMPTY : MARKED;
  refreshWin();
}

/**
 * Right-drag: mark a straight run of cells as empty. Only the row OR column
 * axis is used (whichever spans more), and houses are left untouched so a drag
 * never destroys deliberate placements.
 */
export function markLine(r0, c0, r1, c1) {
  const horizontal = Math.abs(c1 - c0) >= Math.abs(r1 - r0);
  if (horizontal) {
    const [a, b] = [Math.min(c0, c1), Math.max(c0, c1)];
    for (let c = a; c <= b; c++) markEmptyCell(r0, c);
  } else {
    const [a, b] = [Math.min(r0, r1), Math.max(r0, r1)];
    for (let r = a; r <= b; r++) markEmptyCell(r, c0);
  }
  refreshWin();
}

function markEmptyCell(r, c) {
  if (editable(r, c) && state.placement[r][c] !== HOUSE) {
    state.placement[r][c] = MARKED;
  }
}

function editable(r, c) {
  return state && r >= 0 && r < state.board.size && c >= 0 && c < state.board.size &&
    !state.board.trees[r][c];
}

// ── Validation ───────────────────────────────────────────────

/**
 * Live validation for rendering and win detection.
 * @returns {{
 *   rowCounts:number[], colCounts:number[],
 *   rowStatus:string[], colStatus:string[],   // "under" | "exact" | "over"
 *   adjacent:boolean[][],                      // houses touching another house
 *   houseCount:number, won:boolean
 * }}
 */
export function validate() {
  const { board, placement } = state;
  const { size, trees, rowClues, colClues } = board;

  const rowCounts = new Array(size).fill(0);
  const colCounts = new Array(size).fill(0);
  const houses = grid(size, false);
  let houseCount = 0;

  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (placement[r][c] === HOUSE) {
        houses[r][c] = true;
        rowCounts[r]++;
        colCounts[c]++;
        houseCount++;
      }
    }
  }

  const adjacent = grid(size, false);
  let anyAdjacent = false;
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (!houses[r][c]) continue;
      for (let dr = -1; dr <= 1; dr++) {
        for (let dc = -1; dc <= 1; dc++) {
          if (dr === 0 && dc === 0) continue;
          const nr = r + dr, nc = c + dc;
          if (nr >= 0 && nr < size && nc >= 0 && nc < size && houses[nr][nc]) {
            adjacent[r][c] = true;
            anyAdjacent = true;
          }
        }
      }
    }
  }

  const rowStatus = rowCounts.map((n, r) => statusOf(n, rowClues[r]));
  const colStatus = colCounts.map((n, c) => statusOf(n, colClues[c]));

  // Count trees once for the cardinality test.
  let treeCount = 0;
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) if (trees[r][c]) treeCount++;
  }

  const cluesExact =
    rowStatus.every((s) => s === "exact") && colStatus.every((s) => s === "exact");
  const won =
    cluesExact && !anyAdjacent && houseCount === treeCount &&
    hasPerfectMatching(size, trees, houses);

  return { rowCounts, colCounts, rowStatus, colStatus, adjacent, houseCount, won };
}

function refreshWin() {
  state.won = validate().won;
}

function statusOf(n, clue) {
  if (n > clue) return "over";
  if (n === clue) return "exact";
  return "under";
}

function grid(size, fill) {
  const g = new Array(size);
  for (let r = 0; r < size; r++) g[r] = new Array(size).fill(fill);
  return g;
}
