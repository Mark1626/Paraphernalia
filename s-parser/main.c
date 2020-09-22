#include <stdlib.h>
#include <stdio.h>
#include "sexpr.h"

int main() {
  FILE *fp = fopen("example.sc", "r");
  struct SNode *node = snode_parse(fp);

  snode_print(node);

  fclose(fp);
  snode_free(node);
  return 0;
}
