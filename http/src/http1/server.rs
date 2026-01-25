use crate::shared::{HttpType, Stream, server::HttpSocket};

pub struct Http1Socket<S: Stream>{
    pub net: S,
}

impl<S:Stream> Http1Socket<S>{
    pub fn new(net: S) -> Self{
        Http1Socket { 
            net,
        }
    }
}

impl<S:Stream> HttpSocket for Http1Socket<S>{
    fn get_type() -> HttpType {
        HttpType::Http1
    }
}