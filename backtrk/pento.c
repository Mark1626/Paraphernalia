/* Pentomino tiling counter for a 6 x 10 board.
 *
 * C implementation following the strategy of the "Pentominoes" section
 * of Martin Richards, "Backtracking Algorithms in MCPL using Bit
 * Patterns and Recursion", University of Cambridge Computer Lab.
 *
 * Three ideas from the paper:
 *
 *   1. The HANDLE rule.  The next piece placed must always cover the
 *      top-leftmost unoccupied cell (row-major order).  Some piece must
 *      cover that cell eventually, so forcing it now loses no solutions
 *      but makes the search enumerate each tiling exactly once -- no
 *      duplicates from placement-order permutations.
 *
 *   2. Precomputed variant placements.  Every rotation/reflection of
 *      every piece, in every legal board position, is enumerated once
 *      up front as a 60-bit mask.  The search never reasons about
 *      geometry -- only about bit patterns.
 *
 *   3. Bitmask board.  The board is a 60-bit occupancy mask.  "Does
 *      this piece fit?" is a single AND; "place the piece" is an OR.
 *
 * Board: WIDTH x HEIGHT = 6 x 10 = 60 cells, fits in a uint64_t.
 * Cell (r,c) is bit r*WIDTH + c.
 *
 * Symmetry: the 6x10 board (non-square) has a symmetry group of order
 * 4 -- identity, 180-degree rotation, and two mirrors.  The standard
 * published count of 2339 factors all of these out.  A raw handle
 * search counts each tiling once per orientation, i.e. 4 times, giving
 * 9356.  We remove the two mirror images by restricting the F piece
 * (which is chiral -- its mirror is a genuinely different piece) to a
 * single chirality; that leaves identity + 180-rotation, so the search
 * yields 2 * 2339 = 4678, and we divide by 2.
 *
 * Expected answer: 2339 distinct tilings.
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

#define WIDTH   6
#define HEIGHT  10
#define NCELLS  (WIDTH * HEIGHT)      /* 60 */
#define NPIECES 12

typedef uint64_t u64;

/* ---- the twelve pentominoes ----------------------------------------
 * Each piece is a list of (row,col) offsets of its 5 cells in some
 * base orientation.  The solver generates all rotations/reflections. */

typedef struct { int cell[5][2]; } Shape;

static const Shape base[NPIECES] = {
    /* F */ {{{0,1},{0,2},{1,0},{1,1},{2,1}}},
    /* I */ {{{0,0},{1,0},{2,0},{3,0},{4,0}}},
    /* L */ {{{0,0},{1,0},{2,0},{3,0},{3,1}}},
    /* N */ {{{0,1},{1,1},{2,0},{2,1},{3,0}}},
    /* P */ {{{0,0},{0,1},{1,0},{1,1},{2,0}}},
    /* T */ {{{0,0},{0,1},{0,2},{1,1},{2,1}}},
    /* U */ {{{0,0},{0,2},{1,0},{1,1},{1,2}}},
    /* V */ {{{0,0},{1,0},{2,0},{2,1},{2,2}}},
    /* W */ {{{0,0},{1,0},{1,1},{2,1},{2,2}}},
    /* X */ {{{0,1},{1,0},{1,1},{1,2},{2,1}}},
    /* Y */ {{{0,1},{1,0},{1,1},{2,1},{3,1}}},
    /* Z */ {{{0,0},{0,1},{1,1},{2,1},{2,2}}},
};
static const char pname[NPIECES] =
    {'F','I','L','N','P','T','U','V','W','X','Y','Z'};

#define FPIECE 0   /* index of the chiral F piece used to kill mirrors */

/* ---- precomputed placements ----------------------------------------
 * placements[h] lists every way any piece can be placed with its
 * top-leftmost cell exactly on handle cell h. */

typedef struct { u64 mask; int piece; } Placement;

static Placement *placements[NCELLS];
static int        nplace[NCELLS];
static int        capplace[NCELLS];

static void add_placement(int handle, u64 mask, int piece)
{
    if (nplace[handle] == capplace[handle]) {
        capplace[handle] = capplace[handle] ? capplace[handle] * 2 : 8;
        placements[handle] = realloc(placements[handle],
                                     capplace[handle] * sizeof(Placement));
        if (!placements[handle]) { perror("realloc"); exit(1); }
    }
    placements[handle][nplace[handle]].mask  = mask;
    placements[handle][nplace[handle]].piece = piece;
    nplace[handle]++;
}

/* normalise an offset list: shift to non-negative, sort row-major */
static void normalise(int cell[5][2])
{
    int minr = 99, minc = 99;
    for (int i = 0; i < 5; i++) {
        if (cell[i][0] < minr) minr = cell[i][0];
        if (cell[i][1] < minc) minc = cell[i][1];
    }
    for (int i = 0; i < 5; i++) { cell[i][0] -= minr; cell[i][1] -= minc; }
    for (int a = 0; a < 5; a++)
        for (int b = a + 1; b < 5; b++) {
            int ka = cell[a][0]*16 + cell[a][1];
            int kb = cell[b][0]*16 + cell[b][1];
            if (kb < ka) {
                int t0 = cell[a][0], t1 = cell[a][1];
                cell[a][0] = cell[b][0]; cell[a][1] = cell[b][1];
                cell[b][0] = t0;         cell[b][1] = t1;
            }
        }
}

