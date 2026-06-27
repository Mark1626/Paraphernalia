/ Modified version of https://nsl.com/k/t.k for ngn/k Mark1626 2026.6.22
/ relational operations - sa 2005.6.15
/ .
/ a table here is an ngn/k table (type `M: a flipped dict cols!data of equal-length
/ column vectors)

/ table[f;n;m]    returns a table of cardinality n with cols f whose values are drawn from m
/ insert[t;d]     returns t with d appended
/ update[t;f;i;d] returns t with cols f updated at i with d
/ delete[t;i]     returns t with rows i deleted
/ select[t;i]     returns t restricted to rows i
/ project[t;f;g]  returns t with cols g of t projected to f
/ group[t;k;f;o]  returns t with keys k and aggregations o[1]f[1] ... o[n]f[n]
/ where[t;f;o]    returns indices of rows where o[1]f[1] and ... and o[n]f[n]
/ order[t;f;o]    returns indices of rows sorted by o[1]f[1] within ... within o[n]f[n]
/ link[t;u;k;l]   returns the join of the data table t with the key table u

/ helpers

in:{~0N=y?x}                         / membership: is each x in y  (was x _lin y)
del:{$[#y;x@&~in[!#x;y];x]}           / drop elems of x at indices y (was _di)
drw:{[n;y]$[in[@y;`B`I`F`C`S];y@n?#y;y~0;n?1.0;n?y]}   / draw n vals from spec y
avg:{(+/x)%#x}
AGG:`count`sum`avg`list!(#:;+/;avg;:)  / named aggregators (was symbol apply)
ORD:`asc`desc!(<:;>:)                  / named sort directions

/ relational operators

table:{[f;n;m]+((),f)!drw[n]'(),m}     / (),f keys + (),m specs so a single-col table flips (atom key -> 'rank)
select:{[t;i]@[t;!t;@[;i]]}
project:{[t;f;g]+((),f)!t@/:(),g}      / @/: indexes per col so single-col projects work (t@,`c is 'rank)
nm:{[o;f]`$($o),"_",$f}
group:{[t;k;f;o]K:t@/:(),k;g:=+K;G:g@!g;+(((),k)!+?+K),(nm'[(),o;(),f])!{x'y}'[AGG(),o;(t@/:(),f)@\:G]} / (),+@/: handle single key/agg col
where:{[t;f;o]c:t@/:(),f;{x y z x}/[!#*c;(),o;c]} / o: predicates returning &-indices; @/: + (), make single col a 1-list
order:{[t;f;o]where[t;f;ORD o]}                 / o: `asc`desc syms -> grade verbs
link:{[t;u;k;l]dt:+t;+dt,(+u)[;(+(+u)@(),l)?/:+dt@(),k]}   / join in dict-space, reflip
insert:{[t;d]@[t;!t;,;d]}
update:{[t;f;i;d].[t;(f;i);:;d]}
delete:{[t;i]@[t;!t;del[;i]]}
show:{$[`M=@x;x;+x]}                            / print a table as-is, flip a bare dict

/
/ Example

t:table[`f`g`h`i`j;10;(`a`b`c`d`e;7;0;0;0)]
t:insert[t;(`a`b`b`a;3 5 6 2;0.1*!4;0.2*!4;0.3*!4)]
u:select[t;where[t;`f`g;({&in[x;`a`b]};&5=)]]
v:select[u;order[u;`g`f;`desc`asc]]
w:group[t;`f`g;`h`h`h`i;`count`sum`avg`sum]
x:select[w;order[w;`g`f;`desc`asc]]
p:project[u;`name`val;`f`g]
q:update[u;`g;!#u;90+!#u]                       / #u = row count of an `M table

`0:"t:\n";show t
`0:"\nu = select where f in a,b and g=5:\n";show u
`0:"\nv = u ordered by g desc, f asc:\n";show v
`0:"\nw = group by f,g:\n";show w
`0:"\nx = w ordered:\n";show x
`0:"\np = project u cols f,g -> name,val:\n";show p
`0:"\nq = update u: set g to 90,91,... :\n";show q

d:+`f`g!(`a`a`b`a`b`c`c`c`a`a;10?100)
K:+`f`h`i!(`a`b`c;100 200 300;1.1 2.2 3.3)

`0:"\nd:\n";show d
`0:"\nK:\n";show K
`0:"\nlink[d;K;`f;`f]:\n";show link[d;K;`f;`f]

\
