use serde::{Deserialize, Serialize};
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level::signal_name;
use std::convert::TryInto;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{mem, thread};

pub fn say_hello() -> &'static str {
    "hello"
}

pub fn setup_signal_handler(interrupted: Arc<AtomicBool>) -> anyhow::Result<()> {
    let mut signals = Signals::new(&[SIGINT, SIGQUIT, SIGTERM])?;
    thread::spawn(move || {
        for signal in signals.forever() {
            println!(
                "Received signal: {:?}:{}. setting interrupted",
                signal_name(signal),
                signal
            );
            interrupted.store(true, Ordering::Relaxed);
        }
    });
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Action {}

pub enum WriteState {
    WritingHeader(usize, [u8; 2], usize, Vec<u8>),
    WritingBody(usize, Vec<u8>),
}

pub fn network_send(
    stream: &mut TcpStream,
    action: &Action,
    state: Option<WriteState>,
) -> anyhow::Result<Option<WriteState>> {
    // Maybe Create a writer type that will set timeout on construction?
    if stream.write_timeout()?.is_none() {
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .expect("Failed to set_read_timeout");
    }
    let mut state = match state {
        None => {
            let data = serde_json::to_vec(&action).expect("Failed to serialize?");
            let len: u16 = data.len().try_into().expect("serialized len exceeds u16");
            WriteState::WritingHeader(0, len.to_be_bytes(), 0, data)
        }
        Some(state) => state,
    };

    loop {
        match &mut state {
            WriteState::WritingHeader(header_offset, header, payload_offset, payload) => {
                match stream.write(&header[*header_offset..]) {
                    Ok(written) => {
                        *header_offset += written;
                        if *header_offset == header.len() {
                            state = WriteState::WritingBody(*payload_offset, mem::take(payload))
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        return Ok(Some(state));
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            WriteState::WritingBody(payload_offset, payload) => {
                match stream.write(&payload[*payload_offset..]) {
                    Ok(written) => {
                        *payload_offset += written;
                        if *payload_offset == payload.len() {
                            return Ok(None);
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        return Ok(Some(state));
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
    }
}
pub enum ReadState {
    ReadingHeader(usize, [u8; 2]),
    ReadingBody(usize, Vec<u8>),
}

pub fn network_try_read(
    stream: &mut TcpStream,
    state: &mut ReadState,
) -> anyhow::Result<Option<Action>> {
    // Maybe Create a reader type that will set timeout on construction?
    if stream.read_timeout()?.is_none() {
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("Failed to set_read_timeout");
    }
    match state {
        ReadState::ReadingHeader(offset, buf) => match stream.read(&mut buf[*offset..]) {
            Ok(0) => Err(anyhow::format_err!("Disconnected")),
            Ok(read) => {
                *offset += read;
                if *offset == buf.len() {
                    let len: usize = u16::from_be_bytes(*buf).into();
                    *state = ReadState::ReadingBody(len, Vec::with_capacity(len));
                }
                Ok(None)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        },
        ReadState::ReadingBody(offset, buf) => match stream.read(&mut buf[*offset..]) {
            Ok(0) => Err(anyhow::format_err!("Disconnected")),
            Ok(read) => {
                *offset += read;
                if *offset == buf.len() {
                    let action: Action =
                        serde_json::from_slice(buf).expect("Failed to deserialize message");
                    *state = ReadState::ReadingHeader(0, [0u8; 2]);
                    return Ok(Some(action));
                }
                Ok(None)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        },
    }
}
