// sql parser

/ tokenize (atw/sa)

C:";*aq/:"

/          !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
A:(&33),C?"*q****q;;****a*aaaaaaaaaa:;*****aaaaaaaaaaaaaaaaaaaaaaaaaa;/;*aaaaaaaaaaaaaaaaaaaaaaaaaaaa;*;*"

B:(C,"br")?B:1_'B:(";;*aq**"
                   "*;*aq*:"
                   "a;*bq**"
                   "qrrr:/r"
                   "/rrrrrr"
                   ":;*aq**"
                   "b;*bq**"
                   "rrrr:/r")

words:{{@[x;&"and"~/:x;:[;,","]]}(,," ")_(4>B\A@)^"\n\t\r"_x}

/ shift-reduce transitions + valence + operators

/   o,nac|&=+*^_123;()ZN
T:("rssssssssssssssrsrrn 2 on using inner left right full"
   "rrsssssssssssssrsrrn 2 , and or"
   "rrrssssssssssssrsrrn 1 not"
   "rrrrsssssssssssrsrrn 2 as"
   "rrrrrssssssssssrsrrn 1 rank count countd first last sum min max avg var dev enlist"
   "rrrrrrsssssssssrsrrn 2 |"
   "rrrrrrrssssssssrsrrn 2 &"
   "rrrrrrrrsssssssrsrrn 2 = < > ~= ~< ~> ~= <= >= in"
   "rrrrrrrrrssssssrsrrn 2 + -"
   "rrrrrrrrrrsssssrsrrn 2 * /"
   "rrrrrrrrrrrssssrsrrn 2 ^"
   "rrrrrrrrrrrrsssrsrrn 1 ? ~"
   "rrrrrrrrrrrrrssrsrrn 1 date row_number"
   "rrrrrrrrrrrrrrsrsrrn 2 intable in extract"
   "rrrrrrrrrrrrrrrrsrrn 3 if between"
   "fffffffffffffffffffn 1 ;"
   "ssssssssssssssssssxn 0 ("
   "fffffffffffffffffffn 2 )"
   "sssssssssssssssssyan 0")

