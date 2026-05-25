/* Cardinality of the domain D3.
 *
 * C translation of the program in the section "The Cardinality of D3"
 * of Martin Richards, "Backtracking Algorithms in MCPL using Bit
 * Patterns and Recursion", University of Cambridge Computer Laboratory.
 *
 * Background (Stoy, "Denotational Semantics", pp.113-115):
 *   D0 = { bottom, top }                  -- 2 elements
 *   D1 = monotonic functions D0 -> D0     -- 3 elements
 *   D2 = monotonic functions D1 -> D1     -- 10 elements A..J, a lattice
 *   D3 = monotonic functions D2 -> D2     -- size to be computed
 *
 * A function f in D3 is the tuple (f(A),...,f(J)); it is counted iff
 * monotonic:  x <= y => f(x) <= f(y)  in the D2 lattice.
 *
 * Each D2 element is one bit (A = bit 0 ... J = bit 9). tab[x] is the
 * bit set of all elements <= x in D2 (its down-set). Slots are chosen
 * top-down J..A; constraint sets for slots with two lattice parents are
 * intersected with bitwise AND.
 *
 * This is a literal translation: `try` is one function, exactly as in
 * the MCPL, and each fX mirrors its MCPL definition line for line.
 *
 * Expected answer: 120549.
 */

#include <stdio.h>

enum { A=1, B=1<<1, C=1<<2, D=1<<3, E=1<<4,
       F=1<<5, G=1<<6, H=1<<7, I=1<<8, J=1<<9 };

static int  tab[J + 1];
static long count = 0;

static void init_lattice(void)
{
    tab[J] = A+B+C+D+E+F+G+H+I+J;
    tab[I] = A+B+C+D+E+F+G+H+I;
    tab[H] = A+B+C+D  +F  +H;
    tab[G] = A+B+C+D+E+F+G;
    tab[F] = A+B+C+D  +F;
    tab[E] = A+B  +D+E;
    tab[D] = A+B  +D;
    tab[C] = A+B+C;
    tab[B] = A+B;
    tab[A] = A;
}

/* Each slot function takes its constraint set(s) and a "next slot".
 * In the MCPL all fX have the shape  FUN fX : a[,b] => try(fNext, ...).
 * `try` iterates the choice set, mapping the chosen element through
 * tab, and passes a second constraint set `b` straight through.
 *
 * We give every fX the C signature (int a, int b); one-argument MCPL
 * functions simply ignore b. `try` is parameterised by the next fX. */

typedef void (*slot)(int a, int b);

/* try(f, a, b): for each element x in set a, choose it, then call
 *   f(tab[x], b)
 * exactly as the MCPL  FUN try : f,a,b => UNTIL a=0 DO {...; f(tab!x,b)} */
static void try_(slot f, int a, int b)
{
    while (a) {
        int x = a & -a;        /* lowest set bit: one candidate element */
        a -= x;
        f(tab[x], b);          /* tab[x] for the chosen slot; b passed on */
    }
}

static void fA(int a, int b);
static void fB(int a, int b);
static void fC(int a, int b);
static void fD(int a, int b);
static void fE(int a, int b);
static void fF(int a, int b);
static void fG(int a, int b);
static void fH(int a, int b);
static void fI(int a, int b);
static void fJ(int a, int b);

/* Line-for-line with the MCPL fX definitions. Unused params are the
 * MCPL "absent argument" cases and are explicitly voided. */

static void fJ(int a, int b)            /* FUN fJ : a    => try(fI,a)      */
{ (void)b; try_(fI, a, 0); }

static void fI(int a, int b)            /* FUN fI : a    => try(fH,a,a)    */
{ (void)b; try_(fH, a, a); }

static void fH(int a, int b)            /* FUN fH : a,b  => try(fG,b,a)    */
{ try_(fG, b, a); }

static void fG(int a, int b)            /* FUN fG : a,b  => try(fF,a&b,a)  */
{ try_(fF, a & b, a); }

static void fF(int a, int b)            /* FUN fF : a,b  => try(fE,b,a)    */
{ try_(fE, b, a); }

static void fE(int a, int b)            /* FUN fE : a,b  => try(fD,a&b,b)  */
{ try_(fD, a & b, b); }

static void fD(int a, int b)            /* FUN fD : a,b  => try(fC,b,a)    */
{ try_(fC, b, a); }

static void fC(int a, int b)            /* FUN fC : a,b  => try(fB,a&b)    */
{ try_(fB, a & b, 0); }

static void fB(int a, int b)            /* FUN fB : a    => try(fA,a)      */
{ (void)b; try_(fA, a, 0); }

static void fA(int a, int b)            /* FUN fA :      => count++        */
{ (void)a; (void)b; count++; }

int main(void)
{
    init_lattice();
    /* FUN start: ... try(fJ, A+B+C+D+E+F+G+H+I+J) */
    try_(fJ, A+B+C+D+E+F+G+H+I+J, 0);
    printf("Number of elements in D3 = %ld\n", count);
    return count == 120549 ? 0 : 1;
}
