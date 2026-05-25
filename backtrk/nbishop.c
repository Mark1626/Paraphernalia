#include <stdint.h>
#include<stdio.h>

int64_t cnt;
int64_t tot;

void recur(int ld,int col,int rd)
{
    //printf("%d %d %d\n",ld,col,rd);
    int pos,bit;
    if(col == tot)
        ++cnt;
    pos = (~(ld|col)) & tot;
    //printf("%d\n",pos);
    while(pos){
        bit = pos & -pos;
        pos -= bit;
        recur((ld|bit)<<1,col|bit,(rd|bit)>>1);
    }
}

int main()
{
    int i;
    for (i = 0; i < 12; ++i) {
        cnt = 0;
        tot = (2<<i)-1;
        recur(0,0,0);
        printf("%d %lld\n",i, cnt);
    }
    return 0;
}
