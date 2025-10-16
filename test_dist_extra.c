#include <stdio.h>

int ZopfliGetDistExtraBits(int dist)
{
    if (dist < 5)
        return 0;
    return (31 ^ __builtin_clz(dist - 1)) - 1; /* log2(dist - 1) - 1 */
}

int ZopfliGetDistExtraBitsValue(int dist)
{
    if (dist < 5)
    {
        return 0;
    }
    else
    {
        int l = 31 ^ __builtin_clz(dist - 1); /* log2(dist - 1) */
        return (dist - (1 + (1 << l))) & ((1 << (l - 1)) - 1);
    }
}

int main() {
    printf("Extra bits:\n");
    for (int i = 1; i <= 20; i++) {
        printf("dist=%d -> extra_bits=%d, extra_value=%d\n", 
               i, ZopfliGetDistExtraBits(i), ZopfliGetDistExtraBitsValue(i));
    }
    return 0;
}

