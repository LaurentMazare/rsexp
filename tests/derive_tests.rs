#![allow(clippy::approx_constant)]
use rsexp::{IntoSexpError, OfSexp, SexpOf};
use rsexp_derive::{OfSexp, SexpOf};
use std::collections::BTreeMap;

fn test_bytes<T: SexpOf>(t: T, str: &str) {
    let b = t.sexp_of().to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), str);
    let b = t.sexp_of().to_bytes_hum();
    assert_eq!(std::str::from_utf8(&b).unwrap(), str);
}

fn test_rt<T: SexpOf + OfSexp + std::fmt::Debug + Eq>(t: T, str: &str) {
    let sexp = t.sexp_of();
    let b = sexp.to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), str);
    let t2: T = sexp.of_sexp().unwrap();
    assert_eq!(t, t2);
    // Round trip via the to_bytes_hum representation.
    let t2: T = rsexp::from_slice(&sexp.to_bytes_hum()).unwrap().of_sexp().unwrap();
    assert_eq!(t, t2);
}

fn test_rt_no_eq<T: SexpOf + OfSexp + std::fmt::Debug>(t: T, str: &str) {
    let sexp = t.sexp_of();
    let b = sexp.to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), str);
    let t2: T = sexp.of_sexp().unwrap();
    let b = t2.sexp_of().to_bytes();
    assert_eq!(std::str::from_utf8(&b).unwrap(), str);
}

fn test_err<T: OfSexp>(str: &str, expected_err: IntoSexpError) {
    let sexp = rsexp::from_slice(str).unwrap();
    let t_or_err: Result<T, IntoSexpError> = sexp.of_sexp();
    let err = match t_or_err {
        Ok(_) => panic!("expected an error, got a value {}", str),
        Err(err) => err,
    };
    assert_eq!(err, expected_err)
}

fn length_mismatch(type_: &'static str, expected_len: usize, list_len: usize) -> IntoSexpError {
    IntoSexpError::ListLengthMismatch { type_, expected_len, list_len }
}

fn missing_fields(type_: &'static str, field: &'static str) -> IntoSexpError {
    IntoSexpError::MissingFieldsInStruct { type_, field }
}

fn extra_fields(type_: &'static str, extra_fields: &[&str]) -> IntoSexpError {
    IntoSexpError::ExtraFieldsInStruct {
        type_,
        extra_fields: extra_fields.iter().map(|x| x.to_string()).collect(),
    }
}

fn expected_atom_got_list(type_: &'static str, list_len: usize) -> IntoSexpError {
    IntoSexpError::ExpectedAtomGotList { type_, list_len }
}

fn expected_list_got_atom(type_: &'static str) -> IntoSexpError {
    IntoSexpError::ExpectedListGotAtom { type_ }
}

