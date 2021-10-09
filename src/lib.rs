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

pub fn atom(atom: &[u8]) -> Sexp {
    Sexp::Atom(atom.to_vec())
}

pub fn list(list: &[Sexp]) -> Sexp {
    Sexp::List(list.to_vec())
}

pub trait SexpOf {
    fn sexp_of(&self) -> Sexp;
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

impl<T: ToString + UseToString> SexpOf for T {
    fn sexp_of(&self) -> Sexp {
        Sexp::Atom(self.to_string().as_bytes().to_vec())
    }
}

impl SexpOf for &str {
    fn sexp_of(&self) -> Sexp {
        Sexp::Atom(self.as_bytes().to_vec())
    }
}

impl<'a> SexpOf for BytesSlice<'a> {
    fn sexp_of(&self) -> Sexp {
        Sexp::Atom(self.0.to_vec())
    }
}

impl<T> SexpOf for [T]
where
    T: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        Sexp::List(self.iter().map(|x| x.sexp_of()).collect())
    }
}

impl<T1, T2> SexpOf for (T1, T2)
where
    T1: SexpOf,
    T2: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        Sexp::List(vec![self.0.sexp_of(), self.1.sexp_of()])
    }
}

impl<T1, T2, T3> SexpOf for (T1, T2, T3)
where
    T1: SexpOf,
    T2: SexpOf,
    T3: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        Sexp::List(vec![self.0.sexp_of(), self.1.sexp_of(), self.2.sexp_of()])
    }
}

impl<K, V> SexpOf for std::collections::HashMap<K, V>
where
    K: SexpOf,
    V: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        Sexp::List(
            self.iter()
                .map(|(k, v)| list(&[k.sexp_of(), v.sexp_of()]))
                .collect(),
        )
    }
}

impl<K, V> SexpOf for std::collections::BTreeMap<K, V>
where
    K: SexpOf,
    V: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        Sexp::List(
            self.iter()
                .map(|(k, v)| list(&[k.sexp_of(), v.sexp_of()]))
                .collect(),
        )
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
