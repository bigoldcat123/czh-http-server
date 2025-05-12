use http::{HeaderValue, Response};

#[test]
fn test_function() {
    struct A {}
    impl Into<String> for A {
        fn into(self) -> String {
            "as".to_string()
        }
    }

    struct B {}
    impl Into<String> for B {
        fn into(self) -> String {
            "asd".to_string()
        }
    }

    fn do_something<T: Into<String>>(p: impl Fn() -> T) {
        let e = p();
        println!("e{:?}", e.into());
    }

    do_something(|| A {});
    do_something(|| B {});
}
