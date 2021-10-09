use crate::{Sexp, UseToString};

// Conversion from Sexp to T

/// Errors that could be generated when converting a Sexp to a specific
/// type.
#[derive(Debug)]
pub enum IntoSexpError {
    Utf8Error(std::str::Utf8Error),
    FromUtf8Error(std::string::FromUtf8Error),
    ExpectedAtomGotList {
        type_: &'static str,
        list_len: usize,
    },
    ExpectedListGotAtom {
        type_: &'static str,
    },
    ListLengthMismatch {
        type_: &'static str,
        expected_len: usize,
        list_len: usize,
    },
    StringConversionError {
        err: String,
    },
}

impl From<std::str::Utf8Error> for IntoSexpError {
    fn from(e: std::str::Utf8Error) -> Self {
        IntoSexpError::Utf8Error(e)
    }
}

impl From<std::string::FromUtf8Error> for IntoSexpError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        IntoSexpError::FromUtf8Error(e)
    }
}

impl Sexp {
    fn extract_atom<'a>(&'a self, type_: &'static str) -> Result<&'a [u8], IntoSexpError> {
        match self {
            Sexp::Atom(atom) => Ok(atom),
            Sexp::List(list) => Err(IntoSexpError::ExpectedAtomGotList {
                type_,
                list_len: list.len(),
            }),
        }
    }
    fn extract_list<'a>(&'a self, type_: &'static str) -> Result<&'a [Sexp], IntoSexpError> {
        match self {
            Sexp::List(list) => Ok(list),
            Sexp::Atom(_) => Err(IntoSexpError::ExpectedListGotAtom { type_ }),
        }
    }
}

pub trait OfSexp {
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError>
    where
        Self: Sized;
}

impl OfSexp for String {
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        let atom = s.extract_atom("String")?;
        Ok(String::from_utf8(atom.to_vec())?)
    }
}

impl<T> OfSexp for T
where
    T: UseToString + std::str::FromStr,
    T::Err: std::fmt::Display,
{
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        let atom = s.extract_atom("stringable")?;
        let atom = std::str::from_utf8(atom)?;
        T::from_str(atom).map_err(|err| {
            let err = format!("{}", err);
            IntoSexpError::StringConversionError { err }
        })
    }
}

impl<T> OfSexp for Vec<T>
where
    T: OfSexp,
{
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        let list = s.extract_list("Vec")?;
        let mut res = Vec::new();
        for elem in list.iter() {
            res.push(T::of_sexp(elem)?)
        }
        Ok(res)
    }
}

impl<T1, T2> OfSexp for (T1, T2)
where
    T1: OfSexp,
    T2: OfSexp,
{
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        match s.extract_list("tuple2")? {
            [t1, t2] => {
                let t1 = T1::of_sexp(t1)?;
                let t2 = T2::of_sexp(t2)?;
                Ok((t1, t2))
            }
            l => Err(IntoSexpError::ListLengthMismatch {
                type_: "tuple2",
                expected_len: 2,
                list_len: l.len(),
            }),
        }
    }
}

impl<T1, T2, T3> OfSexp for (T1, T2, T3)
where
    T1: OfSexp,
    T2: OfSexp,
    T3: OfSexp,
{
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        match s.extract_list("tuple3")? {
            [t1, t2, t3] => {
                let t1 = T1::of_sexp(t1)?;
                let t2 = T2::of_sexp(t2)?;
                let t3 = T3::of_sexp(t3)?;
                Ok((t1, t2, t3))
            }
            l => Err(IntoSexpError::ListLengthMismatch {
                type_: "tuple3",
                expected_len: 3,
                list_len: l.len(),
            }),
        }
    }
}

impl<T> OfSexp for Option<T>
where
    T: OfSexp,
{
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        match s.extract_list("option")? {
            [] => Ok(None),
            [v] => Ok(Some(T::of_sexp(v)?)),
            l => Err(IntoSexpError::ListLengthMismatch {
                type_: "option",
                expected_len: 1,
                list_len: l.len(),
            }),
        }
    }
}
