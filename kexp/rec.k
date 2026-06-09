/ rec.k - parse a GNU recutils file into a list of dictionaries
/ usage:
/   \l rec.k
/   x: 1:"i/test.rec"   / read the file as a string
/   d: rec.recorig x    / -> list of dicts, one per record
/   d[0]@,"b"           / look up field "b" of record 0 (enlist the string key)
/   *d[0]@,"b"          / ... as a bare value rather than a 1-item list

\d rec

ESC:  "\""
FLD:  ":"
ROW:  "\n"
CONT: "+"

tl:{(+/&\x=" ")_x}                        / drop leading spaces
kv:{i:x?FLD; (i#x; tl (1+i)_x)}           / Split key, value; find on FLD; select key and value separately
ct:{r:1_x; (" "=*r)_r}                     / strip leading "+" and one optional space from a continuation line
mg:{$[CONT=*y; (-1_x),,(*|x),ROW,ct y; x,,y]} / merge "+" continuation lines into the previous line

sp:{[s;x](1&!#l)_'(l:0,&(2!+\x=ESC)<x=s)_x} / split (drop sep from every chunk after the first)

recorig:{r:sp[ROW;x]; c:{x@&0<#'x}'(0,&0=#'r)_r; {(!).+x}'kv''{()mg/x}'c@&0<#'c} / parse REC -> list of dicts

\d .
