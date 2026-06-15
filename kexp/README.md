# K 

k is an APL language

## Examples

1. piwindow.k - Splits the input i/pi.txt into windows
2. rhys_grid.k - ngn/k version of https://nsl.com/oulipo/rhys_grid.k
3. patterns.k - Patterns in K
4. rec.k - Parse GNU recfiles with K

## Monadic and Dyadic Verbs

verbs are loosely functions, a single verb is generally overridden and depending on the number of arguments the meaning of the verb is defined one-argument (monadic), two-arguments (dyadic) eg) monadic # is count, dyadic # is reshape

```
/ Variable assignment
a: 1 2 3

/ Vector with single element
b: ,1

/ Count vector
#a

/ First
*a

/ Seq 1..10
!10

/ Reshape
2 5#!10
(2;5)#!10

/ Reshape done with null
0N 5#!10

/ Indexing
a 1
a@1
a[1]

a 1 0
a@1 0
a@(1;0)
a [1 0]


/ Indexing cont
c:2 3 5#!30
c[1;;]       / layer 1, all rows, all columns
c[;;1]       / all layers, all row, 1st col
c[;2;1]       / all layers, row 2, 1st col

v:1 5 3 2 8 7 0
v[4]: -2                  / change element as pos 4 to -2
v

/ Atomic addition
:m:(3 1 2;5 4 3;(8 1 2;2 3 4))
m+11                    / Add 11 to each atom (element)
```

## Adverbs

adverbs determine how a verb is applied to its arguments

```
\ Map

\ Map # into list
#'(1 2 3; 4 5 6; 1 3 4)

\ fold
+/1 2 3 4

\ scan
+\1 2 3 4

```

## Interesting Insights

1. The adverb `'` is a map and collects return values into a list.

## References

1. https://xpqz.github.io/kbook
