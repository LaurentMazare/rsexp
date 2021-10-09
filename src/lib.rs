#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod of_sexp;
mod parse;

pub use of_sexp::*;
pub use parse::*;
use std::io::Write;

/// Type for S-expressions using owned values.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Sexp {
    Atom(Vec<u8>),
    List(Vec<Sexp>),
}

// This trait is used to mark types for which using the to/from string
// conversion is fine.
pub trait UseToString {}

pub struct BytesSlice<'a>(pub &'a [u8]);

// Conversion from T to sexp.

impl UseToString for u64 {}
impl UseToString for u32 {}
impl UseToString for u16 {}
impl UseToString for u8 {}
impl UseToString for i64 {}
impl UseToString for i32 {}
impl UseToString for i16 {}
impl UseToString for i8 {}
impl UseToString for usize {}
impl UseToString for f64 {}
impl UseToString for f32 {}
impl UseToString for bool {}

impl<T: ToString + UseToString> From<T> for Sexp {
    fn from(t: T) -> Self {
        Sexp::Atom(t.to_string().as_bytes().to_vec())
    }
}

impl From<&str> for Sexp {
    fn from(s: &str) -> Self {
        Sexp::Atom(s.as_bytes().to_vec())
    }
}

impl<'a> From<&BytesSlice<'a>> for Sexp {
    fn from(s: &BytesSlice<'a>) -> Self {
        Sexp::Atom(s.0.to_vec())
    }
}

impl From<String> for Sexp {
    fn from(s: String) -> Self {
        Sexp::Atom(s.as_bytes().to_vec())
    }
}

impl<'a, T: 'a> From<&'a [T]> for Sexp
where
    &'a T: Into<Sexp>,
{
    fn from(t: &'a [T]) -> Self {
        Sexp::List(t.iter().map(|x| x.into()).collect())
    }
}

impl<T1, T2> From<(T1, T2)> for Sexp
where
    T1: Into<Sexp>,
    T2: Into<Sexp>,
{
    fn from(t: (T1, T2)) -> Self {
        Sexp::List(vec![t.0.into(), t.1.into()])
    }
}

impl<T1, T2, T3> From<(T1, T2, T3)> for Sexp
where
    T1: Into<Sexp>,
    T2: Into<Sexp>,
    T3: Into<Sexp>,
{
    fn from(t: (T1, T2, T3)) -> Self {
        Sexp::List(vec![t.0.into(), t.1.into(), t.2.into()])
    }
}

impl<'a, K, V> From<&'a std::collections::HashMap<K, V>> for Sexp
where
    &'a K: Into<Sexp>,
    &'a V: Into<Sexp>,
{
    fn from(map: &'a std::collections::HashMap<K, V>) -> Self {
        Sexp::List(map.iter().map(|(k, v)| (k, v).into()).collect())
    }
}

impl<'a, K, V> From<&'a std::collections::BTreeMap<K, V>> for Sexp
where
    &'a K: Into<Sexp>,
    &'a V: Into<Sexp>,
{
    fn from(map: &'a std::collections::BTreeMap<K, V>) -> Self {
        Sexp::List(map.iter().map(|(k, v)| (k, v).into()).collect())
    }
}

// Serialization

fn must_escape(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    for (index, &c) in data.iter().enumerate() {
        match c {
            0..=32 | 127..=255 | b'"' | b'(' | b')' | b';' | b'\\' => return true,
            b'|' if index > 0 && data[index - 1] == b'#' => return true,
            b'#' if index > 0 && data[index - 1] == b'|' => return true,
            _ => {}
        }
    }
    false
}

fn write_u8<W: Write>(b: u8, w: &mut W) -> std::io::Result<()> {
    w.write_all(&[b])
}

fn write_escaped<W: Write>(data: &[u8], w: &mut W) -> std::io::Result<()> {
    write_u8(b'"', w)?;
    for &c in data.iter() {
        match c {
            b'\\' | b'\"' => w.write_all(&[b'\\', c])?,
            b'\n' => w.write_all(b"\\n")?,
            b'\t' => w.write_all(b"\\t")?,
            b'\r' => w.write_all(b"\\r")?,
            8 => w.write_all(b"\\b")?,
            b' '..=b'~' => write_u8(c, w)?,
            _ => w.write_all(&[b'\\', 48 + c / 100, 48 + (c / 10) % 10, 48 + c % 10])?,
        }
    }
    write_u8(b'"', w)?;
    Ok(())
}

impl Sexp {
    /// Serialize a Sexp to a writer.
    pub fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        match self {
            Sexp::Atom(v) => {
                if must_escape(v) {
                    write_escaped(v, w)
                } else {
                    w.write_all(v)
                }
            }
            Sexp::List(vec) => {
                write_u8(b'(', w)?;
                for (index, elem) in vec.iter().enumerate() {
                    if index > 0 {
                        write_u8(b' ', w)?;
                    }
                    elem.write(w)?;
                }
                write_u8(b')', w)
            }
        }
    }

    /// Serialize a Sexp to a buffer.
    ///
    /// # Example
    ///
    /// ```
    ///     let sexp = rsexp::from_slice(b"((foo bar)(baz (1 2 3)))").unwrap();
    ///     assert_eq!(sexp.to_bytes(), b"((foo bar) (baz (1 2 3)))");
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // This could not fail as the buffer gets extended.
        self.write(&mut buffer).unwrap();
        buffer
    }
}

impl std::fmt::Display for Sexp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bytes = self.to_bytes();
        let cow = String::from_utf8_lossy(&bytes);
        write!(f, "{}", cow)
    }
}

#[cfg(test)]
mod tests {
    use crate::Sexp;
    use quickcheck::Arbitrary;

    fn arbitrary_(g: &mut quickcheck::Gen, max_depth: u8) -> Sexp {
        if max_depth == 0 || bool::arbitrary(g) {
            let data = if bool::arbitrary(g) {
                (0..=(u32::arbitrary(g) % 10))
                    .map(|_| 97 + u8::arbitrary(g) % 26)
                    .collect()
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

    impl quickcheck::Arbitrary for Sexp {
        fn arbitrary(g: &mut quickcheck::Gen) -> Sexp {
            arbitrary_(g, 4)
        }
    }

    fn rt(s: &[u8]) -> String {
        let sexp = crate::from_slice(s).unwrap();
        let bytes = sexp.to_bytes();
        assert_eq!(crate::from_slice(&bytes).unwrap(), sexp);
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[quickcheck]
    fn round_trip(sexp: crate::Sexp) -> bool {
        let bytes = sexp.to_bytes();
        crate::from_slice(&bytes) == Ok(sexp)
    }

    #[test]
    fn roundtrip_sexp() {
        assert_eq!(rt(b"(    ATOM)"), "(ATOM)");
        assert_eq!(
            rt(b" ( \"foo bar\"   baz \"x\\\"\") "),
            "(\"foo bar\" baz \"x\\\"\")"
        );
        assert_eq!(rt(b"\t()"), "()");
        assert_eq!(rt(b"(()()(()()(())))"), "(() () (() () (())))");
        assert_eq!(
            rt(b"((foo bar)()(()()((\"\n\"))))"),
            "((foo bar) () (() () ((\"\\n\"))))"
        );
    }
}
