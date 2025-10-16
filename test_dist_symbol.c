#include <stdio.h>

int ZopfliGetDistSymbol(int dist)
{
    if (dist < 5)
    {
        return dist - 1;
    }
    else
    {
        int l = (31 ^ __builtin_clz(dist - 1)); /* log2(dist - 1) */
        int r = ((dist - 1) >> (l - 1)) & 1;
        return l * 2 + r;
    }
}

int main() {
    printf("dist=1 -> %d\n", ZopfliGetDistSymbol(1));
    printf("dist=2 -> %d\n", ZopfliGetDistSymbol(2));
    printf("dist=3 -> %d\n", ZopfliGetDistSymbol(3));
    printf("dist=4 -> %d\n", ZopfliGetDistSymbol(4));
    printf("dist=5 -> %d\n", ZopfliGetDistSymbol(5));
    printf("dist=6 -> %d\n", ZopfliGetDistSymbol(6));
    printf("dist=7 -> %d\n", ZopfliGetDistSymbol(7));
    printf("dist=8 -> %d\n", ZopfliGetDistSymbol(8));
    printf("dist=9 -> %d\n", ZopfliGetDistSymbol(9));
    printf("dist=10 -> %d\n", ZopfliGetDistSymbol(10));
    
    printf("\nDebug for dist=5:\n");
    int dist = 5;
    int l = (31 ^ __builtin_clz(dist - 1));
    int r = ((dist - 1) >> (l - 1)) & 1;
    printf("dist-1=%d, __builtin_clz(dist-1)=%d, l=%d, r=%d, result=%d\n", 
           dist-1, __builtin_clz(dist-1), l, r, l * 2 + r);
    
    return 0;
}

