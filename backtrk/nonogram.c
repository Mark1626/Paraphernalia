/* Nonogram solver using the bit-pattern line-solving technique.
 *
 * In the spirit of the "Nonograms" section of Martin Richards,
 * "Backtracking Algorithms in MCPL using Bit Patterns and Recursion",
 * University of Cambridge Computer Laboratory.
 *
 * Method
 * ------
 * Each line (row or column) of length n is an n-bit pattern: a set bit
 * means "filled".  A line's clue admits a SET of legal patterns.
 * Across the surviving candidates of one line:
 *
 *   - AND of all candidates  -> cells that are filled in EVERY one
 *                               => definitely FILLED
 *   - OR  of all candidates  -> cells clear in the result are clear in
 *                               every candidate => definitely EMPTY
 *   - anything else is an UNRESOLVED cell.
 *
 * Per line we keep two masks: `fill` (known filled) and `empty` (known
 * empty).  Solving = repeat until a full pass changes nothing:
 *   1. filter each line's candidate list against its known masks,
 *   2. AND/OR the survivors to extract newly forced cells,
 *   3. propagation is automatic: a row's masks and a column's masks
 *      both feed the shared cell grid on the next pass.
 * If cells remain unresolved, guess one and recurse (backtracking).
 *
 * The whole-grid state is two bit matrices, GFILL and GEMPTY, indexed
 * by cell; row and column known-masks are derived from them each pass.
 *
 * Grid limited to 64x64 so each line fits in one uint64_t.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

typedef uint64_t u64;

#define MAXN 64

static int R, C;                         /* rows, cols */

/* clue lists */
static int rclue[MAXN][MAXN], rcluen[MAXN];
static int cclue[MAXN][MAXN], ccluen[MAXN];

/* candidate pattern sets, one resizable list per line */
static u64 *rcand[MAXN]; static int rcandn[MAXN];
static u64 *ccand[MAXN]; static int ccandn[MAXN];

/* whole-grid knowledge: bit c of gfill[r] set => cell (r,c) known filled */
static u64 gfill[MAXN];   /* known-filled cells, per row  */
static u64 gempty[MAXN];  /* known-empty  cells, per row  */

/* ---- candidate generation ------------------------------------------
 * Enumerate every length-`len` pattern satisfying clue[0..k).  Blocks
 * are laid left to right; recursion chooses the gap before each block.
 * Bit 0 is the leftmost cell. */
static void gen(int len, int *clue, int k,
                u64 **list, int *count, int *cap,
                int idx, int pos, u64 bits)
{
    if (idx == k) {
        if (*count == *cap) {
            *cap = *cap ? *cap * 2 : 16;
            *list = realloc(*list, *cap * sizeof(u64));
            if (!*list) { perror("realloc"); exit(1); }
        }
        (*list)[(*count)++] = bits;
        return;
    }
    /* cells still required by blocks idx..k-1, including mandatory gaps */
    int need = 0;
    for (int i = idx; i < k; i++) need += clue[i] + (i > idx ? 1 : 0);

    for (int start = pos; start + need <= len; start++) {
        u64 blk = (((u64)1 << clue[idx]) - 1) << start;
        gen(len, clue, k, list, count, cap,
            idx + 1, start + clue[idx] + 1, bits | blk);
    }
}

static void build_candidates(void)
{
    for (int r = 0; r < R; r++) {
        int cap = 0; rcand[r] = NULL; rcandn[r] = 0;
        if (rcluen[r] == 0) {                       /* blank line */
            rcand[r] = malloc(sizeof(u64));
            rcand[r][0] = 0; rcandn[r] = 1;
        } else {
            gen(C, rclue[r], rcluen[r], &rcand[r], &rcandn[r], &cap, 0,0,0);
        }
    }
    for (int c = 0; c < C; c++) {
        int cap = 0; ccand[c] = NULL; ccandn[c] = 0;
        if (ccluen[c] == 0) {
            ccand[c] = malloc(sizeof(u64));
            ccand[c][0] = 0; ccandn[c] = 1;
        } else {
            gen(R, cclue[c], ccluen[c], &ccand[c], &ccandn[c], &cap, 0,0,0);
        }
    }
}

/* ---- one line-solving step -----------------------------------------
 * Filter `cand[0..*n)` against the line's known masks (fill,empty),
 * compacting survivors in place and updating *n.  Then OR/AND the
 * survivors to produce the line's forced-filled and forced-empty masks,
 * returned through *outfill and *outempty.
 * Returns 0 on contradiction (no candidate survives), 1 otherwise. */
static int line_step(u64 *cand, int *n, int len,
                      u64 fill, u64 empty,
                      u64 *outfill, u64 *outempty)
{
    u64 fullmask = (len == 64) ? ~(u64)0 : (((u64)1 << len) - 1);
    u64 andacc = fullmask, oracc = 0;
    int kept = 0;

    for (int i = 0; i < *n; i++) {
        u64 p = cand[i];
        /* clash if p fills a known-empty cell, or leaves a known-filled
           cell empty */
        if (p & empty)                continue;
        if (fill & ~p & fullmask)     continue;
        cand[kept++] = p;
        andacc &= p;
        oracc  |= p;
    }
    *n = kept;
    if (kept == 0) return 0;                 /* contradiction */

    *outfill  = andacc;                      /* set in every survivor */
    *outempty = fullmask & ~oracc;           /* clear in every survivor */
    return 1;
}

