// TODO: Block comments.
use crate::Sexp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnexpectedCharInString(u8),
    UnexpectedEofInString,
    UnexpectedEof,
    EmptyAtom,
}

type Res<'a, T> = Result<(&'a [u8], T), Error>;

fn space_or_comments(input: &[u8]) -> Res<()> {
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b' ' | b'\t' | b'\r' | b'\n' => index += 1,
            b';' => {
                while index < input.len() && input[index] != b'\r' && input[index] != b'\n' {
                    index += 1
                }
            }
            _ => return Ok((&input[index..], ())),
        }
    }
    Ok((&[], ()))
}

fn unquoted_string_(input: &[u8]) -> Res<&[u8]> {
    for (index, &c) in input.iter().enumerate() {
        match c {
            b';' | b'(' | b')' | b'"' | b' ' | b'\t' | b'\r' | b'\n' => {
                let (str, remaining) = input.split_at(index);
                return Ok((remaining, str));
            }
            b'#' if index > 0 && input[index - 1] == b'|' => {
                return Err(Error::UnexpectedCharInString(b'|'))
            }
            b'|' if index > 0 && input[index - 1] == b'#' => {
                return Err(Error::UnexpectedCharInString(b'#'))
            }
            _ => {}
        }
    }
    Ok((&[], input))
}

fn unquoted_string(input: &[u8]) -> Res<Vec<u8>> {
    match unquoted_string_(input) {
        Ok((next_input, atom)) => {
            if atom.is_empty() {
                Err(Error::EmptyAtom)
            } else {
                Ok((next_input, atom.to_vec()))
            }
        }
        Err(err) => Err(err),
    }
}

fn digit(input: &[u8], index: usize) -> Option<u8> {
    if index >= input.len() {
        None
    } else {
        let c = input[index];
        if (b'0'..=b'9').contains(&c) {
            Some(c - b'0')
        } else {
            None
        }
    }
}

fn hex_digit(input: &[u8], index: usize) -> Option<u8> {
    if index >= input.len() {
        None
    } else {
        let c = input[index];
        if (b'0'..=b'9').contains(&c) {
            Some(c - b'0')
        } else if (b'A'..=b'F').contains(&c) {
            Some(c - b'A' + 10)
        } else if (b'a'..=b'f').contains(&c) {
            Some(c - b'a' + 10)
        } else {
            None
        }
    }
}

fn three_digits(input: &[u8], index: usize) -> Option<u8> {
    let d1 = digit(input, index)?;
    let d2 = digit(input, index + 1)?;
    let d3 = digit(input, index + 2)?;
    Some(100 * d1 + 10 * d2 + d3)
}

fn two_hex_digits(input: &[u8], index: usize) -> Option<u8> {
    let d1 = hex_digit(input, index)?;
    let d2 = hex_digit(input, index + 1)?;
    Some(16 * d1 + d2)
}

// Maybe this should be rewritten using combinators?
fn quoted_string(input: &[u8]) -> Res<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b'"' => {
                let (_, remaining) = input.split_at(index);
                return Ok((remaining, buffer));
            }
            b'\\' => {
                index += 1;
                if index == input.len() {
                    // Unexpected eof
                    return Err(Error::UnexpectedEofInString);
                }
                match input[index] {
                    b'\n' => {
                        while index + 1 < input.len() {
                            match input[index + 1] {
                                b' ' | b'\t' => index += 1,
                                _ => break,
                            }
                        }
                    }
                    b'\'' | b'"' | b'\\' => {
                        buffer.push(input[index]);
                    }
                    b'n' => {
                        buffer.push(b'\n');
                    }
                    b'r' => {
                        buffer.push(b'\r');
                    }
                    b't' => {
                        buffer.push(b'\t');
                    }
                    b'b' => {
                        buffer.push(b'\x08');
                    }
                    b'x' => match two_hex_digits(input, index + 1) {
                        Some(v) => {
                            index += 2;
                            buffer.push(v)
                        }
                        None => {
                            buffer.push(b'\\');
                            buffer.push(b'x');
                        }
                    },
                    c => match three_digits(input, index) {
                        Some(v) => {
                            index += 2;
                            buffer.push(v)
                        }
                        None => {
                            buffer.push(b'\\');
                            buffer.push(c);
                        }
                    },
                }
            }
            c => buffer.push(c),
        };
        index += 1;
    }
    Err(Error::UnexpectedEofInString)
}

fn first_char_is(c: u8, input: &[u8]) -> bool {
    input.first().map(|x| *x == c).unwrap_or(false)
}

fn char(c: u8, input: &[u8]) -> Res<()> {
    if first_char_is(c, input) {
        Ok((&input[1..], ()))
    } else {
        Err(Error::UnexpectedEof)
    }
}

fn atom(input: &[u8]) -> Res<Sexp> {
    let (next_input, atom) = if first_char_is(b'"', input) {
        let (input, ()) = char(b'"', input)?;
        let (input, atom) = quoted_string(input)?;
        let (input, ()) = char(b'"', input)?;
        (input, atom)
    } else {
        unquoted_string(input)?
    };
    Ok((next_input, Sexp::Atom(atom)))
}

