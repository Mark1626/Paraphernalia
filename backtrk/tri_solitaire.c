/* Triangular Solitaire solver.
 *
 * C translation of the memoized bit-pattern algorithm from
 *   Martin Richards, "Backtracking Algorithms in MCPL using Bit
 *   Patterns and Recursion", University of Cambridge Computer Lab.
 *
 * Board (15 holes):
 *
 *            a
 *           b c
 *          d e f
 *         g h i j
 *        k l m n o
 *
 * A move takes a peg-peg-hole triple in a line and turns it into
 * hole-hole-peg.  We start full except hole 'a' and count the number
 * of distinct sequences of 13 moves ending with a single peg at 'a'.
 *
 * Representation: a board state is one 32-bit word.
 *   - low  15 bits (mask Pbits): a 1 means "peg present"
 *   - bits 16..30  (the same bits shifted up by SH): a 1 means "hole"
 * Peg field and hole field are exact complements of each other on the
 * 15 used positions.  A move flips all 6 bits of its triple at once
 * with a single XOR, simultaneously updating both fields.
 *
 * Expected answer: 6816 distinct solutions.
 */

#include <stdio.h>
#include <stdlib.h>

/* ---- bit layout ------------------------------------------------------ */

#define SH    16          /* shift from peg bit to its hole bit          */
#define Pbits 0x7FFFu     /* the 15 peg bits                             */

/* Peg bits: P(i) = 1 << i, for positions a..o = 0..14 */
#define Pa (1u<<0)
#define Pb (1u<<1)
#define Pc (1u<<2)
#define Pd (1u<<3)
#define Pe (1u<<4)
#define Pf (1u<<5)
#define Pg (1u<<6)
#define Ph (1u<<7)
#define Pi (1u<<8)
#define Pj (1u<<9)
#define Pk (1u<<10)
#define Pl (1u<<11)
#define Pm (1u<<12)
#define Pn (1u<<13)
#define Po (1u<<14)

/* Hole bits: H(i) = P(i) << SH */
#define Ha (Pa<<SH)
#define Hb (Pb<<SH)
#define Hc (Pc<<SH)
#define Hd (Pd<<SH)
#define He (Pe<<SH)
#define Hf (Pf<<SH)
#define Hg (Pg<<SH)
#define Hh (Ph<<SH)
#define Hi (Pi<<SH)
#define Hj (Pj<<SH)
#define Hk (Pk<<SH)
#define Hl (Pl<<SH)
#define Hm (Pm<<SH)
#define Hn (Pn<<SH)
#define Ho (Po<<SH)

/* ---- memo table and per-peg dispatch --------------------------------- */

#define NSTATES (1u << 15)        /* 32768 possible peg configurations    */

static long  scorev[NSTATES];     /* memoized count; -1 = not yet known   */
static long  (*fnv[NSTATES])(unsigned brd);  /* sparse: peg-bit -> fn     */

static long try(unsigned brd);    /* forward declaration                  */

/* trymove: test one move and, if legal, recurse into the successor.
 *
 *   brd    current board (peg bits + hole bits)
 *   hhp    selects "hole, hole, peg" -- the cells that must be hole,hole,peg
 *   hpbits all 6 bits of the triple; XORing flips peg<->hole for each cell
 *
 * brd & hhp is non-zero iff one of the first two cells is actually a hole
 * or the third is actually a peg -- i.e. the move is NOT legal.
 */
static long trymove(unsigned brd, unsigned hhp, unsigned hpbits)
{
    if (brd & hhp)
        return 0;                 /* move illegal */
    return try(brd ^ hpbits);     /* make the move, explore successor */
}

/* One generator function per board position. Each lists that position's
 * candidate moves. Positions d, f, m have four; all others have two.
 * The hhp argument is H(start)+H(middle)+P(end); hpbits is all six bits. */

static long fa(unsigned b){return trymove(b,Ha+Hb+Pd,Pa+Ha+Pb+Hb+Pd+Hd)+
                                  trymove(b,Ha+Hc+Pf,Pa+Ha+Pc+Hc+Pf+Hf);}
static long fb(unsigned b){return trymove(b,Hb+Hd+Pg,Pb+Hb+Pd+Hd+Pg+Hg)+
                                  trymove(b,Hb+He+Pi,Pb+Hb+Pe+He+Pi+Hi);}
static long fc(unsigned b){return trymove(b,Hc+He+Ph,Pc+Hc+Pe+He+Ph+Hh)+
                                  trymove(b,Hc+Hf+Pj,Pc+Hc+Pf+Hf+Pj+Hj);}
static long fd(unsigned b){return trymove(b,Hd+Hb+Pa,Pd+Hd+Pb+Hb+Pa+Ha)+
                                  trymove(b,Hd+He+Pf,Pd+Hd+Pe+He+Pf+Hf)+
                                  trymove(b,Hd+Hg+Pk,Pd+Hd+Pg+Hg+Pk+Hk)+
                                  trymove(b,Hd+Hh+Pm,Pd+Hd+Ph+Hh+Pm+Hm);}
