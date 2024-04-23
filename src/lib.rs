mod utils;
use crate::utils::connection_manager::dispatch_commands;
use crate::utils::engine::{execute_transactions, Transaction};
use rsheet_lib::connect::Manager;
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();

    let (tx, rx): (Sender<Transaction>, Receiver<Transaction>) = mpsc::channel();

    let database_thread = thread::spawn(move || execute_transactions(rx));

    while let Ok((recv, send)) = manager.accept_new_connection() {
        let tx_clone = tx.clone();
        let handle = thread::spawn(move || dispatch_commands(recv, send, tx_clone));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    drop(tx);
    database_thread.join().unwrap();
    Ok(())
}
