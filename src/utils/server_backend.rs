use rsheet_lib::connect::{Reader, Writer};
use crate::utils::command::parse_command;


pub fn handle_connection(mut recv: impl Reader, mut send: impl Writer) {
    loop {
        match recv.read_message() {
            Ok(msg) => {
                let command = parse_command(&msg);
                let result = command.execute();
                if let Err(err) = send.write_message(result) { break; };
            },
            Err(_) => break,
        }
    }
}

