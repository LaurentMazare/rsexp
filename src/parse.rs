// TODO: Block comments.
use nom::{
    branch::alt,
    character::complete::char,
    error::{context, Error, ErrorKind, ParseError},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
    IResult, InputTake,
};

use crate::Sexp;

type Res<T, U> = IResult<T, U, Error<T>>;

fn space_or_comments(input: &[u8]) -> Res<&[u8], &[u8]> {
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b' ' | b'\t' | b'\r' | b'\n' => index += 1,
            b';' => {
                while index < input.len() && input[index] != b'\r' && input[index] != b'\n' {
                    index += 1
                }
            }
            _ => return Ok(input.take_split(index)),
        }
    }
    Ok(input.take_split(input.len()))
}

fn unquoted_string_(input: &[u8]) -> Res<&[u8], &[u8]> {
    // Most errors below are handled with Failure rather than error
    // as these cannot be recovered from.
    for (index, &c) in input.iter().enumerate() {
        match c {
            b';' | b'(' | b')' | b'"' | b' ' | b'\t' | b'\r' | b'\n' => {
                return Ok(input.take_split(index));
            }
            b'#' if index > 0 && input[index - 1] == b'|' => {
                return Err(nom::Err::Failure(Error::from_error_kind(
                    input,
                    ErrorKind::Not,
                )));
            }
            b'|' if index > 0 && input[index - 1] == b'#' => {
                return Err(nom::Err::Failure(Error::from_error_kind(
                    input,
                    ErrorKind::Not,
                )));
            }
            _ => {}
        }
    }
    Ok(input.take_split(input.len()))
}

fn unquoted_string(input: &[u8]) -> Res<&[u8], Vec<u8>> {
    match unquoted_string_(input) {
        Ok((next_input, atom)) => {
            if atom.is_empty() {
                Err(nom::Err::Error(Error::from_error_kind(
                    input,
                    ErrorKind::NonEmpty,
                )))
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
fn quoted_string(input: &[u8]) -> Res<&[u8], Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b'"' => {
                let (tail, _) = input.take_split(index);
                return Ok((tail, buffer));
            }
            b'\\' => {
                index += 1;
                if index == input.len() {
                    // Unexpected eof
                    return Err(nom::Err::Failure(Error::from_error_kind(
                        input,
                        ErrorKind::Eof,
                    )));
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
    Err(nom::Err::Failure(Error::from_error_kind(
        input,
        ErrorKind::Eof,
    )))
}

fn atom(input: &[u8]) -> Res<&[u8], Sexp> {
    context(
        "atom",
        alt((
            unquoted_string,
            delimited(char('"'), quoted_string, char('"')),
        )),
    )(input)
    .map(|(next_input, atom)| (next_input, Sexp::Atom(atom)))
}

fn sexp_in_list(input: &[u8]) -> Res<&[u8], Sexp> {
    context(
        "sexp-in-list",
        delimited(
            pair(char('('), space_or_comments),
            many0(sexp_no_leading_blank),
            char(')'),
        ),
    )(input)
    .map(|(next_input, res)| (next_input, Sexp::List(res)))
}

// This is used to encode a list separated by spaces as the
// separated_list combinator does not seem to handle separators that
// can be empty.
fn sexp_no_leading_blank(input: &[u8]) -> Res<&[u8], Sexp> {
    terminated(alt((atom, sexp_in_list)), space_or_comments)(input)
}

/// Deserialize a Sexp from bytes, returning both the sexp and the remaining
/// bytes.
pub fn from_slice_allow_remaining<T: AsRef<[u8]> + ?Sized>(input: &T) -> Res<&[u8], Sexp> {
    preceded(space_or_comments, sexp_no_leading_blank)(input.as_ref())
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
pub fn from_slice<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Sexp, nom::Err<Error<&[u8]>>> {
    let input = input.as_ref();
    let (remaining, sexp) = from_slice_allow_remaining(input)?;
    if remaining.is_empty() {
        Ok(sexp)
    } else {
        Err(nom::Err::Failure(Error::from_error_kind(
            remaining,
            ErrorKind::Eof,
        )))
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

pub fn from_slice_multi<T: AsRef<[u8]> + ?Sized>(
    input: &T,
) -> Result<Vec<Sexp>, nom::Err<Error<&[u8]>>> {
    let input = input.as_ref();
    let (remaining, sexps) = preceded(space_or_comments, many0(sexp_no_leading_blank))(input)?;
    if remaining.is_empty() {
        Ok(sexps)
    } else {
        Err(nom::Err::Failure(Error::from_error_kind(
            remaining,
            ErrorKind::Eof,
        )))
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
            Ok(Sexp::List(vec![
                atom(b"foo bar"),
                atom(b"baz"),
                atom(b"x\""),
            ]))
        );
        assert_eq!(from_slice(b"\"\""), Ok(atom(b"")));
        assert_eq!(from_slice(b"\"\\000A\\123\""), Ok(atom(b"\0A\x7B")));
        assert_eq!(
            from_slice(b"\"\\000A\\x7B\\x99\\x9Z\""),
            Ok(atom(b"\0A\x7B\x99\\x9Z"))
        );
        assert_eq!(from_slice(b"( )"), Ok(list(&[])));
        assert_eq!(from_slice(b"()"), Ok(list(&[])));
        assert_eq!(from_slice(b"(())"), Ok(list(&[list(&[])])));
        assert_eq!(from_slice(b"(\"()\")"), Ok(list(&[atom(b"()")])));
        assert_eq!(from_slice(b"(\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice(b"\t (\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice(b" (\t\"\")"), Ok(list(&[atom(b"")])));
        assert_eq!(from_slice_multi(b""), Ok(vec![]));
        assert_eq!(from_slice_multi(b"()"), Ok(vec![list(&[])]));
        assert_eq!(
            from_slice_multi(b"(\t\t\t)()"),
            Ok(vec![list(&[]), list(&[])])
        );
        assert_eq!(
            from_slice_multi(b"(\"\\\\\\n\")"),
            Ok(vec![list(&[atom(b"\\\n")])])
        );
    }
}