/* Build all placement masks. For piece F only one chirality is kept,
 * which removes the board's mirror symmetry from the final count. */
static void build_placements(void)
{
    for (int p = 0; p < NPIECES; p++) {
        int orient[8][5][2];
        int norient = 0;

        int cur[5][2];
        for (int i = 0; i < 5; i++) {
            cur[i][0] = base[p].cell[i][0];
            cur[i][1] = base[p].cell[i][1];
        }

        for (int refl = 0; refl < 2; refl++) {
            for (int rot = 0; rot < 4; rot++) {
                /* for piece F, skip the mirrored half of orientations */
                if (!(p == FPIECE && refl == 1)) {
                    int norm[5][2];
                    for (int i = 0; i < 5; i++) {
                        norm[i][0] = cur[i][0];
                        norm[i][1] = cur[i][1];
                    }
                    normalise(norm);
                    int dup = 0;
                    for (int o = 0; o < norient && !dup; o++) {
                        int same = 1;
                        for (int i = 0; i < 5; i++)
                            if (orient[o][i][0] != norm[i][0] ||
                                orient[o][i][1] != norm[i][1]) same = 0;
                        if (same) dup = 1;
                    }
                    if (!dup) {
                        for (int i = 0; i < 5; i++) {
                            orient[norient][i][0] = norm[i][0];
                            orient[norient][i][1] = norm[i][1];
                        }
                        norient++;
                    }
                }
                /* rotate 90 deg: (r,c) -> (c,-r) */
                for (int i = 0; i < 5; i++) {
                    int r = cur[i][0], c = cur[i][1];
                    cur[i][0] = c; cur[i][1] = -r;
                }
            }
            /* reflect: (r,c) -> (r,-c) */
            for (int i = 0; i < 5; i++) cur[i][1] = -cur[i][1];
        }

        /* slide each distinct orientation over the whole board */
        for (int o = 0; o < norient; o++)
            for (int dr = 0; dr < HEIGHT; dr++)
                for (int dc = 0; dc < WIDTH; dc++) {
                    u64 mask = 0;
                    int ok = 1, handle = NCELLS;
                    for (int i = 0; i < 5; i++) {
                        int r = orient[o][i][0] + dr;
                        int c = orient[o][i][1] + dc;
                        if (r < 0 || r >= HEIGHT || c < 0 || c >= WIDTH) {
                            ok = 0; break;
                        }
                        int bit = r * WIDTH + c;
                        mask |= (u64)1 << bit;
                        if (bit < handle) handle = bit;
                    }
                    if (ok) add_placement(handle, mask, p);
                }
    }
}

/* ---- the recursive search ------------------------------------------ */

static long total_count = 0;

/* record of one tiling, for printing a sample */
static int  sample[NCELLS];      /* cell -> piece index */
static int  have_sample = 0;
static int  place_piece[NPIECES];/* during recursion: piece at each step */
static u64  place_mask[NPIECES];

static void search(u64 board, int used, int depth)
{
    if (depth == NPIECES) {                 /* board full */
        total_count++;
        if (!have_sample) {                 /* capture the first one */
            for (int s = 0; s < NPIECES; s++) {
                u64 m = place_mask[s];
                for (int b = 0; b < NCELLS; b++)
                    if (m & ((u64)1 << b)) sample[b] = place_piece[s];
            }
            have_sample = 1;
        }
        return;
    }

    int handle = 0;                         /* lowest clear bit */
    while (board & ((u64)1 << handle)) handle++;

    const Placement *list = placements[handle];
    int len = nplace[handle];

    for (int i = 0; i < len; i++) {
        u64 m  = list[i].mask;
        int pc = list[i].piece;
        if (board & m)        continue;     /* overlap */
        if (used & (1 << pc)) continue;     /* piece already used */
        place_piece[depth] = pc;
        place_mask[depth]  = m;
        search(board | m, used | (1 << pc), depth + 1);
    }
}

int main(void)
{
    build_placements();
    search(0, 0, 0);

    long distinct = total_count / 2;        /* remove 180-deg rotation */

    printf("Raw count (F chirality fixed): %ld\n", total_count);
    printf("Distinct tilings of 6x10:      %ld\n", distinct);
    printf("%s\n", distinct == 2339
           ? "OK - matches the known result of 2339."
           : "MISMATCH - expected 2339.");

    if (have_sample) {
        printf("\nOne sample tiling:\n\n");
        for (int r = 0; r < HEIGHT; r++) {
            printf("  ");
            for (int c = 0; c < WIDTH; c++)
                printf("%c ", pname[sample[r * WIDTH + c]]);
            printf("\n");
        }
    }

    for (int h = 0; h < NCELLS; h++) free(placements[h]);
    return 0;
}
