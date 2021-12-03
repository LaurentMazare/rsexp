extern crate quickcheck;
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use rsexp::{from_slice, Sexp};

use quickcheck::Arbitrary;

fn arbitrary_(g: &mut quickcheck::Gen, max_depth: u8) -> Sexp {
    if max_depth == 0 || bool::arbitrary(g) {
        let data = if bool::arbitrary(g) {
            (0..=(u32::arbitrary(g) % 10)).map(|_| 97 + u8::arbitrary(g) % 26).collect()
        } else {
            Vec::<u8>::arbitrary(g)
        };
        Sexp::Atom(data)
    } else {
        let len = usize::arbitrary(g) % 10;
        let list: Vec<_> = (0..len).map(|_| arbitrary_(g, max_depth - 1)).collect();
        Sexp::List(list)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct QSexp(Sexp);

impl quickcheck::Arbitrary for QSexp {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        QSexp(arbitrary_(g, 4))
    }
}

fn rt<T: AsRef<[u8]> + ?Sized>(s: &T) -> String {
    let s = s.as_ref();
    let sexp = from_slice(s).unwrap();
    let bytes = sexp.to_bytes();
    assert_eq!(from_slice(&bytes).unwrap(), sexp);
    assert_eq!(from_slice(&sexp.to_bytes_hum()).unwrap(), sexp);
    String::from_utf8_lossy(&bytes).to_string()
}

#[quickcheck]
fn round_trip(sexp: QSexp) -> bool {
    let sexp = sexp.0;
    from_slice(&sexp.to_bytes()).unwrap() == sexp
        && from_slice(&sexp.to_bytes_hum()).unwrap() == sexp
}

#[test]
fn roundtrip_sexp() {
    assert_eq!(rt("(    ATOM)"), "(ATOM)");
    assert_eq!(rt(" ( \"foo bar\"   baz \"x\\\"\") "), "(\"foo bar\" baz \"x\\\"\")");
    assert_eq!(rt("\t()"), "()");
    assert_eq!(rt("(()()(()()(())))"), "(() () (() () (())))");
    assert_eq!(rt("((foo bar)()(()()((\"\n\"))))"), "((foo bar) () (() () ((\"\\n\"))))");
}

fn rt_mach(s: &str) {
    let sexp = from_slice(s).unwrap();
    let bytes = sexp.to_bytes_mach();
    assert_eq!(from_slice(&bytes).unwrap(), sexp);
    let round_tripped = String::from_utf8_lossy(&bytes).to_string();
    assert_eq!(&round_tripped, s)
}

fn rt_hum(s: &str) {
    let sexp = from_slice(s.as_bytes()).unwrap();
    let bytes = sexp.to_bytes_hum();
    assert_eq!(from_slice(&bytes).unwrap(), sexp);
    let round_tripped = String::from_utf8_lossy(&bytes).to_string();
    assert_eq!(&round_tripped, s)
}

#[test]
fn roundtrip_sexp_mach() {
    rt_mach("(ATOM)");
    rt_mach("(A T O M)");
    rt_mach("(\"foo bar\"baz\"x\\\"\")");
    rt_mach("()");
    rt_mach("(((())))");
    rt_mach("(()()(()()(())))");
    rt_mach("((foo bar)()(()()((\"\\n\"))))");
    rt_mach("((foo\"bar\\\\\")()(()()((\"\\\\n\"))))");
    rt_mach("((g)(\" \"a\" \"b c)(e()d()(()a)b))");
}

#[test]
fn roundtrip_sexp_hum() {
    rt_hum("(ATOM)");
    rt_hum("(A T O M)");
    rt_hum("(\"foo bar\" baz \"x\\\"\")");
    rt_hum("()");
    rt_hum("(((())))");
    rt_hum("(() () (() () (())))");
    rt_hum("((foo bar) () (() () ((\"\\n\"))))");
    rt_hum("((foo \"bar\\\\\") () (() () ((\"\\\\n\"))))");
    rt_hum("((g) (\" \" a \" \" b c) (e () d () (() a) b))");
    rt_hum(
        r#"(a b c d
 (b (c)
  ((hetuaonhunoeatuhaoenuthenuthaonuthoaeunthaounaoethunateohunaotheuanuhaeo
    auhteonuheocuhaoecur eoathuanetouh otn hu aeontuhaoentuhaoenuh d d d d d d d d d d d d)
   (abcd) (ohuntahuaoehnuatheuatoeu)
   (heuntahuncaoehtuanotuehaoentuhoaentuhaoentuhaoenuthaounaheo)))
 (a beuhtaeuntaohutnaouhaonuhaonuthaounao hteounha))"#,
    );
}
