use crossbeam_channel::Receiver;
use pseudo_parti::{setup_signal_handler, Action};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    port: u16,
}

fn slave(_stream: TcpStream, rx: Receiver<Action>) {
    while let Ok(action) = rx.recv() {
        println!("received: action: {:?} will propagate", action);
    }
    println!("Channel died, shutting down.");
}

fn accept_incoming_forever(
    listener: TcpListener,
    rx: Receiver<Action>,
    interrupted: &AtomicBool,
) -> anyhow::Result<()> {
    let mut counter = 0usize;
    while !interrupted.load(Ordering::Relaxed) {
        let name = format!("c_{}", counter);
        let (stream, source) = listener.accept()?;
        println!("Connection:{} accepted from: {:?}", name, source);
        thread::Builder::new()
            .name(name)
            .spawn({
                let rx = rx.clone();
                || {
                    slave(stream, rx);
                }
            })
            .unwrap();
        counter += 0;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let interrupted = Arc::new(AtomicBool::new(false));
    setup_signal_handler(interrupted.clone())?;
    let address = SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = TcpListener::bind(&address)?;
    println!("Now listening on port: {}", opts.port);
    let (_tx, rx) = crossbeam_channel::unbounded();
    accept_incoming_forever(listener, rx, &interrupted)?;
    Ok(())
}
