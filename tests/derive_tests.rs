use rsexp::SexpOf;
use rsexp_derive::SexpOf;

fn test_bytes<T: SexpOf>(t: T, bytes: &str) {
    let b = t.sexp_of().to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), bytes)
}

#[derive(SexpOf, Debug, PartialEq, Eq)]
struct Pancakes(i64);

#[test]
fn breakfast1() {
    test_bytes(Pancakes(12), "(12)");
    test_bytes(Pancakes(12345678910111213), "(12345678910111213)");
    test_bytes(Pancakes(-12345678910111213), "(-12345678910111213)");
}

#[derive(SexpOf, Debug, PartialEq)]
struct MorePancakes(i64, f64, Option<i64>);

#[test]
fn breakfast2() {
    test_bytes(
        MorePancakes(12, 3.141592, Some(1234567890123)),
        "(12 3.141592 (1234567890123))",
    );
    test_bytes(MorePancakes(12, std::f64::NAN, None), "(12 NaN ())");
    test_bytes(
        MorePancakes(12, std::f64::NEG_INFINITY, None),
        "(12 -inf ())",
    );
}

#[derive(SexpOf, Debug, PartialEq)]
struct Breakfasts {
    pancakes: Pancakes,
    more_pancakes: Option<MorePancakes>,
    value1: i32,
    value2: (f64, f64),
}

#[test]
fn breakfast3() {
    test_bytes(
        Breakfasts {
            pancakes: Pancakes(12345),
            more_pancakes: Some(MorePancakes(12, 3.141592, Some(1234567890123))),
            value1: 987654321,
            value2: (3.14159265358979, 2.71828182846),
        },
        "((pancakes (12345)) (more_pancakes ((12 3.141592 (1234567890123)))) (value1 987654321) (value2 (3.14159265358979 2.71828182846)))",
    );
}
