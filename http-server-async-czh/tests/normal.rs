#[test]
fn test_function() {
    let mut  a = "hello".as_bytes().iter().as_slice().split(|x| *x == b'\n');
    let a = a.next();
    println!("a{:?}",a);

}
