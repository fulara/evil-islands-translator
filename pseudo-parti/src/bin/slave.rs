use pseudo_parti::{network_try_read, setup_signal_handler, ReadState};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    host: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    let opts: Options = Options::from_args();
    let interrupted = Arc::new(AtomicBool::new(false));
    setup_signal_handler(interrupted.clone())?;
    println!("Slave connecting to master on: {:?}", opts.host);
    let mut connection = TcpStream::connect(opts.host)?;
    println!("Connection established.");
    while !interrupted.load(Ordering::Relaxed) {
        let mut state = ReadState::default();
        network_try_read(&mut connection, &mut state)?;
    }
    Ok(())
}
