#pragma once
#include<cstdint>

#ifdef __cplusplus
extern "C"{
#endif


typedef struct {
    bool owned;
    size_t len;
    size_t cap;
    uint8_t* ptr;
} FfiSlice;

typedef struct {
    FfiSlice name;
    FfiSlice value;
} HeaderPair;

typedef struct {
    bool owned;
    bool valid;

    bool headComplete;
    bool bodyComplete;

    FfiSlice path;
    uint8_t method;
    uint8_t version;
    FfiSlice methodStr;

    size_t headersLen;
    size_t headersCap;
    HeaderPair* headers;
    FfiSlice body;

    FfiSlice host;
    FfiSlice scheme;
} HttpClient;

typedef void* FfiFuture;
typedef void* FfiBundle;
typedef void* FfiServer;
typedef void* FfiSocket;

bool init_rt();
bool has_init();

FfiFuture ffi_future_new(void (*cb)(void* userdata, void* result), void* userdata);
uint8_t ffi_future_state(FfiFuture fut);
void* ffi_future_result(FfiFuture fut);
void* ffi_future_take_result(FfiFuture fut);
void ffi_future_cancel(FfiFuture fut);
void ffi_future_complete(FfiFuture fut, void* result);
void ffi_future_free(FfiFuture fut);
void ffi_future_await(FfiFuture fut);
int ffi_future_get_errno(FfiFuture fut);
FfiSlice* ffi_future_get_errmsg(FfiFuture fut);

void free_slice(FfiSlice slice);

long long add_i64(long long x, long long y);

void server_new_tcp(FfiFuture fut, char* addr_cstr); // resolves in FfiServer
void server_accept(FfiFuture fut, FfiServer server); // resolves in FfiBundle
void server_loop(FfiFuture fut, FfiServer server, void (*cb)(FfiBundle));

bool addr_is_ipv4(FfiBundle bundle);
bool addr_is_ipv6(FfiBundle bundle);
FfiSlice get_addr_str(FfiBundle bundle);  // manual free

void tcp_detect_prot(FfiFuture fut, FfiBundle bundle);
FfiSocket http1_new(FfiBundle bundle, size_t bufsize);

void http_read_client(FfiFuture fut, FfiSocket http);
void http_read_until_complete(FfiFuture fut, FfiSocket http);
void http_read_until_head_complete(FfiFuture fut, FfiSocket http);

void http_set_header(FfiSocket http, HeaderPair pair);
void http_add_header(FfiSocket http, HeaderPair pair);
void http_del_header(FfiSocket http, HeaderPair pair);

void http_write(FfiFuture fut, FfiSocket http, FfiSlice bytes);
void http_close(FfiFuture fut, FfiSocket http, FfiSlice bytes);
void http_flush(FfiFuture fut, FfiSocket http);

HttpClient http_get_fficlient(FfiSocket http);
void http_free_fficlient(HttpClient client);

bool http_client_has_header(FfiSocket http, FfiSlice name);
size_t http_client_has_header_count(FfiSocket http, FfiSlice name);
FfiSlice http_client_get_first_header(FfiSocket http, FfiSlice name);
FfiSlice http_client_get_header(FfiSocket http, FfiSlice name, size_t index);

void http_free(FfiSocket http);

void http1_direct_write(FfiFuture fut, FfiSocket http, FfiSlice bytes);


#ifdef __cplusplus
}
#endif