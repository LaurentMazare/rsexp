use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::thread_rng;

fn make_n_random_characters(n: i64, alphabet: &Vec<char>) -> String {
    let mut rng = thread_rng();
    (0..n).map(|_| alphabet.choose(&mut rng).unwrap()).collect()
}

fn make_benchmark_string(
    num_repetitions: &[i64],
    str_len: i64,
    quoted: bool,
    alphabet: &Vec<char>,
) -> String {
    if let Some(len) = num_repetitions.get(0) {
        format!(
            "({})",
            std::iter::repeat(make_benchmark_string(
                &num_repetitions[1..],
                str_len,
                quoted,
                alphabet
            ))
            .take(*len as usize)
            .collect::<String>()
        )
    } else {
        let chars = make_n_random_characters(str_len, alphabet);
        if quoted {
            format!("\"{}\"", chars)
        } else {
            chars
        }
    }
}

fn parse_sexp(contents: &[u8]) {
    let _sexp = rsexp::from_slice(&contents).unwrap();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let alphabet: Vec<char> = (b'a'..=b'z').map(char::from).collect();

    for quoted in [true, false] {
        for (str_len, repetitions, depth) in
            [(4, 100, 1), (4, 100, 2), (10, 100, 2), (1000, 100, 1)]
        {
            let bench_name = format!(
                "{repetitions}_repetitions_{depth}_depth_{str_len}_strlen_{quoted}",
                repetitions = repetitions,
                depth = depth,
                str_len = str_len,
                quoted = (if quoted { "quoted" } else { "unquoted" })
            );
            let num_repetitions: Vec<i64> =
                std::iter::repeat(repetitions as i64).take(depth).collect();
            let sexp = make_benchmark_string(&num_repetitions, str_len, quoted, &alphabet);
            c.bench_function(&bench_name, |b| {
                b.iter(|| parse_sexp(black_box(sexp.as_bytes())))
            });
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
