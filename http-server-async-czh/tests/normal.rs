use http::{HeaderValue, Response};

#[test]
fn test_function() {
    let r = Response::builder()
        .header("asd", HeaderValue::from_static("hello"))
        .body("body")
        .unwrap();
    println!("{:?}", format!("{:?} {} {}", r.version(), r.status(), "ok"));
}
