mod utils;
use rsheet_lib::connect::{Manager, Reader, Writer};
use std::error::Error;
use std::thread;
use crate::utils::server_backend::handle_connection;


pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
    where M: Manager,
{
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    loop {
        match manager.accept_new_connection() { // daemon
            Ok((recv, send)) => {
                // Spawn server thread
                let handle = thread::spawn(move || handle_connection(recv, send));
                handles.push(handle);
            },
            Err(_) => break,
        };
    }
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}

