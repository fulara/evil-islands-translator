use pseudo_parti::say_hello;
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    host: SocketAddr,
}

fn main() {
    let opts = Options::from_args();
    println!("slave says {} {:?}", say_hello(), opts);
}
