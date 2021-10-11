# rsexp
S-expression parsing and writing in Rust using [nom](https://github.com/Geal/nom) parser combinators.

[![Build Status](https://github.com/LaurentMazare/rsexp/workflows/Continuous%20integration/badge.svg)](https://github.com/LaurentMazare/rsexp/actions)
[![Latest version](https://img.shields.io/crates/v/rsexp.svg)](https://crates.io/crates/rsexp)
[![Documentation](https://docs.rs/rsexp/badge.svg)](https://docs.rs/rsexp)
![License](https://img.shields.io/crates/l/rsexp.svg)

This implemantion aims at being compatible with OCaml's [sexplib](https://github.com/janestreet/sexplib).
The main type for S-expression is as follows:
```rust
pub enum Sexp {
    Atom(Vec<u8>),
    List(Vec<Sexp>),
}
```

## Reading and Writing Sexp Files

Reading a sexp file can be done by first reading the whole
file content and then converting the resulting slice into
a `Sexp` object as follows:

```rust
    let contents = std::fs::read(input_filename)?;
    let sexp = rsexp::from_slice(&contents)?;
```

Writing a sexp file can be done by first serializing the data
to a buffer then writing this buffer out. Alternatively, the
`sexp.write(w)?` function can be used to directly output the
data to a `w` object that implements the `Write` trait.

```rust
    let data = sexp.to_bytes();
    std::fs::write(output_filename, data)?;
```

## Conversion to/from Native Types

The `OfSexp` and `SexpOf` traits define some functions to
convert a given type from/to a `Sexp`. These traits are implemented
for most basic types, including maps. Two associated derive macros
can be used to define these traits on structs and enums.

```rust
#[derive(OfSexp, SexpOf)]
struct StructXYZ {
    x: i64,
    y: Option<(i32, i32)>,
    z: String,
}

#[derive(OfSexp, SexpOf)]
enum Abc {
    A(StructXYZ),
    B { x: i64, z: String },
    C,
}


    let sexp = abc.sexp_of();
    let abc: Abc = sexp.of_sexp()?;
```
