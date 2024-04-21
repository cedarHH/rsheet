use rsheet_lib::connect::{Reader, Writer};
use crate::utils::command::parse_command;


pub fn handle_connection(mut recv: impl Reader, mut send: impl Writer) {
    loop {
        match recv.read_message() {
            Ok(msg) => {
                let command = parse_command(&msg);
                let result = command.execute();
                if let Some(response) = result{
                    if let Err(err) = send.write_message(response) { break; };
                }
            },
            Err(_) => break,
        }
    }
}