F:[n:{cp'(1_x;y;x[,0],z)}                                     / noun
   s:{cp'(1_x;x[,0],y;z)}                                     / shift
   r:{cp'(x;1_y;(,y[,0],|n#z),(n:va[y;z])_z)}                 / reduce
   f:{cp'(x;va[y;y]_y;z)}                                     / function
   a:{(x;y;z)}                                                / accept
   y:{LEFT}                                                   / missing left
   x:{RIGHT}]                                                 / missing right

T:{1_'(&x=" ")^x:" ",x}'T                                     / split table
O:1_'T;V:(.*)'O;O:(-1_1_'O),,E:,," ";T:T[;0]                  / O V E T

/ digits, noise words, keywords, windows

D:".0123456789"
J:$`by`join`outer`over
K:("select";"distinct";"partition";"from";"where";"group";"having";"order";"limit")

expr:{$[~#x;x;1=#x;*x;cc mm mi@**|co/:(me cs ls x,E;E;())]}   / expression parser
cc:{$[`C=@x;x;cc'cf x]}                                       / flatten lists
ls:{$[#i:&{(y~,"(")&"*"~*x}':x;@[x;i;:[;,"_"]];x]}            / L* -> L_
cs:{$[#i:&{(y~"count")&"*"~*x}':x;@[x;i;:[;,"_"]];x]}         / count * -> count _
me:{@[x;&{(x~,"-")&|/y in,/(,,")")_O}':x;:[;,"?"]]}           / v - n -> v ? n
mi:{$[`C=@x;(x;,"-")x~,"?";mi'x]}                             / ? -> -
mm:{$[~`:=@x;x;~2=#x;mm'x;((*x)~,"-")&(*/:x 1)in D;,/x;x]}    / - D -> -D
co:{(F`$T .|op'x[!2;0]). x}                                   / continue
cp:{$[#x;x;()]}                                               / collapse prototype
op:{(|/'x~/:/:O)?1}                                           / operator
va:{$[(#y)<n:V op@*x;VALENCE;n]}                              / valence

parse:{as di ra sd us it J_words[\x]}                         / sql parser
di:{$[1=#x;*x;x]}                                             / disclose  
it:{$[#j:&"in"~/:x i:-2+1_&x~\:*K;@[x;i j;:[;"intable"]];x]}  / in select -> intable select
us:{$[#i:&{(y~*K)&"*"~*x}':x;@[x;i;:[;,"_"]];x]}              / select * -> select _
sd:{$[#i:&{(y;x)~2#K}':x;x@&~(!#x)in i-1;x]}                  / select distinct -> distinct
pe:{{,(ce').(,/x;expr'y)}.+0 1^/:(|/K~/:\:)^x}                / parse expression
rd:{$[k:fs x;,/@[k^x;1;ra 1_-1_];x]}                          / recur down
fs:{$[(#x)=a:(lp[x]&1_(|/x~\:/:K),0)?1;!0;es[a_x]a]}          / first nested keyword
es:{0,y,$[(#x)=a:((-1_0,+\rp x)-+\lp x)?0;();y+a]}            / end select
lp:(,"(")~/:;rp:(,")")~/:                                     / L and R match
ce:{(x;cf y)}                                                 / , elimination
cf:{$[1=#x;x;ci x]}                                           / , flatten
ci:{$[3>#r:cr[x]x;x;(,,","),r]}                               / , introduction
cr:{$["or"~*y;();","~**y;cr[x;y 1],,y 2;,y]}                  / , recursion
ra:pe@rd/:                                                    / recur across
as:{$[`C=@x;x;|/(*x)~/:2#K;@[x;1;fg];as'x]}                   / f as g recursive
fg:{$[","~**x;(,,","),fh'[1_x;$!-1+#x];fh[x;,"0"]]}           / f as `$n
fh:{$["as"~*x;x;`C=@x;("as";x;x);("as";x;y)]}                 / field or exp

/ map-reduce

enlist:{,x};delist:{,/x}
G:("avg";"var";"dev")!({(,"%";("sum";x);("count";x))};{("{(x%z)-y*y%:z}";("sum";(,"*";x;x));("sum";x);("count";x))};{("sqrt";("var";x))})
gx:{$[`.=t:@x;x;`C=t;x;(*x)in!G;gx'G[*x]. 1_x;gx'x]}
H:("first";"last";"sum";"min";"max";"count";"countd";"{x}";"enlist")!("first";"first";"sum";"min";"max";"sum";"sum";"{x}";"delist")
hx:{$[`C=@x;();`.=@*x;,/hx'1_x;(*x)in!H;,x;,/hx'x]}
ix:{$[`C=@y;y;~(*y)in!H;ix[x]'y;(k:ex . y)in!x;x k;ix[x]'y]}

/ sort, index, select, table-at

ord:{[t;f;s]t@{x y z x}/[!#t;s;+t[;f]]}
idx:{[t;i]$[#i;t@&&/t tat/:i;t]}
sel:{[t;f]$[#f:f[;&`C=@'f 1];+f[0]!shp@tat[t]'f 1;t]}
shp:{[d](|/#'(),/:d)#'d}
tat:{x@. y}

eval:{[s]
 z:table of[s]"from"              / table(s)
 c:(!*)'z:virtual z;v:*z;z:z 1    / (virtual;real)
 g:`$parts of[s]"group"           / group by
 d:"distinct"~**s                 / select | distinct
 f:field[,/c]of[s](2#K)d          / select columns
 w:of[s]"where"                   / where-clause
 i:which[$*c]w                    / virtual conditions
 z:restrict/[(v;z);i]             / restrict 
 i:i_where w                      / non-virtual conditions
 z:mapred[s;i;f;g]. z             / map-reduce
 z:final[s;f;d]z                  / final processing
 z}

/ virtual column processing

virtual:{[z]$[`A=@z;(, [];,z);. z]}
restrict:{[z;i]z@\:&(*z)@. i}
which:{[v;w]$[~#v;();~#w;();","=**w;,/v which/:1_w;|/v in w;where w;()]}

/ :f[x] <-> (f;x)

ex:{":",x,"[",y,"]"}
xe:{$[(#x)=i:x?"[";1_x;(1_i#x;1_-1_i_x)]}
xf:{$[#x;xe y;("{x}";1_y)]}

/ map-reduce

mapred:{[s;i;f;g;v;z]
 a:(gx xf[g]@)'!/f[;&`C=@'f 1]
 d:?ex .',/hx'a
 e:(!a)#g_!v
 d,:{":first[",x,"]"}'$e
 c:+xe'd
 b:((`$"f",'$!(#*c)-#e),e)!d
 z:map[a;b;i;g;v]z
 z:red[a;b;g;v;*c]z
 z}

map:{[a;b;i;g;v;t],/map1[a;b;i;g]'[v;t]}
map1:{[a;b;i;g;v;t]m[idx[+v,+t]i;v;g].. b}

red:{[a;b;g;v;h;t]
 z:red1[v;g;ex .'n!H[h]{(x;y)}'$n:!b;t]
 z:+z@/:.'":",'unparse'$ix[!/|. b]'a
 z}

red1:{[v;g;f;z]m[z;[];g].. f}

m:{[t;v;g;e;f]$[mcount f;mfreq[t;g]e;mfunc[t;v;g;e]f]}
mfreq:{[t;g;e]+{@[+y;x;:;z]}[*e].. freq g#t}
mcount:{[f]$[1<#f;0;"count"~*xe@*f]}
mfunc:{[t;v;g;e;f]$[#g;mpart;mflat][v;g;t;e;f]}
mpart:{[v;g;t;e;f]$[#a:(!v)_g;msome a;mnone][v;t;e]f}
mnone:{[v;t;e;f]unkey(+,'v)!+e!+(.'f){y@/:x}/:,t}
msome:{[a;v;t;e;f]unkey(+v,+*k)!+e!+(.'f){y@/:x}/:t@*|k:.=a#t}
mflat:{[v;g;t;e;f]+(),/:e!(.'f)@\:t}

final:{[s;f;d;z]
 z:idx[z]where of[s]"having"
 z:$[#i:order of[s]"order";ord[z]. i;z]
 z:$[#i:limit of[s]"limit";(i&#z)#z;z]
 z:$[#f:f[;&~`C=@'f 1];function/[z;*f;f 1];z]
 z:$[d;?z;z]
 z}

function:{[z;s;k]W[*k][z;s]k 1}

window:{[z;a]
 z:$[#p:`$parts of[a]"partition";z[=p#z][];,z]
 z:$[#o:order of[a]"order";{ord[y]. x}[o]'z;z]
 z}

rownumber:{[z;s;a]
 z:window[z]a
 z:,/{+@[+y;x;:;1+!#y]}[s]'z
 z}

W:(,"row_number")!,rownumber

intable:{x in*+y}

table:{$[~`C=@x;eval x;","~**x;table'1_x;. x]}
field:{$[~#y;y;","~**y;,/'+field[x]'1_y;"_"~**y 1;(x;":",'$x);((),*y 1)in!W;(,`$y 2;,y 1);(,`$y 2;,":",unparse$["count"~*y 1;(*y 1;$*x);y 1])]}
where:{$[~#x;x;","~**x;,/where'1_ x;,":",unparse x]}
parts:{$[~#x;x;","~**x;,/parts'1_x;,unparse x]}
order:{$[~#x;x;`C=@x;+,(`$x;<);(,",")~*x;,/'+order'1_x;+,(`$x 1;(>;<)("desc";"asc")?x 2)]}
limit:{$[~#x;x;. x]}

unparse:{$["select"~**x;"eval[",(`k x),"]";`C=@x;x;","=**x;"(",(1_,/(";",unparse@)'1_x),")";(*x),"[",(1_,/(";",unparse@)'1_x),"]"]}
of:{$[(#x[;0])>i:x[;0]?y;x[i;1];()]}

tables:{[v;t]$[#v;(+((),v)!+p)!t ./:p:paths t;t]}
paths:{$[`A=@*x;+,!x;,/(!x),/:'*|. paths'x]}

sql:eval parse@

/ examples

/ https://www.c-sharpcorner.com/blogs/rownumber-function-with-partition-by-clause-in-sql-server1#:~:text=ROW_NUMBER()%20Function&text=The%20Row_Number%20function%20is%20used,number%20of%20the%20subsequent%20rows.
/ parse"select s,d,p,r,f,row_number() over (partition by d,s order by n) as foo from x"

abc:[[]f:0 1 2 10 20 30;g:101 102 103 104 3 4;h:6 5 4 3 2 1;i:2 3 8 2 9 0;j:1 1 2 2 2 2;k:3 3 3 3 4 5]
ABC:tables[`x]`a`b`c!3#,abc

tab:2018.01.01 2018.01.02!([[]f:1 1 2;g:10 11 12;h:0 1 2;i:1 2 1;j:0 0 1];[[]f:1 2 2 3;g:20 21 22 23;h:0 1 2 3;i:0 1 0 1;j:0 0 1 1])
sd1:100 200 300!3#,tab / store by date
sd2:3 4!2#,sd1
t:tables[`s`d]sd1

emp:[[]id:1 2 3 4 5 6 7;name:$`ro`sm`me`ra`sm`me`ra;salary:16 5 2 12 13 9 20]
emp2:[[]id:1 2 3 4 5 6 7;name:`ro`sm`me`ra`sm`me`ra;salary:16 5 2 12 13 9 20]

sql"select f+g from (select f,g from abc where i<9)"
sql"select f+1,g-1 from abc where h>0,g in (select g,h+1 from abc)"
sql"select distinct f,g,h from t"
sql"select count * from ABC group by j"
/ sql"select j,sum f from abc group by k"
/ sql"select s,d,j,sum f from t group by s,d,j"
sql"select s,d,sum f from t group by s,d"
sql"select s,j,sum f from t group by s,j"
sql"select s,j,d,sum f from t group by s,j"
sql"select s,d,sum f from t group by s,d"
sql"select s,d,j,sum f from t group by s,j"
sql"select id from emp where name in(\"ro\",\"sm\")"
sql"select id from emp2 where name in(`ro,`sm)"
sql"select k,countd j from abc group by k"
sql"select f,g from t where g<20"
sql"select j,sum f,sum g from abc group by j"
sql"select s,d,j,sum f from t where s>100,d=2018.01.01,f>0 group by s,j"
sql"select j,sum f,sum g from abc group by j"
sql"select s,d,j,sum f from t where s>100,d=2018.01.01,f>0 group by s,j"
sql"select j,sum f from abc group by j"
sql"select enlist f from abc"
sql"select j,sum f from abc group by j"
sql"select sum f from abc"
sql"select *,row_number() over (order by name) as rn from emp"
sql"select f,g from t where g<20"
sql"select f from abc"
sql"select f from t"
sql"select sum f from t"
sql"select j,sum f from abc"
sql"select j,sum f from abc group by j"
sql"select *,row_number() over (order by name) as rn from emp"
sql"select *,row_number() over (partition by name order by name) as row from emp"
sql"select count(*) from abc"
sql"select sum f+1 from t group by h,i" 
sql"select count * from t group by i,j"
sql"select f,sum g,sum h,avg h from t group by f"
sql"select count * from t group by i,j"
sql"select i,j,count g from t group by i,j"
sql"select * from abc"
sql"select j,count * from t group by j"
sql"select sum f+1 from abc group by j,k"
sql"select sum f+1,count g from abc group by j"
sql"select f+1 from abc"
sql"select sum g,sum h from t group by f"
sql"select sum g + 1 as a,sum h as b,avg h as c from t group by f"
sql"select sum f,min f from t group by h"  
sql"select count * from t group by f"
sql"select sum g+1,sum h,avg h from t group by f"
sql"select sum f,sum g from abc group by j"
sql"select sum f, f from abc"
sql"select count *, sum f+1 from abc group by j"
sql"select * from abc" 
sql"select f,g from abc where f<30"
sql"select f,g from abc where f<30, g>0"
sql"select sum f+1 as fsum,sum g-1 as gsum from abc group by j having fsum>4 order by fsum desc"
sql"select f,g from abc where f<30"
sql"select sum f+1 as fsum, sum g-1 as gsum from abc group by j"
sql"select sum f+1 as fsum,sum g-1 as gsum from abc where f<30 group by j,k"
sql"select sum f+1 as fsum,sum g-1 as gsum from abc group by j having fsum>4 order by fsum desc"
sql"select sum f+1 as fsum,sum g-1 as gsum from abc group by j order by fsum desc"
sql"select f as ff,g+1 as gg,h from abc where f<30,g>100 order by f asc,g desc limit 5"
sql"select f as ff,g+1 as gg,h from abc order by h as asc,i as desc limit 5"  
sql"select f as ff,g+1 as gg,h from abc limit 5"  
sql"select f as ff,g+1 as gg,h from abc"  
sql"select * from abc" 

\

parse"select x from a left b on a=b.x"
parse"select x from (select x--3 from t)"
parse"select distinct x,sum y from z"
parse"select x,sum y from t left outer join u group by x order by x"
parse"select x from (select y from t) where x"
parse"select e.ename, d.dname from (employees as e left join departments as d)on (e.dept=d.dept)"
parse"select o.o, o.i, o.c, o.r, p.p, c.c from orders as o inner join people as p on o.r = p.r inner join customers as c on o.i=c.i"
parse"select o.o, o.i, o.c, o.r, p.p, c.c from o inner join p on o.r = p.r inner join c on o.i=c.i"

\

tt:{tu/[x;$[","~**x[1;1];x[1;1];,x[1;1]]]}                    / amend t as t
tu:{$["as"~*y;tv[y 1;y 2]x;x]}                                / if t as t
tv:{$["from"~*z;z;~`C=@z;tv[x;y]'z;ty[z]y;tx[z]x;z]}          / find t.f
ty:{y~(x?".")#x}                                              / y~x of x.z
tx:{y,(x?".")_x}                                              / x.z -> y.z

M:("sum";"min";"max";"count")
is:{(x in M)&~"["in y}
agg:{[t;f;d]$[f~(count);freq t;t f'd]}

/ rotate
r:{,/|(0,(#y)mod x)^y}

\

https://shakti.sh/document/sql.d?eula=shakti.com/license

\

k universal database uses Real/sql -- fast easy expressive

Real/sql[1974 atw@ipsa.ca] is much better than ANSI/sql[1992..]

Simple query: Real and ANSI the same
 select sym, price from t where size>100

Groupby query: Real-easy ANSI-annoy
 Real: select[G] A from T where B
 ANSI: select G, A from T where B group by G order by G

Complex query: Real-easy ANSI-awful
 asofjoins       select from t,q where price<bid
 first/last      select last bid from quote where sym=`A
 deltas/sums     select from t where 0<deltas price
 arithmetic      x+y e.g. combine markets through time
 foreignjoin     select order.cust.nation.region ..

Example: TPC-H National Market Share Query 8 http://www.qdpma.com/tpch/TPCH100_Query_plans.html
What order.year market share does supplier.nation BRAZIL have for order.customer.nation.region AMERICA and part.type STEEL?

 Real:
 select[order.year]revenue avg supplier.nation=`BRAZIL from t where order.customer.nation.region=`AMERICA, part.type=`STEEL

 ANSI:
 select o_year,sum(case when nation = 'BRAZIL' then revenue else 0 end) / sum(revenue) as mkt_share from (
  select extract(year from o_orderdate) as o_year, revenue, n2.n_name as nation 
  from t,part,supplier,orders,customer,nation n1,nation n2,region
  where p_partkey = l_partkey and s_suppkey = l_suppkey and l_orderkey = o_orderkey and o_custkey = c_custkey and 
   c_nationkey = n1.n_nationkey and n1.n_regionkey = r_regionkey and r_name = 'AMERICA' and 
   s_nationkey = n2.n_nationkey and o_orderdate between date '1995-01-01' and date '1996-12-31' and p_type = 'STEEL') as all_nations 
 group by o_year order by o_year;

Comparison:     Real                  ANSI(sqlserver/oracle/db2/sap/teradata/..)
 install           1 second        100,000 second
 hardware          1 milliwatt     100,000 milliwatt
 software        140 kilobyte    8,000,000 kilobyte  (+ 10,000,000kilobyte O/S)
 mediandb  1,000,000 megarow             1 megarow

https://docs.microsoft.com/en-us/sql/database-engine/install-windows/install-sql-server?view=sql-server-ver15





