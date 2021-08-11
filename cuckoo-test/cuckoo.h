#include <stdint.h>

#ifndef CUCKOO_H
#define CUCKOO_H

typedef struct {
  int64_t v[4];
} cuckoo;

#define EDGEBITS 19
#define NEDGES (1ULL << EDGEBITS)
#define NODEBITS (EDGEBITS + 1)
#define NNODES (1ULL << NODEBITS)
#define EDGEMASK (NEDGES - 1)
#define PROOFSIZE 42

enum verify_code { POW_OK, POW_HEADER_LENGTH, POW_TOO_BIG, POW_TOO_SMALL, POW_NON_MATCHING, POW_BRANCH, POW_DEAD_END, POW_SHORT_CYCLE};
// const char *solnerr[] = { "OK", "wrong header length", "edge too big", "edges not ascending", "endpoints don't match up", "branch in cycle", "cycle dead ends", "cycle too short"};

cuckoo* init_graph(char *header, int headerlen);
int verify(cuckoo* v, int32_t *nonces, int32_t easiness);

#endif