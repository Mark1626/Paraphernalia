#include "cuckoo.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <inttypes.h>

int verbose = 0;

int main(int argc, char** argv) {
  int easipct = 50;
  
  char* header;

  int c;
  while ((c = getopt(argc, argv, "vh:n:e:")) != -1) {
    switch (c) {
      case 'v':
        verbose = 1;
        break;
      case 'h':
        header = optarg;
        break;
      case 'e':
        easipct = atoi(optarg);
        break;
    }
  }

  if (verbose) printf("Reading nonces \n");
  int64_t nonce = 0;
  int32_t nonces[PROOFSIZE];
  if (verbose) printf("%d ", optind);
  for (int optIdx = optind, idx = 0; optIdx < argc && idx < PROOFSIZE; optIdx++, idx++) {
    if (verbose) printf("%s ", argv[optIdx]);
    sscanf(argv[optIdx], " %" SCNx64, &nonce);
    nonces[idx] = nonce;
  }

  int32_t easiness = (int32_t)((50 * NNODES) / 100L);

  if (verbose) {
    printf("Header %s \n", header);
    printf("Easiness %d \n", easiness);
    printf("Nonces ");
    for (int idx = 0; idx < PROOFSIZE; idx++) {
      printf("%d \t", nonces[idx]);
    }
    printf("\n");
  }

  if (verbose) printf("Init graph \n");
  cuckoo* cukoo_graph = init_graph(header, strlen(header));
  if (verbose)
  printf("graph { %lld %lld %lld %lld } \n", 
    cukoo_graph->v[0],
    cukoo_graph->v[1],
    cukoo_graph->v[2],
    cukoo_graph->v[3]
  );

  if (verbose) printf("Verifing solution \n");
  int pow_rc = verify(cukoo_graph, nonces, easiness);

  if (pow_rc == POW_OK) printf("Solution found\n");
  else printf("Solution not found: errcode: %d\n", pow_rc);
}
