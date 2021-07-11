use pseudo_parti::say_hello;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    port: u16,
}

fn main() {
    let opts = Options::from_args();
    println!("master says {} opts: {:?}", say_hello(), opts);
}
