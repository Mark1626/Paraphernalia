/*
 *
 * Solution to mazelog challenge Whorl
 * http://www.mazelog.com/show?L2
 * Author: Nimalan M, mark1626
 * cc -o mazelog-whorl mazelog-whorl.c -O3 -std=c99 -pedantic
 */
#include <stdio.h>
#include <stdlib.h>

#define W 5
#define H 5
#define PATH_LEN 20

/* Describes arrows */
enum {NN, NE, EE, SE, SS, SW, WW, NW};

static const char grid[] = {
	EE, SW, SE, SW, WW,
	NN, WW, SW, SW, WW,
	SE, SE, NW, NW, NE,
	NE, WW, NE, SE, SS,
	SW, SS, SE, NW, 00
};

static const int moves[] = {
	+0, -1, // NN
	+1, -1, // NE
	+1, +0, // EE
	+1, +1, // SE
	+0, +1, // SS
	-1, +1, // SW
	-1, +0, // WW
	-1, -1  // NW
};

static int solve(int *path, int step, int bestn)
{
	int dir = step % 4 < 2 ? +1 : -1;

	// Reached the right bottom corner
	if (path[step] == W*H - 1)
	{
		for (int i=0; i <= step; i++)
		{
			printf("%2d%c", path[i] + 1, " \n"[i == step]);
		}
		bestn = step;
	}
	else if (step < bestn-1)
	{
		div_t d = div(path[step], W);
		// Current position
		int x = d.rem;
		int y = d.quot;

		// Current arrow
		int arrow = grid[path[step]];

		for (int d = 1; ;d++)
		{

			int xx = x + d * moves[arrow*2 + 0] * dir;
			int yy = y + d * moves[arrow*2 + 1] * dir;
			// Check if within boundary
			if (xx >= 0 && xx < W && yy >= 0 && yy < H)
			{
				path[step+1] = yy * W + xx;
				bestn = solve(path, step+1, bestn);
			}
			else
			{
				break;
			}
		}
	}
	return bestn;
}

int main(void) {
	int path[PATH_LEN] = {0};
	solve(path, 0, PATH_LEN);
}
