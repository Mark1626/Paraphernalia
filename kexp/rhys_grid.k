grid:{(0N,_%#x)#x}        / reshape a flat list into an (inferred x sqrt) grid
story:{`0:(" "/x),"."}    / join phrases with spaces, add a period, print one line
diag:{x@'!#x}             / main diagonal: row i indexed at column i
reverse:{|'x}             / reverse every row (so diag picks the anti-diagonal)
random:{c:#x; (,/d,\:/:d:!#x)@(-c)?c*c}   / c distinct (row;col) cells, no repeats

a:(("setting sail";"in the morning";"when all are asleep";"i leave my life behind";"seeking another")
   ("with unseemly haste";"from the shore";"the moonlight fades";"like the smile of the only";"woman i can adore")
   ("i catch the wind";"with a net on a pole";"i am uncertain about";"lips that sigh and devour";"like a caged beast")
   ("blowing from the north";"the stranger greets me";"the dryness of wit and";"my final desgination";"heavy on my mind")
   ("colder than a hateful eye";"with a menacing wave";"the sea breaks over me";"and a mermaid kisses me";"but it is not you"))

/ story'a;                      / each row as a sentence
/ story'+a;                     / each column as a sentence (+ is flip/transpose)
/ story diag a;                 / the main diagonal
/ story diag reverse a;         / the anti-diagonal
{story a ./:random a}'!#a;    / #a random sentences, one per pass
