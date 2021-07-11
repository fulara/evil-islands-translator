use pseudo_parti::say_hello;
use std::net::{SocketAddr, TcpStream};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    host: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    println!("Slave connecting to master on: {:?}", opts.host);
    TcpStream::connect(opts.host)?;
    println!("Connection established.");
    println!("slave says {} {:?}", say_hello(), opts);
    Ok(())
}
