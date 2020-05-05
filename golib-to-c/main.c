#include "libarith.h"
#include <stdio.h>

int main(int argc, char** argv) {

  long long a = 10;
  long long b = 10;

  printf("Sum %lld\n", add(a, b));
  printf("Diff %lld\n", sub(a, b));
  printf("Mul %lld\n", mul(a, b));

  return 0;
}
