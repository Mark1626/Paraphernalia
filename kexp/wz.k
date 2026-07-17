/ Wizard adventure - port of Casting SPELs in LISP
/ Run: k wz.k
/ All game data lives in comments in this file - the file reads itself at
/ startup: /o object locations, /l location keys, /g action conditions,
/ and the trailing block comment holds the scenes keyed by \N.

l:"0"                                       / current location key
wl:0                                        / chain welded to bucket
fl:0                                        / bucket filled with water
fin:0                                       / game over

ept:{x@&0<#'x}                              / drop empty lines
p:{(&x~\:80#"=")_x}                         / cut a scene at the ==== lines
dc:{[q](#q)_'r@&((#q)#'r,\:(#q)#"\0")~\:q}  / lines of "/x " data comments, prefix dropped
cm:{" "\'dc x}                              / word lists of "/x " data comments

/ Game Data
r:0:*|`argv[]                               / this file
gd:(&t~\:80#"-")_t:(1+*&r~\:,"/")_-1_r      / \N scenes of the trailing comment

/o whiskey-bottle living-room
/o bucket living-room
/o chain garden
/o frog garden

/l living-room 0
/l garden 1
/l attic 2

/g weld chain bucket 2
/g dunk bucket well 1
/g splash bucket wizard 0

/h Help:
/h look
/h walk <dir> / go <dir>
/h pickup <obj> / get <obj>
/h inventory / i
/h weld <object> <object>
/h dunk <object> <object>
/h splash <object> <object>
/h quit

t:cm"/l ";lk:(`$t[;0])!*'t[;1]              / location name -> scene key
t:cm"/o ";ol:(`$t[;0])!lk`$t[;1]            / object -> scene key ("b" is carried)
t:cm"/g ";ga:(`$t[;0])!{(`$x 1;`$x 2;*x 3)}'t / verb -> (subject;object;place)
hlp:dc"/h "                                 / help text

dsc:pth:(0#" ")!()                          / scene key -> description, paths
{k:x[1]1;a:ept'1_'p@2_x;s:" "\'a 1;dsc::dsc,(,k)!,a 0;pth::pth,(,k)!,s}'gd;

/ Describe Scene
dp:{"There is a ",x[1]," going ",x[0]," from here to the ",x[2]}
look:{`0:dsc l;`0:dp'pth l;$[#o:(!ol)@&l=.ol;`0:{"You see a ",x," on the floor"}'$o;]}

/ Verbs
mv:{t:pth l;i:t[;0]?x;$[^i;`0:"You cannot go that way";[l::lk`$t[i;2];look 0]]}
pk:{$[l~ol x;[ol::@[ol;x;:;"b"];`0:"You are now carrying the ",$x];`0:"You cannot get that"]}
inv:{$[#o:(!ol)@&"b"=.ol;`0:"You are carrying: ",", "/$o;`0:"You are not carrying anything"]}
hv:{"b"~ol x}

/ Actions - conditions live in the /g comments above; outcomes here
win:("The wizard awakens from his slumber and greets you warmly -";"he hands you the magic low-carb donut - you win! the end")
lose:("The wizard awakens and sees that you stole his frog -";"he is so upset he banishes you to the netherworlds - you lose! the end")
wd:{$[hv`bucket;[wl::1;"The chain is now securely welded to the bucket"];"You do not have a bucket"]}
dk:{$[wl;[fl::1;"The bucket is now full of water"];"The water level is too low to reach"]}
sp:{$[~fl;"The bucket has nothing in it";hv`frog;[fin::1;lose];[fin::1;win]]}
gh:(`weld`dunk`splash)!(wd;dk;sp)
act:{[v;s;o](S;O;P):ga v;$[(l~P)&(s~S)&(o~O)&hv S;`0:gh[v]0;`0:"I cannot ",($v)," like that"]}

/ Command Dispatch
hp:{`0:hlp,,""}
vb:(`look`walk`go`pickup`get`inventory`i`help`quit`q)!({look 0};{mv x 1};{mv x 1};{pk`$x 1};{pk`$x 1};{inv 0};{inv 0};{hp 0};{fin::1};{fin::1})
do:{x:3#x,3#,"";c:`$*x;$[~^(!vb)?c;vb[c]x;~^(!ga)?c;act[c;`$x 1;`$x 2];`0:"I do not understand"]}

intro:{`0:"Wizard adventure\n";hp 0}

/ Game Loop
run:{intro 0;look 0;{~fin}{x;`1:"\n> ";i:1:`;$[#i;{$[fin;;#w:ept@" "\x;do w;]}'"\n"\-1_i;fin::1];0}/0}
run 0;

/
--------------------------------------------------------------------------------
\0
================================================================================

Living Room:

You are in the living room of a wizards house

There is a wizard snoring loudly on the couch

================================================================================

west door garden
upstairs stairway attic

--------------------------------------------------------------------------------
\1
================================================================================
Garden:

You are in a beautiful garden

There is a well in front of you

================================================================================

east door living-room

--------------------------------------------------------------------------------
\2
================================================================================
Attic:

You are in the attic of the wizards house

There is a giant welding torch in the corner

================================================================================

downstairs stairway living-room

\
