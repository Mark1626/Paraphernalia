#include <stdint.h>
#include <stdio.h>

int64_t tot = 0;
int64_t cnt = 0;

void recur(int ld, int col, int rd) {
  int pos, bit;
  if (col == tot)
    ++cnt;
  pos = (~(ld | col | rd)) & tot;
  while (pos) {
    bit = pos & -pos;
    pos -= bit;
    recur((ld | bit) << 1, col | bit, (rd | bit) >> 1);
  }
}

int main() {
  int i;
  for (i = 0; i < 12; ++i) {
    cnt = 0;
    tot = (2 << i) - 1;
    recur(0, 0, 0);
    printf("%d %lld\n", i + 1, cnt);
  }
  return 0;
}
