files:"\n"\`x(("/usr/bin/env";"ls");"")

filter:{x@&{".md"~-3#x}'x}

"out.txt" 1:,/{v:1:x; v,"\n"}'-1_filter@files
