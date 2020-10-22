#include<stdio.h>

int main () {
    char* line = (char*) malloc(256);
    scanf("%[^\n]",line);
    printf("%s\n", line);
    free(line);
    return 0;
}