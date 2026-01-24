

#[allow(dead_code)]
#[link(name = "httpcpp")]
unsafe extern "C" {
    pub unsafe fn mainthing() -> i32;

    pub unsafe fn add(a: i32, b: i32) -> i32;
    pub unsafe fn add_f64(a: f64, b: f64) -> f64;
}

