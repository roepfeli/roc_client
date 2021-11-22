use command::Command;
use std::thread;

use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod command;
mod message;

fn thread_recieve(mut stream: TcpStream, shoud_exit: Arc<AtomicBool>) {
    stream.set_read_timeout(Some(std::time::Duration::new(0, 20_000)))
        .expect("ERROR: Could not set read-timeout. Exiting recieve-thread. You might want to reconnect to the server.");

    while !shoud_exit.load(Ordering::Relaxed) {
        let message = match message::get_message(&mut stream) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                ErrorKind::ConnectionAborted => {
                    println!("Server aborted connection.");
                    break;
                }
                ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                    continue;
                }
                _ => {
                    println!("WARNING: Error when recieving message: {}", e);
                    continue;
                }
            },
        };

        println!("{}", message.to_string());
    }
}

fn disconnect(
    stream: Option<TcpStream>,
    thread_handle: Option<thread::JoinHandle<()>>,
    thread_killer: &Option<Arc<AtomicBool>>,
) {
    if let Some(handle) = thread_handle {
        println!("Shutting down recieve-thread");

        if let Some(killer) = thread_killer {
            killer.store(true, Ordering::Relaxed);
            // TODO: put in timeout
            handle.join().unwrap();
        } else {
            println!(
                "ERROR: Handle but no killer-atomic-bool! Trying to shutdown stream anyway..."
            );
            // TODO: put in timeout
            handle.join().unwrap();
        }
    }

    if let Some(v) = stream {
        drop(v);
    }
}

fn connect(adress: String) -> Option<(TcpStream, thread::JoinHandle<()>, Arc<AtomicBool>)> {
    let stream = match TcpStream::connect(&adress) {
        Ok(v) => v,
        Err(e) => {
            println!("Unable to connect to given adress: {}", adress);
            println!("Reason: {}", e);
            return None;
        }
    };

    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_cloned = should_exit.clone();

    let stream_cloned = stream
        .try_clone()
        .expect("ERROR: Could not clone TcpStream. Failed crucial operation, exiting...");

    let handle = thread::spawn(|| {
        thread_recieve(stream_cloned, should_exit_cloned);
    });

    Some((stream, handle, should_exit))
}

fn main() {
    let mut stream = None;
    let mut thread_handle = None;
    let mut thread_killer = None;

    loop {
        match Command::get_from_stdin() {
            Command::Text(v) => {
                if let Err(e) = message::send_text(v, &mut stream) {
                    println!("ERROR: Could not send text-message: {}", e);
                }
            }
            Command::RegisterUsername(v) => {
                if let Err(e) = message::send_register_username(v, &mut stream) {
                    println!("ERROR: Could not send register-username-message: {}", e);
                }
            }
            Command::Connect(v) => {
                if let Some((new_stream, new_thread_handle, new_thread_killer)) = connect(v) {
                    stream = Some(new_stream);
                    thread_handle = Some(new_thread_handle);
                    thread_killer = Some(new_thread_killer);
                }
            }
            Command::Disconnect => {
                disconnect(stream, thread_handle, &thread_killer);
                stream = None;
                thread_handle = None;
                thread_killer = None;
            }
            Command::Quit => {
                break;
            }
        }
    }

    disconnect(stream, thread_handle, &thread_killer);
}
