mod of_sexp;
mod parse;
mod sexp_of;

pub use of_sexp::*;
pub use parse::*;
pub use sexp_of::*;
use std::io::Write;

const MAX_LINE_WIDTH: usize = 90;

/// Type for S-expressions using owned values.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

    /// Serialize multiple Sexps to a writer.
    pub fn write_multi<W: Write>(sexps: &[Self], w: &mut W) -> std::io::Result<()> {
        for (index, s) in sexps.iter().enumerate() {
            if index > 0 {
                write_u8(b' ', w)?
            }
            s.write(w)?
        }
        Ok(())
    }

    /// Serialize a Sexp to a writer in a machine readable way rather than
    /// human readable. This tries to avoid unnecessary whitespaces.
    pub fn write_mach<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        // The returned bool mentions whether a white space could be required.
        fn write_loop<W: Write>(
            s: &Sexp,
            need_whitespace: bool,
            w: &mut W,
        ) -> std::io::Result<bool> {
            match s {
                Sexp::Atom(v) => {
                    if must_escape(v) {
                        write_escaped(v, w)?;
                        Ok(false)
                    } else {
                        if need_whitespace {
                            write_u8(b' ', w)?;
                        }
                        w.write_all(v)?;
                        Ok(true)
                    }
                }
                Sexp::List(vec) => {
                    write_u8(b'(', w)?;
                    let mut need_whitespace = false;
                    for elem in vec.iter() {
                        need_whitespace = write_loop(elem, need_whitespace, w)?;
                    }
                    write_u8(b')', w)?;
                    Ok(false)
                }
            }
        }
        write_loop(self, false, w).map(|_| ())
    }

    /// Serialize a Sexp to a writer in a human readable way with some new lines
    /// and indentation.
    pub fn write_hum<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        enum EscapedSexpWithSize<'a> {
            AtomRef(&'a [u8]),
            AtomOwned(Vec<u8>),
            List {
                total_size: usize,
                values: Vec<EscapedSexpWithSize<'a>>,
            },
        }

        fn size(s: &EscapedSexpWithSize) -> usize {
            match s {
                EscapedSexpWithSize::AtomRef(atom) => atom.len(),
                EscapedSexpWithSize::AtomOwned(atom) => atom.len(),
                EscapedSexpWithSize::List { total_size, .. } => *total_size,
            }
        }

        fn escape(s: &Sexp) -> EscapedSexpWithSize {
            match s {
                Sexp::Atom(a) if must_escape(a) => {
                    let mut escaped = Vec::new();
                    write_escaped(a, &mut escaped).unwrap();
                    EscapedSexpWithSize::AtomOwned(escaped)
                }
                Sexp::Atom(a) => EscapedSexpWithSize::AtomRef(a),
                Sexp::List(l) => {
                    let mut total_size = 2 + l.len();
                    let mut values = Vec::new();
                    for elem in l.iter() {
                        let v = escape(elem);
                        total_size += size(&v);
                        values.push(v);
                    }
                    EscapedSexpWithSize::List { total_size, values }
                }
            }
        }

        fn write_loop<'a, W: Write>(
            s: &EscapedSexpWithSize<'a>,
            first_elem: bool,
            indent_level: usize,
            already_written_on_line: &mut usize,
            w: &mut W,
        ) -> std::io::Result<()> {
            if !first_elem && size(s) + *already_written_on_line > MAX_LINE_WIDTH {
                write_u8(b'\n', w)?;
                for _i in 0..indent_level {
                    write_u8(b' ', w)?;
                }
                *already_written_on_line = indent_level
            } else if !first_elem {
                *already_written_on_line += 1;
                write_u8(b' ', w)?;
            }
            match s {
                EscapedSexpWithSize::AtomRef(a) => {
                    *already_written_on_line += a.len();
                    w.write_all(a)
                }
                EscapedSexpWithSize::AtomOwned(a) => {
                    *already_written_on_line += a.len();
                    w.write_all(a)
                }
                EscapedSexpWithSize::List { values, .. } => {
                    *already_written_on_line += 1;
                    write_u8(b'(', w)?;
                    for (index, elem) in values.iter().enumerate() {
                        write_loop(
                            elem,
                            index == 0,
                            indent_level + 1,
                            already_written_on_line,
                            w,
                        )?;
                    }
                    *already_written_on_line += 1;
                    write_u8(b')', w)?;
                    Ok(())
                }
            }
        }
        let s = escape(self);
        write_loop(&s, true, 0, &mut 0, w)
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

    /// Serialize multiple Sexps to a buffer.
    ///
    /// # Example
    ///
    /// ```
    ///     let sexps = rsexp::from_slice_multi(b"()((foo bar)(baz (1 2 3)))").unwrap();
    ///     assert_eq!(rsexp::Sexp::to_bytes_multi(&sexps), b"() ((foo bar) (baz (1 2 3)))");
    /// ```
    pub fn to_bytes_multi(sexps: &[Self]) -> Vec<u8> {
        let mut buffer = Vec::new();
        Self::write_multi(sexps, &mut buffer).unwrap();
        buffer
    }

    /// Serialize a Sexp to a buffer, machine readable version.
    ///
    /// # Example
    ///
    /// ```
    ///     let sexp = rsexp::from_slice(b"((foo bar)(baz (1 2 3)))").unwrap();
    ///     assert_eq!(sexp.to_bytes_mach(), b"((foo bar)(baz(1 2 3)))");
    /// ```
    pub fn to_bytes_mach(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write_mach(&mut buffer).unwrap();
        buffer
    }

    /// Serialize a Sexp to a buffer, human readable version.
    ///
    /// # Example
    ///
    /// ```
    ///     let sexp = rsexp::from_slice(b"((foo bar)(baz (1 2 3)))").unwrap();
    ///     assert_eq!(sexp.to_bytes_hum(), b"((foo bar) (baz (1 2 3)))");
    /// ```
    pub fn to_bytes_hum(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write_hum(&mut buffer).unwrap();
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