fn unknown_constructor(type_: &'static str, constructor: &str) -> IntoSexpError {
    IntoSexpError::UnknownConstructorForEnum { type_, constructor: constructor.to_string() }
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
struct Pancakes(i64);

#[test]
fn breakfast1() {
    test_rt(Pancakes(12), "(12)");
    test_rt(Pancakes(12345678910111213), "(12345678910111213)");
    test_rt(Pancakes(-12345678910111213), "(-12345678910111213)");
    test_err::<Pancakes>("()", length_mismatch("Pancakes", 1, 0));
    test_err::<Pancakes>("(1 2)", length_mismatch("Pancakes", 1, 2));
    test_err::<Pancakes>("(1 2 3 4)", length_mismatch("Pancakes", 1, 4));
    test_err::<Pancakes>("(())", expected_atom_got_list("stringable", 0));
    test_err::<Pancakes>("((1))", expected_atom_got_list("stringable", 1));
    test_err::<Pancakes>("((1 2))", expected_atom_got_list("stringable", 2));
    test_err::<Pancakes>(
        "(a)",
        IntoSexpError::StringConversionError { err: "invalid digit found in string".to_string() },
    );
}

#[derive(OfSexp, SexpOf, Debug, PartialEq)]
struct MorePancakes(i64, f64, Option<i64>);

#[test]
fn breakfast2() {
    test_rt_no_eq(MorePancakes(12, 3.141592, Some(1234567890123)), "(12 3.141592 (1234567890123))");
    test_rt_no_eq(MorePancakes(12, std::f64::NAN, None), "(12 NaN ())");
    test_rt_no_eq(MorePancakes(12, std::f64::NEG_INFINITY, None), "(12 -inf ())");
    test_err::<MorePancakes>("()", length_mismatch("MorePancakes", 3, 0));
    test_err::<MorePancakes>("(1 2 3)", expected_list_got_atom("option"));
    test_err::<MorePancakes>("(1 2 (3 4))", length_mismatch("option", 1, 2));
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
    test_err::<Breakfasts>("()", missing_fields("Breakfasts", "pancakes"));
    test_err::<Breakfasts>("((pancakes (1)))", missing_fields("Breakfasts", "more_pancakes"));
    test_err::<Breakfasts>(
        "((pancakes (1))(more_pancakes ())(value1 1)(value3 (1 2)))",
        missing_fields("Breakfasts", "value2"),
    );
    test_err::<Breakfasts>(
        "((pancakes (1))(more_pancakes ())(value1 1)(value2 (1 2))(extra foo))",
        extra_fields("Breakfasts", &["extra"]),
    );
    test_err::<Breakfasts>(
        "((pancakes (1))(more_pancakes ())(value1 1)(value2 (1 2))(a foo)(b bar))",
        extra_fields("Breakfasts", &["a", "b"]),
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

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
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
    test_bytes(MyEnum::G(StructXY { x: -1, y: 3.14 }), "(G ((x -1) (y 3.14)))");
    test_bytes(MyEnum::H("foo", " needs escaping\n"), "(H foo \" needs escaping\\n\")");
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
struct StructXYZ {
    x: i64,
    y: Option<(i32, i32)>,
    z: String,
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
enum MyEnum2 {
    A(),
    AEmptyStruct {},
    B(()),
    C(i64),
    D(i64, i64),
    E(PairInt),
    F { x: i64, y: String },
    G(StructXYZ),
}

#[test]
fn my_enum2() {
    test_rt(MyEnum2::A(), "A");
    test_rt(MyEnum2::AEmptyStruct {}, "AEmptyStruct");
    test_rt(MyEnum2::B(()), "(B ())");
    test_rt(MyEnum2::C(42), "(C 42)");
    test_rt(MyEnum2::D(42, 1337), "(D 42 1337)");
    test_rt(MyEnum2::E(PairInt(42, 1337)), "(E (42 1337))");
    test_rt(MyEnum2::F { x: -1, y: "foo bar\x7F".to_string() }, "(F (x -1) (y \"foo bar\\127\"))");
    test_rt(
        MyEnum2::G(StructXYZ { x: -1, y: Some((12345, 678910)), z: "test\"".to_string() }),
        "(G ((x -1) (y ((12345 678910))) (z \"test\\\"\")))",
    );
    test_bytes(
        MyEnum2::G(StructXYZ { x: -1, y: None, z: "".to_string() }),
        "(G ((x -1) (y ()) (z \"\")))",
    );
    test_err::<MyEnum2>("()", IntoSexpError::ExpectedConstructorGotEmptyList { type_: "MyEnum2" });
    test_err::<MyEnum2>("Z", unknown_constructor("MyEnum2", "Z"));
    test_err::<MyEnum2>("1", unknown_constructor("MyEnum2", "1"));
    test_err::<MyEnum2>("(Z foo)", unknown_constructor("MyEnum2", "Z"));
}

#[derive(OfSexp, SexpOf, Debug, PartialEq, Eq)]
struct WithVec {
    x: Vec<(String, i32)>,
    y: Option<(i32, i32)>,
    z: Vec<String>,
    m: BTreeMap<String, (i32, i32)>,
}

#[test]
fn with_vec() {
    let mut m = BTreeMap::new();
    let wv = WithVec {
        x: vec![("foo".to_string(), 1337), (" bar".to_string(), 42)],
        y: Some((98765, -4321)),
        z: vec![],
        m: m.clone(),
    };
    test_rt(wv, "((x ((foo 1337) (\" bar\" 42))) (y ((98765 -4321))) (z ()) (m ()))");
    m.insert("foo".to_string(), (1, 2));
    m.insert("bar".to_string(), (12, 23));
    m.insert("foo bar".to_string(), (123, 234));
    let wv = WithVec {
        x: vec![("\0".to_string(), 1337), ("xyz123".to_string(), 42)],
        y: None,
        z: vec!["a".to_string(), "bcd".to_string()],
        m,
    };
    test_rt(
        wv,
        "((x ((\"\\000\" 1337) (xyz123 42))) (y ()) (z (a bcd)) (m ((bar (12 23)) (foo (1 2)) (\"foo bar\" (123 234)))))"
    );
}
