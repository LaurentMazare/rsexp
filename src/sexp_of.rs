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

macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: SexpOf),+> SexpOf for ($($name,)+)
        {
            #[allow(non_snake_case)]
            fn sexp_of(&self) -> Sexp {
                let ($($name,)+) = self;
                list(&[$($name.sexp_of(),)+])
            }

        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }

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
