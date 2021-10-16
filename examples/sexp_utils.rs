// TODO: use tokio?
use clap::{AppSettings, Clap};
use rsexp::Sexp;
use tracing::{event, Level};

#[derive(Clap)]
#[clap(version = "1.0", author = "Laurent Mazare <lmazare@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Bench(Bench),
    Print(Print),
}

#[derive(Clap)]
struct Bench {
    #[clap(short, long)]
    input_filename: String,

    #[clap(short, long)]
    output_filename: Option<String>,

    #[clap(long, default_value = "1")]
    iterations: u32,

    #[clap(short)]
    verbose: bool,
}

#[derive(Clap)]
struct Print {
    #[clap(short, long)]
    input_filename: String,
}

impl Print {
    fn run(&self) -> std::io::Result<()> {
        let contents = std::fs::read(&self.input_filename)?;
        let sexp = rsexp::from_slice(&contents).unwrap();
        sexp.write_mach(&mut std::io::stdout())?;
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
        event!(
            Level::INFO,
            "found {} atoms, total of {} bytes",
            cnt_atoms,
            cnt_bytes
        );
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
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match opts.subcmd {
        SubCommand::Bench(bench) => bench.run()?,
        SubCommand::Print(print) => print.run()?,
    };
    Ok(())
}
