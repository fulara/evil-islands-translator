use pseudo_parti::{say_hello, setup_signal_handler};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    host: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let interrupted = Arc::new(AtomicBool::new(false));
    setup_signal_handler(interrupted)?;
    println!("Slave connecting to master on: {:?}", opts.host);
    TcpStream::connect(opts.host)?;
    println!("Connection established.");
    println!("slave says {} {:?}", say_hello(), opts);
    Ok(())
}
