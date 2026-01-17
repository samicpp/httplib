#include<stdio.h>
#include "ffi.h"

int add(int x, int y){
    return x + y;
}

int mainthing(){
    printf("herro");
    auto test = 1.0 + 2.0;
    printf("result is %f", test);
    return 0;
}

double add_f64(double x, double y){
    return x + y;
}