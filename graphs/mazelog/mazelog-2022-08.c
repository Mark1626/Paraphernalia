/*
 *
 * Solution to mazelog monthly challenge Aug 2022
 * Author: Nimalan M, mark1626
 * cc -o mazelog-2022-08 mazelog-2022-08.c -std=c99 -pedantic
 */
#include <stdio.h>
#include <stdlib.h>

#define W 6
#define H 5
#define MAX_PATH_LEN 26
#define NO_PATH 99


static const int grid[] = {
   3, +0, -2, -1, -1, -1,
  -2, +1, +0, -1, -2, +0,
  +0, +3, +2, -2, -1, +2,
  +2, +0, -2, -2, -1, +1,
  +1, +2, -2, +0, -1,  0
};


static const int moves[] = {
  +0, -1,
  +1, -1,
  +1, +0,
  +1, +1,
  +0, +1,
  -1, +1,
  -1, +0,
  -1, -1
};


static void print_path(int *path, int step)
{
    printf("Path len: %d; ", step);
    for (int i = 0; i <= step; i++)
    {
      printf("%d%c", path[i]+1, " \n"[i==step]);
    }
}


static int solve(int *path, int step, int prevd, int bestn)
{
  if (step >= MAX_PATH_LEN)
  {
    return NO_PATH;
  }
  // Reached bottom right corner
  if (path[step] == W*H - 1)
  {
    // Print path if we reach the end
    print_path(path, step);
    bestn = step;
  }
  else if (step <= bestn)
  {
    // Position zero is assign not add
    if (path[step] == 0) prevd = grid[0];
    div_t d = div(path[step], W);

    // Current position
    int x = d.rem;
    int y = d.quot;

    // Possible next direction
    for (int arrow = 0; arrow < 8; arrow++)
    {
      // Next position
      int xx = x + prevd * moves[arrow*2 + 0];
      int yy = y + prevd * moves[arrow*2 + 1];

      if (xx >= 0 && xx < W && yy >= 0 && yy < H)
      {
        int newd = prevd + grid[yy * W + xx];
        int same_pt = xx == x && yy == y;
        path[step+1] = yy * W + xx;
        if (newd > 0 && !same_pt)
        {
          bestn = solve(path, step+1, newd, bestn);
        }
      }
    }

    // Continue only if new path is better than old path
    bestn = bestn < step ? bestn : NO_PATH;
  }
  return bestn;
}

int main(void)
{
  int path[MAX_PATH_LEN + 1] = {0};
  solve(path, 0, grid[0], MAX_PATH_LEN + 1);
}

