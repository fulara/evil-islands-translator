use pseudo_parti::{say_hello, setup_signal_handler};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    port: u16,
}

fn slave(_stream: TcpStream) {}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let interrupted = Arc::new(AtomicBool::new(false));
    setup_signal_handler(interrupted)?;
    let address = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(&address)?;
    println!("Now listening on port: {}", opts.port);
    let (stream, source) = listener.accept()?;
    println!("Connection accepted from: {:?}", source);
    thread::Builder::new()
        .name("c_0".into())
        .spawn(|| {
            slave(stream);
        })
        .unwrap();
    println!("master says {} opts: {:?}", say_hello(), opts);
    Ok(())
}
