use crate::{atom, list, BytesSlice, Sexp, UseToString};

pub trait SexpOf {
    fn sexp_of(&self) -> Sexp;
}

impl<T: ToString + UseToString> SexpOf for T {
    fn sexp_of(&self) -> Sexp {
        atom(self.to_string().as_bytes())
    }
}

impl SexpOf for String {
    fn sexp_of(&self) -> Sexp {
        atom(self.as_bytes())
    }
}

impl SexpOf for &str {
    fn sexp_of(&self) -> Sexp {
        atom(self.as_bytes())
    }
}

impl<'a> SexpOf for BytesSlice<'a> {
    fn sexp_of(&self) -> Sexp {
        atom(self.0)
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
        list(&[self.0.sexp_of(), self.1.sexp_of()])
    }
}

impl<T1, T2, T3> SexpOf for (T1, T2, T3)
where
    T1: SexpOf,
    T2: SexpOf,
    T3: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        list(&[self.0.sexp_of(), self.1.sexp_of(), self.2.sexp_of()])
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

impl<T> SexpOf for Option<T>
where
    T: SexpOf,
{
    fn sexp_of(&self) -> Sexp {
        match self {
            None => list(&[]),
            Some(value) => list(&[value.sexp_of()]),
        }
    }
}

impl SexpOf for () {
    fn sexp_of(&self) -> Sexp {
        list(&[])
    }
}
