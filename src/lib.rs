mod utils;
use rsheet_lib::connect::{Manager};
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use crate::utils::connection_manager::dispatch_commands;
use crate::utils::engine::{execute_transactions, Transaction};


pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
    where M: Manager,
{
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    let (tx, rx): (Sender<Transaction>, Receiver<Transaction>) = mpsc::channel();

    let database_thread = thread::spawn(move || { execute_transactions(rx) } );

    loop {
        match manager.accept_new_connection() { // daemon
            Ok((recv, send)) => {
                // Spawn server thread
                let tx_clone = tx.clone();
                let handle = thread::spawn(move || dispatch_commands(recv, send, tx_clone));
                handles.push(handle);
            },
            Err(_) => break,
        };
    }
    for handle in handles {
        handle.join().unwrap();
    }

    drop(tx);
    database_thread.join().unwrap();
    Ok(())
}