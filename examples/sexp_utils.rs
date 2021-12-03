// TODO: use tokio?
use clap::Parser;
use rsexp::Sexp;
use tracing::{event, Level};

#[derive(Parser)]
#[clap(version = "1.0", author = "Laurent Mazare <lmazare@gmail.com>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// Run some benchmark converting repeatedly an in-memory sexp string to and
    /// from a sexp object.
    Bench(Bench),
    /// Read a sexp and print it back.
    Print(Print),
}

#[derive(Parser)]
struct Bench {
    /// The sexp file to use as input.
    #[clap(short, long)]
    input_filename: String,

    /// When specified, write the sexp in the specified file.
    #[clap(short, long)]
    output_filename: Option<String>,

    /// The number of times to run the to and of sexp conversions.
    #[clap(long, default_value = "1")]
    iterations: u32,
}

#[derive(Parser)]
struct Print {
    /// The sexp file to use as input.
    #[clap(short, long)]
    input_filename: String,

    /// When set, print the machine readable version rather than the human readable one.
    #[clap(short, long)]
    mach: bool,
}

impl Print {
    fn run(&self) -> std::io::Result<()> {
        let contents = std::fs::read(&self.input_filename)?;
        let sexp = rsexp::from_slice(&contents).unwrap();
        if self.mach {
            sexp.write_mach(&mut std::io::stdout())?;
        } else {
            sexp.write_hum(&mut std::io::stdout())?;
        }
        println!("");
        Ok(())
    }
}

fn cnt_loop(s: &Sexp) -> (usize, usize) {
    match s {
        Sexp::Atom(atom) => (1, atom.len()),
        Sexp::List(list) => {
            let mut cnt_atoms = 0;
            let mut cnt_bytes = 0;
            for elem in list.iter() {
                let (tmp_atoms, tmp_bytes) = cnt_loop(&elem);
                cnt_atoms += tmp_atoms;
                cnt_bytes += tmp_bytes;
            }
            (cnt_atoms, cnt_bytes)
        }
    }
}

impl Bench {
    fn run(&self) -> std::io::Result<()> {
        event!(Level::INFO, "reading {}", self.input_filename);
        let contents = std::fs::read(&self.input_filename)?;
        event!(Level::INFO, "read {} bytes", contents.len());
        let mut value = Sexp::List(vec![]);
        for _index in 0..self.iterations {
            value = rsexp::from_slice(&contents).unwrap();
        }
        event!(Level::INFO, "converted to sexp {} times", self.iterations);
        let sexp = rsexp::from_slice(&contents).unwrap();
        let (cnt_atoms, cnt_bytes) = cnt_loop(&sexp);
        event!(Level::INFO, "found {} atoms, total of {} bytes", cnt_atoms, cnt_bytes);
        for _index in 0..self.iterations {
            let _contents = value.to_bytes();
        }
        event!(Level::INFO, "converted to buffer {} times", self.iterations);
        if let Some(output_filename) = &self.output_filename {
            let data = value.to_bytes();
            std::fs::write(output_filename, data)?;
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let subscriber =
        tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing::Level::TRACE).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match opts.subcmd {
        SubCommand::Bench(bench) => bench.run()?,
        SubCommand::Print(print) => print.run()?,
    };
    Ok(())
}
