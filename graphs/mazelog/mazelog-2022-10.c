/*
 *
 * Solution to mazelog monthly challenge Oct 2022
 * Author: Nimalan M, mark1626
 * cc -o mazelog-2022-10 mazelog-2022-10.c -O3 -std=c99 -pedantic
 */
#include <stdio.h>
#include <stdlib.h>

#define W 6
#define H 5
#define PATH_LEN 50



/* Describes arrows */
enum {
  BNN, BNE, BEE, BSE, BSS, BSW, BWW, BNW,
  PNN, PNE, PEE, PSE, PSS, PSW, PWW, PNW};

static const char grid[] = {
	BEE, PSS, PSS, PSS, PSE, PSW,
	PEE, PNN, PNN, PNN, PSS, PNN,
	PSS, BSS, BSS, BSS, BSS, PSS,
	PSE, BNN, BNN, BNN, BNE, PSS,
	PNN, BNN, PNW, BNN, PEE, 00
};

static const int moves[] = {
	+0, -1, // BNN
	+1, -1, // BNE
	+1, +0, // BEE
	+1, +1, // BSE
	+0, +1, // BSS
	-1, +1, // BSW
	-1, +0, // BWW
	-1, -1, // BNW

	+0, -1, // PNN
	+1, -1, // PNE
	+1, +0, // PEE
	+1, +1, // PSE
	+0, +1, // PSS
	-1, +1, // PSW
	-1, +0, // PWW
	-1, -1  // PNW
};



static void print_path(int *path, int step)
{
    printf("Path len: %d; ", step);
    for (int i = 0; i <= step; i++)
    {
      printf("%d%c", path[i]+1, " \n"[i==step]);
    }
}



static const char* print_arrow(int arrow) {
	switch (arrow) {
	case BNN: return "BNN";
	case BNE: return "BNE";
	case BEE: return "BEE";
	case BSE: return "BSE";
	case BSS: return "BSS";
	case BSW: return "BSW";
	case BWW: return "BWW";
	case BNW: return "BNW";

  case PNN: return "PNN";
	case PNE: return "PNE";
	case PEE: return "PEE";
	case PSE: return "PSE";
	case PSS: return "PSS";
	case PSW: return "PSW";
	case PWW: return "PWW";
	case PNW: return "PNW";
	}
	return "";
}



static int solve(int *path, int step, int bestn, int iter_step, int iter_to_change)
{
	if (path[step] == W*H - 1)
	{
		print_path(path, step);
		bestn = step;
	} else if (step < bestn-1)
	{
		div_t d = div(path[step], W);
		// Current position
		int x = d.rem;
		int y = d.quot;

		// Current arrow
		int arrow = grid[path[step]];
		// parity - Current color
		// 1 - Pink
		// 0 - Blue
		int curr_pink = arrow >= PNN;

		// Possible
		for (int d = 1; d <= W ;d++)
		{
			int xx = x + d * moves[arrow*2 + 0];
			int yy = y + d * moves[arrow*2 + 1];
			// Check if within boundary
			if (xx >= 0 && xx < W && yy >= 0 && yy < H)
			{
				// Check the color
				int poss_arrow = grid[yy * W + xx];
				int next_pink = poss_arrow >= PNN;

				// Logic to switch number of blue/pink arrows
				if (iter_step == iter_to_change || iter_step == 2 * iter_to_change)
				{
					// curr_pink and !next_pink
					// or
					// !curr_pink and next_pink
					if (curr_pink ^ next_pink)
					{
						#ifdef DEBUG
						printf("Poss next from (%d, %d) %s - %d to diff color -> (%d, %d) - %d\n",
							x, y, print_arrow(arrow), y * W + x + 1,
							xx, yy, yy * W + xx + 1);
						print_path(path, step);
						#endif
						path[step+1] = yy * W + xx;

							// printf("Iter change%d\n", iter_to_change);
							bestn = (iter_step == 2 * iter_to_change)
								? solve(path, step+1, bestn, 1, iter_to_change + 1)
								: solve(path, step+1, bestn, iter_step + 1, iter_to_change);
					}
				}
				else
				{
				  // curr_pink and next_pink
					// or
					// !curr_pink and !next_pink
					if (curr_pink == next_pink)
					{
						#ifdef DEBUG
						printf("Poss next from (%d, %d) %s - %d to same color -> (%d, %d) - %d\n",
							x, y, print_arrow(arrow), y * W + x + 1,
							xx, yy, yy * W + xx + 1);
						print_path(path, step);
						#endif
						path[step+1] = yy * W + xx;
						bestn = solve(path, step+1, bestn, iter_step + 1, iter_to_change);
					}
				}
			}
			else
				break;
		}
	}
	return bestn;
}



int main() {
	int path[PATH_LEN+1] = {0};
	solve(path, 0, PATH_LEN, 1, 1);
}
