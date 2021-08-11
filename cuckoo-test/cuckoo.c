#include "cuckoo.h"
#include <stdint.h>
#include <stdio.h>
#include "openssl/sha.h"

typedef struct _Edge {
  int8_t u;
  int8_t v;
} Edge;

#define u8(n) ((int64_t) (n & 0xff))

int64_t u8to64(uint8_t *p, int32_t i) {
  return u8(p[i])         | u8(p[i+1]) << 8 |
         u8(p[i+2]) << 16 | u8(p[i+3]) << 24 |
         u8(p[i+4]) << 32 | u8(p[i+5]) << 40 |
         u8(p[i+6]) << 48 | u8(p[i+7]) << 56;
}

int64_t siphash24(int64_t *v, int32_t nonce) {
  int64_t v0 = v[0], v1 = v[1], v2 = v[2], v3 = v[3] ^ nonce;
  v[3] = v[3] ^ nonce;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    v[0] ^= nonce; v[2] ^= 0xff;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    v[0] += v[1]; v[2] += v[3]; v[1] = (v[1] << 13) | v[1] >> 51;
                        v[3] = (v[3] << 16) | v[3] >> 48;
    v[1] ^= v[0]; v[3] ^= v[2]; v[0] = (v[0] << 32) | v[0] >> 32;
    v[2] += v[1]; v[0] += v[3]; v[1] = (v[1] << 17) | v[1] >> 47;
                        v[3] = (v[3] << 21) | v[3] >> 43;
    v[1] ^= v[2]; v[3] ^= v[0]; v[2] = (v[2] << 32) | v[2] >> 32;

    return v[0] ^ v[1] ^ v[2] ^ v[3];
}

int32_t sipnode(int64_t *v, int32_t nonce, int uorv) {
  return (int32_t) siphash24(v, 2*nonce + uorv) & EDGEMASK;
}

Edge sipedge(cuckoo *c, int32_t nonce) {
  return (Edge) { .u = sipnode(c->v, nonce, 0), .v = sipnode(c->v, nonce, 1) };
}

int verify(cuckoo *c, int32_t *nonces, int32_t easiness) {
  int32_t us[PROOFSIZE], vs[PROOFSIZE];
  int32_t uxor, vxor;
  uxor = vxor = (PROOFSIZE / 2) & 1;

  for (int n = 0; n < PROOFSIZE; n++) {
    // Proof too big
    if (nonces[n] >= easiness)
      return POW_TOO_BIG;
    // Proof too small
    if (n != 0 && nonces[n] <= nonces[n-1])
      return POW_TOO_SMALL;
    us[n] = sipnode(c->v, nonces[n], 0);
    vs[n] = sipnode(c->v, nonces[n], 1);
    printf("nodes %d %d \n", us[n], vs[n]);
    uxor ^= us[n];
    vxor ^= vs[n];
  }
  // if (uxor | vxor) return POW_NON_MATCHING;

  int32_t i = 0; // Start from i == 0
  int32_t n = PROOFSIZE;
  do {
    // follow until i == 0
    int j = i;
    // find j with same vs[j]
    for (int k = 0; k < PROOFSIZE; k++) {
      if (k != i && vs[k] == vs[i]) {
        if (j != i) return POW_BRANCH;
        j = k;
      }
    }
    if (j == i) return POW_DEAD_END;
    i = j;

    // find i with same us[i]
    for (int k = 0; k < PROOFSIZE; k++) {
      if (k != j && us[k] == us[j]) {
        if (i != j) return POW_BRANCH;
        i = k;
      }
    }
    if (i == j) return POW_DEAD_END;
    n -= 2;
  } while (i != 0);
  return n == 0 ? POW_OK : POW_SHORT_CYCLE;
}

cuckoo* init_graph(char *header, int headerlen) {
  unsigned char* hdrkey;
  SHA256((unsigned char *)header, headerlen, (unsigned char *)hdrkey);
  return &(cuckoo){.v = {u8to64(hdrkey, 0), u8to64(hdrkey, 8),
                         u8to64(hdrkey, 16), u8to64(hdrkey, 24)}};
}
