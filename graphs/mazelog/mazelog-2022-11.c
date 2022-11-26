/*
 *
 * Solution to mazelog monthly challenge Aug 2022
 * Author: Nimalan M, mark1626
 * cc -o mazelog-2022-11 mazelog-2022-1.c -O3 -std=c99 -pedantic
 */
#include <stdio.h>
#include <stdlib.h>

#define W 6
#define H 6
#define NO_PATH 99
#define MAX_PATH_LEN 26

// Suites
typedef enum { NA, HEART, DIAMD, SPADE, CLOVR } suit_t;

static const suit_t grid[] = {
	HEART, HEART, CLOVR, HEART, CLOVR, CLOVR,
	DIAMD, SPADE, CLOVR, DIAMD, CLOVR, HEART,
	CLOVR, CLOVR, HEART, CLOVR, HEART, CLOVR,
	CLOVR, DIAMD, CLOVR, HEART, CLOVR, HEART,
	DIAMD, SPADE, DIAMD, HEART, CLOVR, HEART,
	HEART, HEART, HEART, DIAMD, HEART, HEART
};

typedef enum {
	NO_DIR, NN, EE, SS, WW
} dir_t;

static void print_path(int *path, int step) {
	printf("Path len: %d; ", step);
	for (int i = 0; i <= step; i++) {
		printf("%d%c", path[i] + 1, " \n"[i == step]);
	}
}

static int solve(int *path, int step, int bestn, suit_t prev, dir_t prev_dir) {
	if (step >= MAX_PATH_LEN) {
		return NO_PATH;
	}

	// Reached bottom right corner
	if (path[step] == W * H - 1) {
		print_path(path, step);
		bestn = step;
	} else if (step <= bestn) {
		div_t d = div(path[step], W);
		// Current position
		int x = d.rem;
		int y = d.quot;

		suit_t curr_suit = grid[path[step]];

		if (prev_dir != SS)
		{
			// Possible NN
			for (int d = y; d >= 0; d--) {
				suit_t poss_suit = grid[d * W + x];
				if (poss_suit == curr_suit)
					continue;
				if (prev != NA)
					if (poss_suit == prev)
						continue;

				path[step+1] = d * W + x;
				bestn = solve(path, step+1, bestn, curr_suit, NN);
			}
		}

		if (prev_dir != WW)
		{
			// Possible EE
			for (int d = x; d < W; d++) {
				suit_t poss_suit = grid[y * W + d];
				if (poss_suit == curr_suit)
					continue;
				if (prev != NA)
					if (poss_suit == prev)
						continue;

				path[step+1] = y * W + d;
				bestn = solve(path, step+1, bestn, curr_suit, EE);
			}
		}

		if (prev_dir != NN)
		{
			// Possible SS
			for (int d = y; d < W; d++) {
				suit_t poss_suit = grid[d * W + x];
				if (poss_suit == curr_suit)
					continue;
				if (prev != NA)
					if (poss_suit == prev)
						continue;

				path[step+1] = d * W + x;
				bestn = solve(path, step+1, bestn, curr_suit, SS);
			}
		}

		if (prev_dir != EE)
		{
			// Possible WW
			for (int d = x; d >= 0; d--) {
				suit_t poss_suit = grid[y * W + d];
				if (poss_suit == curr_suit)
					continue;
				if (prev != NA)
					if (poss_suit == prev)
						continue;

				path[step+1] = y * W + d;
				bestn = solve(path, step+1, bestn, curr_suit, WW);
			}
		}

	}
	return bestn;
}

int main() {
  int path[MAX_PATH_LEN + 1] = {0};
  solve(path, 0, MAX_PATH_LEN + 1, NA, NO_DIR);
}
