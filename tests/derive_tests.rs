use rsexp::{OfSexp, SexpOf};
use rsexp_derive::{OfSexp, SexpOf};

fn test_bytes<T: SexpOf>(t: T, bytes: &str) {
    let b = t.sexp_of().to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), bytes)
}

fn test_rt<T: SexpOf + OfSexp + std::fmt::Debug + Eq>(t: T, bytes: &str) {
    let sexp = t.sexp_of();
    let b = sexp.to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), bytes);
    let t2: T = OfSexp::of_sexp(&sexp).unwrap();
    assert_eq!(t, t2)
}

fn test_rt_no_eq<T: SexpOf + OfSexp + std::fmt::Debug>(t: T, bytes: &str) {
    let sexp = t.sexp_of();
    let b = sexp.to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), bytes);
    let t2: T = OfSexp::of_sexp(&sexp).unwrap();
    let b = t2.sexp_of().to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), bytes);
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
struct Pancakes(i64);

#[test]
fn breakfast1() {
    test_rt(Pancakes(12), "(12)");
    test_rt(Pancakes(12345678910111213), "(12345678910111213)");
    test_rt(Pancakes(-12345678910111213), "(-12345678910111213)");
}

#[derive(OfSexp, SexpOf, Debug, PartialEq)]
struct MorePancakes(i64, f64, Option<i64>);

#[test]
fn breakfast2() {
    test_rt_no_eq(
        MorePancakes(12, 3.141592, Some(1234567890123)),
        "(12 3.141592 (1234567890123))",
    );
    test_rt_no_eq(MorePancakes(12, std::f64::NAN, None), "(12 NaN ())");
    test_rt_no_eq(
        MorePancakes(12, std::f64::NEG_INFINITY, None),
        "(12 -inf ())",
    );
}

#[derive(OfSexp, SexpOf, Debug, PartialEq)]
struct Breakfasts {
    pancakes: Pancakes,
    more_pancakes: Option<MorePancakes>,
    value1: i32,
    value2: (f64, f64),
}

#[test]
fn breakfast3() {
    test_rt_no_eq(
        Breakfasts {
            pancakes: Pancakes(12345),
            more_pancakes: Some(MorePancakes(12, 3.141592, Some(1234567890123))),
            value1: 987654321,
            value2: (3.14159265358979, 2.71828182846),
        },
        "((pancakes (12345)) (more_pancakes ((12 3.141592 (1234567890123)))) (value1 987654321) (value2 (3.14159265358979 2.71828182846)))",
    );
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
struct BreakfastsEq {
    pancakes: Pancakes,
    more_pancakes: Option<String>,
    value1: i32,
    value2: (i64, i64),
}

#[test]
fn breakfast4() {
    test_rt(
        BreakfastsEq {
            pancakes: Pancakes(12345),
            more_pancakes: Some("foo".to_string()),
            value1: 987654321,
            value2: (314159265358979, 271828182846),
        },
        "((pancakes (12345)) (more_pancakes (foo)) (value1 987654321) (value2 (314159265358979 271828182846)))",
    );
    test_rt(
        BreakfastsEq {
            pancakes: Pancakes(12345),
            more_pancakes: None,
            value1: 987654321,
            value2: (314159265358979, 271828182846),
        },
        "((pancakes (12345)) (more_pancakes ()) (value1 987654321) (value2 (314159265358979 271828182846)))",
    );
}

// From the OCaml implementation
// type truc = int * int
// type t =
//     A
//   | B of unit
//   | C of int
//   | D of int * int
//   | E of truc
//   | F of { x : int; y : float; }
// utop # F {x=1; y=3.14} |> sexp_of_t |> Sexp.to_string;;
// - : string = "(F(x 1)(y 3.14))"
// utop # E (1, 2) |> sexp_of_t |> Sexp.to_string;;
// - : string = "(E(1 2))"
// utop # D (1, 2) |> sexp_of_t |> Sexp.to_string;;
// - : string = "(D 1 2)"
// utop # A |> sexp_of_t |> Sexp.to_string;;
// - : string = "A"
// utop # B () |> sexp_of_t |> Sexp.to_string;;
// - : string = "(B())"
// utop # C 42 |> sexp_of_t |> Sexp.to_string;;
// - : string = "(C 42)"

#[derive(SexpOf, Debug, PartialEq)]
struct PairInt(i64, i64);

#[derive(SexpOf, Debug, PartialEq)]
struct StructXY {
    x: i64,
    y: f32,
}

#[derive(SexpOf, Debug, PartialEq)]
enum MyEnum {
    A(),
    AEmptyStruct {},
    B(()),
    C(i64),
    D(i64, i64),
    E(PairInt),
    F { x: i64, y: f32 },
    G(StructXY),
    H(&'static str, &'static str),
}

#[test]
fn my_enum() {
    test_bytes(MyEnum::A(), "A");
    test_bytes(MyEnum::AEmptyStruct {}, "AEmptyStruct");
    test_bytes(MyEnum::B(()), "(B ())");
    test_bytes(MyEnum::C(42), "(C 42)");
    test_bytes(MyEnum::D(42, 1337), "(D 42 1337)");
    test_bytes(MyEnum::E(PairInt(42, 1337)), "(E (42 1337))");
    test_bytes(MyEnum::F { x: -1, y: 3.14 }, "(F (x -1) (y 3.14))");
    test_bytes(
        MyEnum::G(StructXY { x: -1, y: 3.14 }),
        "(G ((x -1) (y 3.14)))",
    );
    test_bytes(
        MyEnum::H("foo", " needs escaping\n"),
        "(H foo \" needs escaping\\n\")",
    );
}
