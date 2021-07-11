use pseudo_parti::{network_try_read, setup_signal_handler, ReadState};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
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
    let connection = loop {
        match TcpStream::connect_timeout(&opts.host, Duration::from_secs(10)) {
            Ok(conn) => {
                break conn;
            }
            Err(_) => {
                println!("Connection failed, will retry")
            }
        }
    };
    println!("Connection established.");
    connection.set_read_timeout(Some(Duration::from_millis(1000)))?;
    connection.set_write_timeout(Some(Duration::from_millis(1000)))?;
    let mut connection = mio::net::TcpStream::from_std(connection);
    let mut state = ReadState::default();
    while !interrupted.load(Ordering::Relaxed) {
        if let Some(action) = network_try_read(&mut connection, &mut state)? {
            println!("Client got action: {:?}", action);
        }
    }
    Ok(())
}