static long fe(unsigned b){return trymove(b,He+Hh+Pl,Pe+He+Ph+Hh+Pl+Hl)+
                                  trymove(b,He+Hi+Pn,Pe+He+Pi+Hi+Pn+Hn);}
static long ff(unsigned b){return trymove(b,Hf+Hc+Pa,Pf+Hf+Pc+Hc+Pa+Ha)+
                                  trymove(b,Hf+He+Pd,Pf+Hf+Pe+He+Pd+Hd)+
                                  trymove(b,Hf+Hi+Pm,Pf+Hf+Pi+Hi+Pm+Hm)+
                                  trymove(b,Hf+Hj+Po,Pf+Hf+Pj+Hj+Po+Ho);}
static long fg(unsigned b){return trymove(b,Hg+Hd+Pb,Pg+Hg+Pd+Hd+Pb+Hb)+
                                  trymove(b,Hg+Hh+Pi,Pg+Hg+Ph+Hh+Pi+Hi);}
static long fh(unsigned b){return trymove(b,Hh+He+Pc,Ph+Hh+Pe+He+Pc+Hc)+
                                  trymove(b,Hh+Hi+Pj,Ph+Hh+Pi+Hi+Pj+Hj);}
static long fi(unsigned b){return trymove(b,Hi+He+Pb,Pi+Hi+Pe+He+Pb+Hb)+
                                  trymove(b,Hi+Hh+Pg,Pi+Hi+Ph+Hh+Pg+Hg);}
static long fj(unsigned b){return trymove(b,Hj+Hf+Pc,Pj+Hj+Pf+Hf+Pc+Hc)+
                                  trymove(b,Hj+Hi+Ph,Pj+Hj+Pi+Hi+Ph+Hh);}
static long fk(unsigned b){return trymove(b,Hk+Hg+Pd,Pk+Hk+Pg+Hg+Pd+Hd)+
                                  trymove(b,Hk+Hl+Pm,Pk+Hk+Pl+Hl+Pm+Hm);}
static long fl(unsigned b){return trymove(b,Hl+Hh+Pe,Pl+Hl+Ph+Hh+Pe+He)+
                                  trymove(b,Hl+Hm+Pn,Pl+Hl+Pm+Hm+Pn+Hn);}
static long fm(unsigned b){return trymove(b,Hm+Hh+Pd,Pm+Hm+Ph+Hh+Pd+Hd)+
                                  trymove(b,Hm+Hi+Pf,Pm+Hm+Pi+Hi+Pf+Hf)+
                                  trymove(b,Hm+Hl+Pk,Pm+Hm+Pl+Hl+Pk+Hk)+
                                  trymove(b,Hm+Hn+Po,Pm+Hm+Pn+Hn+Po+Ho);}
static long fn(unsigned b){return trymove(b,Hn+Hi+Pe,Pn+Hn+Pi+Hi+Pe+He)+
                                  trymove(b,Hn+Hm+Pl,Pn+Hn+Pm+Hm+Pl+Hl);}
static long fo(unsigned b){return trymove(b,Ho+Hj+Pf,Po+Ho+Pj+Hj+Pf+Hf)+
                                  trymove(b,Ho+Hn+Pm,Po+Ho+Pn+Hn+Pm+Hm);}

/* try: number of distinct ways to reach the final state from `brd`.
 * Memoized on the 15-bit peg field. */
static long try(unsigned brd)
{
    unsigned poss  = brd & Pbits;          /* pegs still on the board */
    long     score = scorev[poss];

    if (score < 0) {                       /* not seen this state before */
        score = 0;
        while (poss) {
            unsigned bit = poss & (unsigned)(-poss);  /* lowest set bit */
            poss -= bit;
            score += fnv[bit](brd);        /* explore that peg's moves */
        }
        scorev[brd & Pbits] = score;       /* memoize */
    }
    return score;
}

static void initvecs(void)
{
    unsigned i;
    for (i = 0; i < NSTATES; i++) {
        scorev[i] = -1;
        fnv[i]    = NULL;
    }
    /* sparse dispatch table, indexed by a single peg bit */
    fnv[Pa]=fa; fnv[Pb]=fb; fnv[Pc]=fc; fnv[Pd]=fd; fnv[Pe]=fe;
    fnv[Pf]=ff; fnv[Pg]=fg; fnv[Ph]=fh; fnv[Pi]=fi; fnv[Pj]=fj;
    fnv[Pk]=fk; fnv[Pl]=fl; fnv[Pm]=fm; fnv[Pn]=fn; fnv[Po]=fo;
}

int main(void)
{
    unsigned start;
    long ways;

    initvecs();

    /* Base case: the final position is a single peg at 'a'. It is
       reached in exactly 1 way (the empty sequence of further moves). */
    scorev[Pa] = 1;

    /* Initial board: every hole has a peg except 'a', which is a hole.
       Peg field = all positions except a; hole field = just a. */
    start = (Pbits & ~Pa)               /* pegs b..o            */
          | Ha;                         /* hole a               */

    ways = try(start);

    printf("Number of solutions = %ld\n", ways);
    return 0;
}