/* extract column c's bits from the per-row grid masks */
static void column_masks(int c, u64 *fill, u64 *empty)
{
    u64 f = 0, e = 0;
    for (int r = 0; r < R; r++) {
        if (gfill[r]  & ((u64)1 << c)) f |= (u64)1 << r;
        if (gempty[r] & ((u64)1 << c)) e |= (u64)1 << r;
    }
    *fill = f; *empty = e;
}

/* ---- propagation to fixpoint ---------------------------------------
 * Returns 1 if consistent, 0 if a contradiction was found. */
static int propagate(void)
{
    int changed = 1;
    while (changed) {
        changed = 0;

        /* rows */
        for (int r = 0; r < R; r++) {
            u64 of, oe;
            if (!line_step(rcand[r], &rcandn[r], C,
                           gfill[r], gempty[r], &of, &oe))
                return 0;
            if ((of & ~gfill[r]) || (oe & ~gempty[r])) changed = 1;
            gfill[r]  |= of;
            gempty[r] |= oe;
        }

        /* columns */
        for (int c = 0; c < C; c++) {
            u64 cf, ce, of, oe;
            column_masks(c, &cf, &ce);
            if (!line_step(ccand[c], &ccandn[c], R,
                           cf, ce, &of, &oe))
                return 0;
            /* scatter the column's new deductions back into rows */
            for (int r = 0; r < R; r++) {
                u64 bit = (u64)1 << c;
                if ((of & ((u64)1 << r)) && !(gfill[r] & bit)) {
                    gfill[r] |= bit; changed = 1;
                }
                if ((oe & ((u64)1 << r)) && !(gempty[r] & bit)) {
                    gempty[r] |= bit; changed = 1;
                }
            }
        }
    }
    return 1;
}

/* ---- backtracking search -------------------------------------------
 * Save/restore the whole mutable state (grid + candidate counts) so a
 * guess can be undone. */

static int solve(void);

static int try_with_rebuild(void)
{
    /* rebuild candidate lists fresh, then filter to current grid */
    for (int r = 0; r < R; r++) { free(rcand[r]); }
    for (int c = 0; c < C; c++) { free(ccand[c]); }
    build_candidates();
    return solve();
}

/* recursively solve from the current grid state; returns 1 if solved */
static int solve(void)
{
    if (!propagate()) return 0;

    /* find an unresolved cell */
    int gr = -1, gc = -1;
    for (int r = 0; r < R && gr < 0; r++) {
        u64 known = gfill[r] | gempty[r];
        u64 full  = (C == 64) ? ~(u64)0 : (((u64)1 << C) - 1);
        u64 unk   = full & ~known;
        if (unk) {
            gr = r;
            gc = __builtin_ctzll(unk);
        }
    }
    if (gr < 0) return 1;                 /* fully resolved */

    /* snapshot grid only; candidate lists are rebuilt on each branch */
    u64 savef[MAXN], savee[MAXN];
    memcpy(savef, gfill,  sizeof savef);
    memcpy(savee, gempty, sizeof savee);

    /* branch 1: guess the cell FILLED */
    gfill[gr] |= (u64)1 << gc;
    if (try_with_rebuild()) return 1;

    /* restore, branch 2: guess EMPTY */
    memcpy(gfill,  savef, sizeof savef);
    memcpy(gempty, savee, sizeof savee);
    gempty[gr] |= (u64)1 << gc;
    if (try_with_rebuild()) return 1;

    /* neither worked: restore and fail */
    memcpy(gfill,  savef, sizeof savef);
    memcpy(gempty, savee, sizeof savee);
    return 0;
}

/* ---- I/O ------------------------------------------------------------ */

static void print_grid(void)
{
    for (int r = 0; r < R; r++) {
        for (int c = 0; c < C; c++)
            putchar((gfill[r] & ((u64)1 << c)) ? '#' : '.');
        putchar('\n');
    }
}

int main(void)
{
    /* 10x10 nonogram of a small house. Clues were derived from the
       known picture below, so the puzzle is guaranteed solvable; the
       solver should reproduce exactly that picture. This instance
       needs propagation plus some backtracking guesses.

         ...##.....
         ..####....
         .######...
         ##########
         ..#....#..
         ..#....#..
         ..#.##.#..
         ..#.##.#..
         ..#.##.#..
         ..######..                                            */
    R = 10; C = 10;

    int rn[10] = { 1,1,1,1,2,2,3,3,3,1 };
    int rc[10][3] = {
        {2,0,0}, {4,0,0}, {6,0,0}, {10,0,0}, {1,1,0},
        {1,1,0}, {1,2,1}, {1,2,1}, {1,2,1}, {6,0,0},
    };
    int cn[10] = { 1,1,1,2,2,2,2,1,1,1 };
    int cc[10][2] = {
        {1,0}, {2,0}, {9,0}, {4,1}, {4,4},
        {3,4}, {2,1}, {7,0}, {1,0}, {1,0},
    };

    for (int r = 0; r < R; r++) {
        rcluen[r] = rn[r];
        for (int i = 0; i < rn[r]; i++) rclue[r][i] = rc[r][i];
    }
    for (int c = 0; c < C; c++) {
        ccluen[c] = cn[c];
        for (int i = 0; i < cn[c]; i++) cclue[c][i] = cc[c][i];
    }

    build_candidates();
    for (int r = 0; r < R; r++) { gfill[r] = 0; gempty[r] = 0; }

    if (solve()) { printf("Solved:\n\n"); print_grid(); }
    else         { printf("No solution.\n"); }

    for (int r = 0; r < R; r++) free(rcand[r]);
    for (int c = 0; c < C; c++) free(ccand[c]);
    return 0;
}
