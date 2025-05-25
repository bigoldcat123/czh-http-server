use std::{
    fs::File,
    path::{self, Path},
};

use http::Uri;
use http_server_async_czh::body_type::JsonBody;
use serde::{Deserialize, Serialize};

#[test]
fn test_function() {
    let a = Path::new("s");
    #[derive(Serialize, Deserialize)]
    struct A {
        name: String,
    }
    let a = A{
        name:String::from("easd")
    };
    let s = serde_json::to_string(&a).unwrap();
    println!("s{:?}",s);
    let j = JsonBody::new(s);
    let a:A = j.data();
    println!("{:?}",a.name);
    
}
