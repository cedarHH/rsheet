use crate::utils::command::parse_command;
use crate::utils::engine::Transaction;
use rsheet_lib::connect::{Reader, Writer};
use std::sync::mpsc;

pub fn dispatch_commands(
    mut recv: impl Reader,
    mut send: impl Writer,
    transactions_sender: mpsc::Sender<Transaction>,
) {
    while let Ok(msg) = recv.read_message() {
        let command = parse_command(&msg);
        if let Some(response) = command.execute(&transactions_sender) {
            if send.write_message(response).is_err() {
                break;
            };
        }
    }
}
