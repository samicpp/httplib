#include<stdio.h>
#include "ffi.h"
#include "rsffi.h"
#include "wrapper.hpp"
#include <coroutine>


int add(int x, int y){
    return x + y;
}

int add_test(){
    // printf("herro\n");
    // double test = 1.0 + 2.0;
    // printf("result is %f\n", test);

    long long i64 = add_i64(1, 2);
    if (i64 != 3) return 1;

    return 0;
}

double add_f64(double x, double y){
    return x + y;
}

int server_test(){
    printf("hello?\n");
    char addr[] = "0.0.0.0:2048";

    printf("starting tokio rt\n");
    if (!has_init() && !init_rt()) return 1;

    FfiServer server;
    {
        bool done = false;
        printf("making future\n");
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        printf("passing future\n");
        server_new_tcp(fut, addr);
        printf("waiting for future\n");
        while (!done) ;
        // ffi_future_await(fut);
        server = ffi_future_take_result(fut);
    }
    printf("server ptr = %p\n", server);
    // if (!server) return 2;

    FfiBundle bundle;
    {
        bool done = false;
        printf("making future\n");
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        printf("passing future\n");
        server_accept(fut, server);
        printf("waiting for future\n");
        while (!done) ;
        // ffi_future_await(fut);
        bundle = ffi_future_take_result(fut);
    }
    printf("bundle ptr = %p\n", bundle);
    // if (!bundle) return 3;

    {
        bool done = false;
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        http_read_until_head_complete(fut, bundle);
        while (!done) ;
        // ffi_future_await(fut);
    }

    http_set_header(bundle, HeaderPair { sliceFromCstr("Content-Type"), sliceFromCstr("text/plain") });

    {
        bool done = false;
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        http_close(fut, bundle, sliceFromCstr("Hello, world!\n"));
        while (!done) ;
        // ffi_future_await(fut);
    }


    return 0;
}