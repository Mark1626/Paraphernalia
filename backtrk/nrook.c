#include <stdint.h>
#include <stdio.h>

int64_t cnt;
int64_t tot;

void recur(int col) {
  int bit, pos;
  if (col == tot)
    ++cnt;
  pos = ~col & tot;
  while (pos) {
    bit = pos & -pos;
    pos -= bit;
    recur(col | bit);
  }
}

int main() {
  int i;
  for (i = 0; i < 12; ++i) {
    cnt = 0;
    tot = (2 << i) - 1;
    recur(0);
    printf("%d %lld\n", i, cnt);
  }
  return 0;
}
