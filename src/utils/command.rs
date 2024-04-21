use std::error::Error;
use std::fmt;
use rsheet_lib::cells::{column_name_to_number,column_number_to_name};
use rsheet_lib::replies::Reply;
use rsheet_lib::command_runner::{CellValue,CommandRunner,CellArgument};

enum Command {
    Set(String),
    Get(String),
    Unsupported,
}

impl Command {
    pub fn execute(&self) -> Reply {
        match self {
            Command::Set(args) => {
                Self::handle_set(args)
            },
            Command::Get(args) => {
                Self::handle_get(args)
            },
            Command::Unsupported => {
                Reply::Error(String::from("Unsupported Command"))
            }
        }
    }

    fn handle_set(args: &String) -> Reply {
        let args_list: Vec<_> = args.split_whitespace().collect();
        if args_list.len() != 2{ return Reply::Error(format!("Error: Error parsing request: {}", args))}
        let runner = CommandRunner::new(args_list[1]);
        //runner.run();
        todo!()
    }

    fn handle_get(args: &String) -> Reply {
        todo!()
    }
}

#[derive(Debug)]
struct RsheetError {
    details: String,
}

impl RsheetError {
    fn new(msg: &str) -> RsheetError {
        RsheetError { details: msg.to_string() }
    }
}

impl fmt::Display for RsheetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RsheetError {}

pub fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    match parts.as_slice() {
        ["set", args] => Command::Set(args.to_string()),
        ["get", args] => Command::Get(args.to_string()),
        _ => Command::Unsupported,
    }
}