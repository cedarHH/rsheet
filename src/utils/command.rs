use crate::utils::database::{database_get_value, split_cell_id};
use rsheet_lib::command_runner::CellValue;
use rsheet_lib::replies::Reply;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::utils::engine::Transaction;

pub enum Command {
    Set(String),
    Get(String),
    Unsupported,
}

impl Command {
    // execute a command
    pub fn execute(&self, transactions_sender: &Sender<Transaction>) -> Option<Reply> {
        match self {
            Command::Set(args) => Self::handle_set(args, transactions_sender),
            Command::Get(args) => Some(Self::handle_get(args)),
            Command::Unsupported => Some(Reply::Error(String::from("Unsupported Command"))),
        }
    }

    fn handle_set(args: &str, transactions_sender: &Sender<Transaction>) -> Option<Reply> {
        let args_list: Vec<String> = args.splitn(2, ' ').map(|s| s.to_string()).collect();
        if args_list.len() == 1 {
            return Some(Reply::Error(format!(
                "Error: Error parsing request: {}",
                args
            )));
        }

        // Send set request to worker thread for dependency update
        let (resp_tx, resp_rx): (Sender<Option<Reply>>, Receiver<Option<Reply>>) = mpsc::channel();
        let transaction = Transaction::new(args_list, resp_tx);
        transactions_sender.send(transaction).unwrap();

        // Wait for results to return
        resp_rx.recv().unwrap()
    }

    fn handle_get(args: &String) -> Reply {
        let args_list: Vec<_> = args.split_whitespace().collect();
        if args_list.len() != 1 {
            return Reply::Error(format!("Error: Error parsing request: {}", args));
        }
        if let Some(cell_position) = split_cell_id(args_list[0]) {
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Querying the database to get the value of a cell
            let cell_ref = database_get_value(&cell_position);
            if cell_ref.dependency.is_some() {
                if let CellValue::Error(e) = cell_ref.cell_value {
                    return Reply::Error(e);
                }
            }
            Reply::Value(args_list[0].to_string(), cell_ref.cell_value)
        } else {
            Reply::Error(format!("Error: Invalid Key Provided: {}", args))
        }
    }
}

pub fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    match parts.as_slice() {
        ["set", args] => Command::Set(args.to_string()),
        ["get", args] => Command::Get(args.to_string()),
        _ => Command::Unsupported,
    }
}