fn sexp_in_list(input: &[u8]) -> Res<Sexp> {
    let (input, ()) = char(b'(', input)?;
    let (input, ()) = space_or_comments(input)?;
    let mut input = input;
    let mut res = vec![];
    while let Ok((next_input, sexp)) = sexp_no_leading_blank(input) {
        input = next_input;
        res.push(sexp)
    }
    let (input, ()) = char(b')', input)?;
    Ok((input, Sexp::List(res)))
}

// This is used to encode a list separated by spaces as the
// separated_list combinator does not seem to handle separators that
// can be empty.
fn sexp_no_leading_blank(input: &[u8]) -> Res<Sexp> {
    if first_char_is(b'(', input) {
        let (input, sexp) = sexp_in_list(input)?;
        let (input, ()) = space_or_comments(input)?;
        Ok((input, sexp))
    } else {
        let (input, sexp) = atom(input)?;
        let (input, ()) = space_or_comments(input)?;
        Ok((input, sexp))
    }
}

/// Deserialize a Sexp from bytes, returning both the sexp and the remaining
/// bytes.
pub fn from_slice_allow_remaining<T: AsRef<[u8]> + ?Sized>(input: &T) -> Res<Sexp> {
    let input = input.as_ref();
    let (input, ()) = space_or_comments(input)?;
    sexp_no_leading_blank(input)
}

/// Deserialize a Sexp from bytes. This fails if there are remaining bytes.
///
/// # Example
///
/// ```
///     let sexp = rsexp::from_slice(b"((foo bar)(baz (1 2 3)))").unwrap();
///     println!("{:?}", sexp);
///     if let rsexp::Sexp::List(l) = sexp {
///         assert_eq!(2, l.len());
///     }
/// ```
///
/// # Errors
///
/// This deserialization can fail if the bytes do not follow the expected
/// sexp format.
pub fn from_slice<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Sexp, Error> {
    let input = input.as_ref();
    let (remaining, sexp) = from_slice_allow_remaining(input)?;
    if remaining.is_empty() {
        Ok(sexp)
    } else {
        Err(Error::UnexpectedEof)
    }
}

/// Deserialize multiple Sexps from bytes. This fails if there are remaining bytes.
///
/// # Example
///
/// ```
///   let sexps = rsexp::from_slice_multi(b"(foo bar)(baz (1 2 3))").unwrap();
///   println!("{:?}", sexps);
///   assert_eq!(2, sexps.len());
/// ```
///
/// # Errors
///
/// This deserialization can fail if the bytes do not follow the expected
/// sexp format.

pub fn from_slice_multi<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Vec<Sexp>, Error> {
    let input = input.as_ref();
    let (input, ()) = space_or_comments(input)?;
    let mut input = input;
    let mut sexps = vec![];
    while let Ok((next_input, sexp)) = sexp_no_leading_blank(input) {
        input = next_input;
        sexps.push(sexp)
    }
    if input.is_empty() {
        Ok(sexps)
    } else {
        Err(Error::UnexpectedEof)
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_slice, from_slice_multi, Sexp};

    fn atom(b: &[u8]) -> Sexp {
        Sexp::Atom(b.to_vec())
    }

    fn list(l: &[Sexp]) -> Sexp {
        Sexp::List(l.to_vec())
    }

    #[test]
    fn basic_sexps() {
        assert_eq!(from_slice(b"( ATOM)"), Ok(Sexp::List(vec![atom(b"ATOM")])));
        assert_eq!(
            from_slice(b" ( \"foo bar\"   baz \"x\\\"\") "),
            Ok(Sexp::List(vec![atom(b"foo bar"), atom(b"baz"), atom(b"x\""),]))
        );
        assert_eq!(from_slice(b"\"\""), Ok(atom(b"")));
        assert_eq!(from_slice(b"\"\\000A\\123\""), Ok(atom(b"\0A\x7B")));
        assert_eq!(from_slice(b"\"\\000A\\x7B\\x99\\x9Z\""), Ok(atom(b"\0A\x7B\x99\\x9Z")));
        assert_eq!(from_slice(b"( )"), Ok(list(&[])));
        assert_eq!(from_slice(b"()"), Ok(list(&[])));
        assert_eq!(from_slice(b"(())"), Ok(list(&[list(&[])])));
        assert_eq!(from_slice(b"(\"()\")"), Ok(list(&[atom(b"()")])));
        assert_eq!(from_slice(b"(\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice(b"\t (\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice(b" (\t\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice_multi(b""), Ok(vec![]));
        assert_eq!(from_slice_multi(b"()"), Ok(vec![list(&[])]));
        assert_eq!(from_slice_multi(b"(\t\t\t)()"), Ok(vec![list(&[]), list(&[])]));
        assert_eq!(from_slice_multi(b"(\"\\\\\\n\")"), Ok(vec![list(&[atom(b"\\\n")])]));
    }
}
