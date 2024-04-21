use std::collections::HashMap;
use clap::builder::Str;
use rsheet_lib::cells::{column_name_to_number,column_number_to_name};
use rsheet_lib::replies::Reply;
use rsheet_lib::command_runner::{CellValue,CommandRunner,CellArgument};
use rsheet_lib::command_runner::CellArgument::Value;
use crate::utils::database::{CellRef, DATABASE, database_get_value};
use crate::utils::dependency_chain::{TopoError::CycleDetected,update_incoming_edges,find_topology_sort_of_weakly_component};

pub enum Command {
    Set(String),
    Get(String),
    Unsupported,
}

impl Command {
    pub fn execute(&self) -> Option<Reply> {
        match self {
            Command::Set(args) => {
                Self::handle_set(args)
            },
            Command::Get(args) => {
                Some(Self::handle_get(args))
            },
            Command::Unsupported => {
                Some(Reply::Error(String::from("Unsupported Command")))
            }
        }
    }

    fn handle_set(args: &str) -> Option<Reply> {
        let args_list: Vec<_> = args.split_whitespace().collect();
        if args_list.len() != 2{ return Some(Reply::Error(format!("Error: Error parsing request: {}", args)))}
        if let Some(cell_position) = split_cell_id(args_list[0]){
            let runner = CommandRunner::new(args_list[1]);
            let variables = runner.find_variables();
            let mut var_list = Vec::new();
            for id in variables.iter() {
                match split_cell_id(id) {
                    Some(result) => var_list.push(result),
                    None => return Some(Reply::Error(format!("Error: Invalid Key Provided: {}", id))),
                }
            }
            if var_list.len() == 0{
                let cell_value = runner.run(&HashMap::new());
                DATABASE.insert(cell_position, CellRef::new(cell_value, None));
            }
            else {
                DATABASE.insert(cell_position, CellRef::new(CellValue::None, Some(String::from(args_list[1]))));
            }
            update_incoming_edges(var_list,cell_position);
            match find_topology_sort_of_weakly_component(cell_position){
                Ok(topological_order) => {
                    for cell in topological_order.iter(){
                        let cell_value = database_get_value(cell);
                        match cell_value.dependency {
                            Some(expr) => {
                                let runner = CommandRunner::new(&expr);
                                let var_list = runner.find_variables();
                                let mut variables = HashMap::new();
                                for id in var_list.iter() {
                                    match split_cell_id(id) {
                                        Some(result) => {
                                            variables.insert(pos_to_cell_id(&result), Value(database_get_value(&result).cell_value));
                                        },
                                        None => unreachable!()
                                    }
                                }
                                DATABASE.insert(*cell, CellRef::new(runner.run(&variables),Some(expr)));
                            }
                            None => ()
                        }
                    }
                }
                Err(topo_error) => {
                    if let CycleDetected(cell_self_ref) = topo_error{
                        for cell in cell_self_ref.iter(){
                            let cell_value = database_get_value(cell);
                            DATABASE.insert(*cell, CellRef::new(CellValue::Error(
                                String::from(format!("Error: Cell {} is self-referential", pos_to_cell_id(cell)))),
                                                                cell_value.dependency));
                        }
                    }
                }
            };
        }
        else { return Some(Reply::Error(format!("Error: Invalid Key Provided: {}", args))) }
        None
    }

    fn handle_get(args: &String) -> Reply {
        let args_list: Vec<_> = args.split_whitespace().collect();
        if args_list.len() != 1{ return Reply::Error(format!("Error: Error parsing request: {}", args))}
        if let Some(cell_position) = split_cell_id(args_list[0]){
            let cell_value = database_get_value(&cell_position);
            Reply::Value(args_list[0].to_string(), cell_value.cell_value)
        }
        else { Reply::Error(format!("Error: Invalid Key Provided: {}", args)) }
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

fn split_cell_id(cell_id: &str) -> Option<(u32, u32)> {
    let mut split_index = 0;
    for (index, ch) in cell_id.chars().enumerate() {
        if ch.is_digit(10) {
            split_index = index;
            break;
        }
    }
    if split_index == 0 {
        None
    } else {
        let (col, row) = cell_id.split_at(split_index);
        row.parse::<u32>().ok().map(|row| (column_name_to_number(col), row))
    }
}

fn pos_to_cell_id(position:&(u32,u32)) -> String {
    format!("{}{}",column_number_to_name(position.0),&position.1.to_string())
}
