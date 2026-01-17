#[cfg(test)]


#[test]
fn one_is_one(){
    assert!(1 == 1);
}

#[test]
fn httpcpp_test(){
    use crate::httpcpp::add;
    use crate::httpcpp::add_f64;
    use crate::httpcpp::mainthing;
    
    unsafe{
        assert!(add_f64(1.0, 2.0) == 3.0);
        assert!(add(1, 2) == 3);
        assert!(mainthing() == 0);
    }
}