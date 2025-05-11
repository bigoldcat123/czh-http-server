#[test]
fn test_function() {
    trait A {
        fn hello(self);
    }
    struct C {}
    impl A for C {
        fn hello(self) {}
    }
    struct D {}
    impl A for D {
        fn hello(self) {}
    }
    fn a(aa: Box<dyn A>) {}

    let ax = C {};
    a(Box::new(ax));

    fn aa(a: &[impl Fn() -> Box<dyn Future<Output = String>>]) {}

    // aa(&[
    //     || Box::new(async { String::new() }),
    //     || Box::new(async { String::new() }),
    // ]);
    let a = Box::new(async { String::new() });
    let b = Box::new(async { String::new() });
    // println!("{:?}", a == b);
    let v: Vec<Box<dyn Future<Output = String>>> = vec![a, b];

    // aa(v.as_slice());
    // for e in v {
    //     e()
    // }
}
