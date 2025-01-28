#include <stdio.h>
#include <stdlib.h>
#define I int
#define R return
#define elif else if
#define D div_t
#define W 6
#define H 5
#define MPL 26
#define NP 99
#define PF printf
#define PLP(p,s) for(I i=0;i<=s;i++)PF("%2d%c",p[i]+1," \n"[i==s]);
I g[]={
   3, +0, -2, -1, -1, -1,
  -2, +1, +0, -1, -2, +0,
  +0, +3, +2, -2, -1, +2,
  +2, +0, -2, -2, -1, +1,
  +1, +2, -2, +0, -1,  0
};
I mv[]={+0,-1,+1,-1,+1,+0,+1,+1,+0,+1,-1,+1,-1,+0,-1,-1};
I slv(I*p,I s,I pd,I bn){if(s>=MPL)R NP;if(p[s]==W*H-1){PF("Path len: %d; ",s);PLP(p,s);R s;}elif(s<=bn){if(p[s]==0)pd=g[0];D d=div(p[s],W);I x=d.rem,y=d.quot;for(I a=0;a<8;a++){I xx=x+pd*mv[2*a],yy=y+pd*mv[2*a+1];if(xx>=0&&xx<W&&yy>=0&&yy<H){I i=yy*W+xx,nd=pd+g[i];p[s+1]=i;if(nd>0&&!(xx==x&&yy==y)){I r=slv(p,s+1,nd,bn);bn=r<bn?r:bn;}}}}R bn;}
I main(){I p[MPL+1]={0};slv(p,0,g[0],MPL+1);}
