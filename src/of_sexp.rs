use crate::{Sexp, UseToString};
use std::collections::{BTreeMap, HashMap};

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
    ExpectedPairForMapGotAtom {
        type_: &'static str,
    },
    DuplicateKeyInMap {
        type_: &'static str,
        key: Option<String>,
    },
    ExpectedPairForMapGotList {
        type_: &'static str,
        list_len: usize,
    },
    ListLengthMismatch {
        type_: &'static str,
        expected_len: usize,
        list_len: usize,
    },
    StringConversionError {
        err: String,
    },
    MissingFieldsInStruct {
        type_: &'static str,
        field: &'static str,
    },
    ExtraFieldsInStruct {
        type_: &'static str,
        extra_fields: Vec<String>,
    },
    UnknownConstructorForEnum {
        type_: &'static str,
        constructor: String,
    },
    ExpectedConstructorGotEmptyList {
        type_: &'static str,
    },
    ExpectedConstructorGotListInList {
        type_: &'static str,
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
    pub fn extract_atom<'a>(&'a self, type_: &'static str) -> Result<&'a [u8], IntoSexpError> {
        match self {
            Sexp::Atom(atom) => Ok(atom),
            Sexp::List(list) => Err(IntoSexpError::ExpectedAtomGotList {
                type_,
                list_len: list.len(),
            }),
        }
    }

    pub fn extract_list<'a>(&'a self, type_: &'static str) -> Result<&'a [Self], IntoSexpError> {
        match self {
            Sexp::List(list) => Ok(list),
            Sexp::Atom(_) => Err(IntoSexpError::ExpectedListGotAtom { type_ }),
        }
    }

    /// Extracts the constructor and fields for an Enum.
    pub fn extract_enum<'a>(
        &'a self,
        type_: &'static str,
    ) -> Result<(&'a [u8], &'a [Self]), IntoSexpError> {
        match self {
            Sexp::Atom(ref atom) => Ok((atom, &[])),
            Sexp::List(list) if list.is_empty() => {
                Err(IntoSexpError::ExpectedConstructorGotEmptyList { type_ })
            }
            Sexp::List(ref list) => match list[0] {
                Sexp::Atom(ref atom) => Ok((atom, &list[1..])),
                Sexp::List(_) => Err(IntoSexpError::ExpectedConstructorGotListInList { type_ }),
            },
        }
    }

    pub fn extract_map<'a>(
        list: &'a [Self],
        type_: &'static str,
    ) -> Result<HashMap<&'a [u8], &'a Self>, IntoSexpError> {
        let mut map = HashMap::new();
        for elem in list.iter() {
            match elem {
                Sexp::Atom(_atom) => {
                    return Err(IntoSexpError::ExpectedPairForMapGotAtom { type_ })
                }
                Sexp::List(list) => match list.as_slice() {
                    [Sexp::Atom(key), value] => {
                        if map.insert(key.as_slice(), value).is_some() {
                            return Err(IntoSexpError::DuplicateKeyInMap {
                                type_,
                                key: Some(String::from_utf8_lossy(key).to_string()),
                            });
                        }
                    }
                    list => {
                        return Err(IntoSexpError::ExpectedPairForMapGotList {
                            type_,
                            list_len: list.len(),
                        })
                    }
                },
            }
        }
        Ok(map)
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

impl OfSexp for () {
    fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
        match s.extract_list("()")? {
            [] => Ok(()),
            l => Err(IntoSexpError::ListLengthMismatch {
                type_: "()",
                expected_len: 0,
                list_len: l.len(),
            }),
        }
    }
}

macro_rules! of_sexp_map {
    ($container_name:ident) => {
        fn of_sexp(s: &Sexp) -> Result<Self, IntoSexpError> {
            let type_ = stringify!($container_name);
            let list = s.extract_list(type_)?;
            let mut map = $container_name::new();
            for elem in list.iter() {
                match elem {
                    Sexp::Atom(_atom) => {
                        return Err(IntoSexpError::ExpectedPairForMapGotAtom { type_ })
                    }
                    Sexp::List(list) => match list.as_slice() {
                        [key, value] => {
                            if map
                                .insert(OfSexp::of_sexp(key)?, OfSexp::of_sexp(value)?)
                                .is_some()
                            {
                                return Err(IntoSexpError::DuplicateKeyInMap { type_, key: None });
                            }
                        }
                        list => {
                            return Err(IntoSexpError::ExpectedPairForMapGotList {
                                type_,
                                list_len: list.len(),
                            })
                        }
                    },
                }
            }
            Ok(map)
        }
    };
}

impl<K, V> OfSexp for std::collections::HashMap<K, V>
where
    K: OfSexp + Eq + std::hash::Hash,
    V: OfSexp,
{
    of_sexp_map!(HashMap);
}

impl<K, V> OfSexp for BTreeMap<K, V>
where
    K: OfSexp + Ord,
    V: OfSexp,
{
    of_sexp_map!(BTreeMap);
}
