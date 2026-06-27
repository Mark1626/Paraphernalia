/ GNU recfile parser in ngn/k - Mark1626 26.06.26
/ rec.k - parse a GNU recutils file into k tables

\d rec

ESC:  "\""
FLD:  ":"
ROW:  "\n"
CONT: "+"
CMT:  "#"

tl:{(+/&\x=" ")_x}                        / drop leading spaces
kv:{i:x?FLD; (`$i#x; tl (1+i)_x)}         / Split key, value; key as a symbol so records flip into a column table
ct:{r:1_x; (" "=*r)_r}                     / strip leading "+" and one optional space from a continuation line
mg:{$[CONT=*y; (-1_x),,(*|x),ROW,ct y; x,,y]} / merge "+" continuation lines into the previous line
cmt:{(0<#x)&CMT=*x}                        / is x a comment line ("# ...")

sp:{[s;x](1&!#l)_'(l:0,&(2!+\x=ESC)<x=s)_x} / split (drop sep from every chunk after the first)

/ REC text -> list of blocks, each a list of (`key; "value") pairs
parse:{r:{x@&~cmt'x}sp[ROW;x]; c:{x@&0<#'x}'(0,&0=#'r)_r; kv''{()mg/x}'c@&0<#'c}

recorig:{{(!).+x}'parse x}                 / single record set -> flat list of record dicts (`M)

/ -- multiple record sets (real GNU recfiles) --

isd:{"%"=*$**x}                            / is block x a descriptor? (its first field key starts with "%")
field:{[p;k]$[#p;(p@\:1)@&k=p@\:0;()]}      / values of pairs p whose key is k
cst:`int`real!(`I$';`F$')                  / %type keyword -> column caster
mkT:{p:" "\x; (`$","\*p;*1_p)}             / "F1,F2 int" -> (`F1`F2; "int")
types:{[dp]pp:mkT'field[dp;`"%type"]; $[#pp; (,/pp@\:0)!,/(#'pp@\:0)#'`$pp@\:1; (0#`)!0#`]} / field -> type keyword

tbl:{[blk]                                 / blocks of one record set -> `M table (typed)
  T:types[,/blk@&isd'blk];                 / type declarations from the descriptor
  R:{(!).+x}'blk@&~isd'blk;                / records as dicts
  $[#R;;:+(0#`)!()];                       / empty record set -> empty table
  K:?,/!'R;                                / union of field names (handles optional fields)
  D:K!+R@\:K;                              / column dict
  ck:(!T)@&(#!D)>(!D)?!T;                  / declared fields that actually occur
  +@[D;ck;:;{rec.cst[x]y}'[T@ck;D@ck]]}     / cast those columns, then flip to a table

read:{b:parse x; g:+\isd'b; B:{x@&z=y}[b;;g]'?g; / cut blocks into record sets at each %rec: descriptor
  nms:{f:field[,/x@&isd'x;`"%rec"]; `$$[#f;*f;""]}'B; / record-set name (`` for a leading anonymous set)
  nms!tbl'B}

\d .


/ Example

\l rec.k
x: 1:"i/test.rec"   / read the file as a string
d: rec.recorig x    / single record set -> flat list of record dicts (`M)
d[0]                / record 0 as a dict, e.g. `a`b!(,"1";"\"abc\"")
d[0]`b              / field "b" of record 0
d`a                 / whole of column `a across every record

r: rec.read x       / real recfile -> dict mapping `RecordSet -> table
!r                  / -> `Author`Book`Loan  (one per %rec: descriptor)
r`Book              / the Book record set as an `M table
r`Book`Year         / the Year column (cast to int per %type: Year int)
%type: int/real declarations are honoured; columns are cast accordingly.

relational querying with t/k:

\l t.k

r: rec.read@1:"i/library.rec"
select[r`Book;order[r`Book;`Year`Pages;`asc`desc]]
╭─────────────────────────────┬───────────┬─────┬───────────┬──────╮
│           `Title            │  `Author  │`Year│  `Genre   │`Pages│
│             `C              │    `C     │ `i  │    `C     │  `i  │
├─────────────────────────────┼───────────┼─────┼───────────┼──────┤
│         "Compilers"         │   "Aho"   │1986 │"textbook" │ 1009 │
│        "TAOCP Vol 2"        │  "Knuth"  │1969 │"textbook" │ 688  │
│           "SICP"            │ "Abelson" │1985 │"textbook" │ 657  │
│        "TAOCP Vol 1"        │  "Knuth"  │1968 │"textbook" │ 650  │
│"The Practice of Programming"│"Kernighan"│1999 │"reference"│ 267  │
│"The C Programming Language" │"Kernighan"│1978 │"reference"│ 228  │
╰─────────────────────────────┴───────────┴─────┴───────────┴──────╯

\
