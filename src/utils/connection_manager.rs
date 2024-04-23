use std::sync::mpsc;
use rsheet_lib::connect::{Reader, Writer};
use crate::utils::command::parse_command;
use crate::utils::engine::Transaction;


pub fn dispatch_commands(mut recv: impl Reader, mut send: impl Writer, transactions_sender: mpsc::Sender<Transaction>) {
    loop {
        match recv.read_message() {
            Ok(msg) => {
                let command = parse_command(&msg);
                if let Some(response) = command.execute(&transactions_sender){
                    if let Err(_) = send.write_message(response) { break; };
                }
            },
            Err(_) => break,
        }
    }
}

