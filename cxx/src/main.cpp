#include<stdio.h>
#include "ffi.h"

int add(int x, int y){
    return x + y;
}

int main(){
    printf("herro");
    auto test = add_f64(1.0, 2.0);
    printf("result is %f", test);
}