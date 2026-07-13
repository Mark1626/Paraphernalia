/ K version of https://github.com/blinry/habitctl

\d habit

hbts:0:"/Users/nimalanm/.habitctl/habits"
log:"/Users/nimalanm/.habitctl/log"
olog: 0:log

/ util
tu:{((-12!14-M)+-4716+-1461!e;M:1+12!2+-153!h;1+-5!153!h:2+5*-4!1461!e:3+4*J+1363+-4!3*-146097!274277+4*J:2440588+*t),1_t:0 24 60 60 1000000\_x}
tds:{p:{-2#"0",$x};($x[0]),"-",(p x[1]),"-",(p x[2])}
n:(tds tu `t[])

/ habit files
ept:{x@&0<#'x}
cmt:{x@&~"#"=(+(1#'x))[0]}
parse:{d:cmt ept x;{(`I$1_2#x; 3_x)}'d}

/ h ask and log
ask: {`0:x[1],"\t[y/n/s/⏎]"; a:0:`;n,"\t",x[1],"\t",a[0]}

spk: (" ";"▁";"▂";"▃";"▄";"▅";"▆";"▇";"█")
lg: (`u;`nd;`d;`s;`sk;`sku;`w)!(" ";" ";"━";"─";"•";"·";"!")

/ h log
h: parse hbts
ht: +(`r;`h)!(+h)
lt: +(`d;`h;`s)!+{3#x,("";"";"")}'("\t"\'ept olog)
twa:{[w;b]w&:#b;s:+\b;0<s-(w#0),(-w)_s}

D:lt@\:`d; H:lt@\:`h; S:*'lt@\:`s
ds:tds'tu'`t[]-86400000000*|!101  / 101 day display window
row:{[hb]i:&H~\:hb; ((S i)," ")(D i)?ds}
m:row'ht@\:`h

rs:ht@\:`r                                                        / habit frequencies
dn:{[r;v]("y"=v)|(r>0)&("n"=v)&twa[r;"y"=v]}'[rs;m]               / done|satisfied
sk:{[r;v]("s"=v)|(r>0)&("n"=v)&twa[r;"s"=v]&~twa[r;"y"=v]}'[rs;m] / skipped|skipified
i:&rs>0                                                           / tracked-only habits excluded
sc:{$[y=z;0;(100*x)%y-z]}'[+/dn i;#i;+/sk i]                      / score per day

sym:"ysn "!(lg`d;lg`sk;lg`nd;lg`u)

ll:{`0:(26#" "),,/spk 8&_sc%9;`0:{(-25$x)," ",,/sym y}'[ht@\:`h;m];`0:"Score today: ",($_*|sc),"%"}
tt:{log 0:olog,"\n",(ask'h)}

a:`argv[]
$[2=#a;tt();(3=#a)&"log"~a[2];ll();{`0:"usage: k h.k [log]";`exit@1}[]]

\d .
