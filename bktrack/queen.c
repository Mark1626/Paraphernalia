#include <stdio.h>

static const int N = 12;
static const int all = (1 << N) - 1;
static int count = 0;

static void search(int ld, int cols, int rd) {
  if (cols == all) {
    count++;
  }
  int poss = ~(ld | cols | rd) & all;
  while (poss) {
    int bit = poss & -poss;
    poss -= bit;
    search((ld | bit) << 1, cols | bit, (rd | bit) >> 1);
  }
}

int main() {
  search(0, 0, 0);
  printf("Solutions to %d queens : %d\n", N, count);
  return 0;
}
