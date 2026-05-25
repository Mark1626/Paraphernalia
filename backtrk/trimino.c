#include <stdint.h>
#include <stdio.h>
#include "stdlib.h"

#define WIDTH 2
#define HEIGHT 9
#define NCELLS (WIDTH * HEIGHT)
#define NPIECES 2                /* distinct tromino types: L, I */
#define NPLACED (NCELLS / 3)     /* trominoes needed to fill the board */
#define FULL (((u64)1 << NCELLS) - 1)

typedef uint64_t u64;

typedef struct {
  int cell[3][2];
} Shape;

static const Shape base[NPIECES] = {
    /* L */ {{{0, 0}, {1, 0}, {1, 1}}},
    /* I */ {{{0, 0}, {1, 0}, {2, 0}}},
};

static const char pname[NPIECES] = {'L', 'I'};

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
static void normalise(int cell[3][2])
{
    int minr = 99, minc = 99;
    for (int i = 0; i < 3; i++) {
        if (cell[i][0] < minr) minr = cell[i][0];
        if (cell[i][1] < minc) minc = cell[i][1];
    }
    for (int i = 0; i < 3; i++) { cell[i][0] -= minr; cell[i][1] -= minc; }
    for (int a = 0; a < 3; a++)
        for (int b = a + 1; b < 3; b++) {
            int ka = cell[a][0]*16 + cell[a][1];
            int kb = cell[b][0]*16 + cell[b][1];
            if (kb < ka) {
                int t0 = cell[a][0], t1 = cell[a][1];
                cell[a][0] = cell[b][0]; cell[a][1] = cell[b][1];
                cell[b][0] = t0;         cell[b][1] = t1;
            }
        }
}

static void build_placements(void)
{
    for (int p = 0; p < NPIECES; p++) {
        int orient[8][3][2];
        int norient = 0;

        int cur[3][2];
        for (int i = 0; i < 3; i++) {
            cur[i][0] = base[p].cell[i][0];
            cur[i][1] = base[p].cell[i][1];
        }

        for (int refl = 0; refl < 2; refl++) {
            for (int rot = 0; rot < 4; rot++) {
                    int norm[3][2];
                    for (int i = 0; i < 3; i++) {
                        norm[i][0] = cur[i][0];
                        norm[i][1] = cur[i][1];
                    }
                    normalise(norm);
                    int dup = 0;
                    for (int o = 0; o < norient && !dup; o++) {
                        int same = 1;
                        for (int i = 0; i < 3; i++)
                            if (orient[o][i][0] != norm[i][0] ||
                                orient[o][i][1] != norm[i][1]) same = 0;
                        if (same) dup = 1;
                    }
                    if (!dup) {
                        for (int i = 0; i < 3; i++) {
                            orient[norient][i][0] = norm[i][0];
                            orient[norient][i][1] = norm[i][1];
                        }
                        norient++;
                    }
                /* rotate 90 deg: (r,c) -> (c,-r) */
                for (int i = 0; i < 3; i++) {
                    int r = cur[i][0], c = cur[i][1];
                    cur[i][0] = c; cur[i][1] = -r;
                }
            }
            /* reflect: (r,c) -> (r,-c) */
            for (int i = 0; i < 3; i++) cur[i][1] = -cur[i][1];
        }

        /* slide each distinct orientation over the whole board */
        for (int o = 0; o < norient; o++)
            for (int dr = 0; dr < HEIGHT; dr++)
                for (int dc = 0; dc < WIDTH; dc++) {
                    u64 mask = 0;
                    int ok = 1, handle = NCELLS;
                    for (int i = 0; i < 3; i++) {
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

static long total_count = 0;

/* record of one tiling, for printing a sample */
static int  sample[NCELLS];       /* cell -> placement-order id (A,B,C,...) */
static int  sample_type[NPLACED]; /* placement-order id -> piece type */
static int  sample_n = 0;         /* number of pieces in the captured sample */
static int  have_sample = 0;
static int  place_piece[NPLACED]; /* during recursion: piece at each step */
static u64  place_mask[NPLACED];

static void search(u64 board, int depth)
{
    if (board == FULL) {                 /* board full */
        total_count++;
        if (!have_sample) {                 /* capture the first one */
            for (int s = 0; s < depth; s++) {
                u64 m = place_mask[s];
                sample_type[s] = place_piece[s];   /* remember its L/I type */
                for (int b = 0; b < NCELLS; b++)
                    if (m & ((u64)1 << b)) sample[b] = s;  /* id, not type */
            }
            sample_n = depth;
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
        place_piece[depth] = pc;
        place_mask[depth]  = m;
        search(board | m, depth + 1);
    }
}

int main(void)
{
    build_placements();
    search(0, 0);

    printf("Tilings of 2x9:                %ld\n", total_count);
    printf("%s\n", total_count == 41
           ? "OK - matches the known result of 41."
           : "MISMATCH - expected 41.");

    if (have_sample) {
        printf("\nOne sample tiling (each piece a distinct id):\n\n");
        for (int r = 0; r < HEIGHT; r++) {
            printf("  ");
            for (int c = 0; c < WIDTH; c++)
                printf("%c ", 'A' + sample[r * WIDTH + c]);
            printf("\n");
        }
        printf("\n  legend: ");
        for (int id = 0; id < sample_n; id++)
            printf("%c=%c ", 'A' + id, pname[sample_type[id]]);
        printf("\n");
    }

    for (int h = 0; h < NCELLS; h++) free(placements[h]);
    return 0;
}
