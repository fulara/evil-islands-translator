use crossbeam_channel::Receiver;
use pseudo_parti::{network_send, setup_signal_handler, Action};
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

fn slave(mut stream: TcpStream, rx: Receiver<Action>) -> anyhow::Result<()> {
    while let Ok(action) = rx.recv() {
        println!("received: action: {:?} will propagate", action);
        let mut state = network_send(&mut stream, &action, None)?;
        while let Some(new_state) = state {
            state = network_send(&mut stream, &action, Some(new_state))?;
        }
    }
    println!("Channel died, shutting down.");
    Ok(())
}

fn accept_incoming_forever(
    listener: TcpListener,
    rx: Receiver<Action>,
    interrupted: &AtomicBool,
) -> anyhow::Result<()> {
    let mut counter = 0usize;
    let mut handles = Vec::new();
    while !interrupted.load(Ordering::Relaxed) {
        let name = format!("c_{}", counter);
        match listener.accept() {
            Ok((stream, source)) => {
                println!("Connection:{} accepted from: {:?}", name, source);
                let handle = thread::Builder::new()
                    .name(name)
                    .spawn({
                        let rx = rx.clone();
                        || slave(stream, rx)
                    })
                    .unwrap();
                handles.push(handle);
                counter += 0;
            }
            Err(e) => {
                // TODO: read when this happens
                println!("Accept failed. {:?}", e);
            }
        }
    }

    println!("Joining handles.");
    for handle in handles {
        println!("{:?}", handle.join());
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
