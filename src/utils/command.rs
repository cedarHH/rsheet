use std::collections::HashMap;
use rsheet_lib::cells::{column_name_to_number,column_number_to_name};
use rsheet_lib::replies::Reply;
use rsheet_lib::command_runner::{CellValue,CommandRunner,CellArgument};
use crate::utils::database::{CellRef, database_get_value, database_insert};
use crate::utils::dependency_chain::{TopoError::CycleDetected, update_incoming_edges, find_topology_sort_of_weakly_component};

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
        let args_list: Vec<_> = args.splitn(2, ' ').collect();
        if args_list.len() == 1{ return Some(Reply::Error(format!("Error: Error parsing request: {}", args)))}
        if let Some(cell_position) = split_cell_id(args_list[0]){
            let runner = CommandRunner::new(args_list[1]);
            let variables = runner.find_variables();
            let mut var_list = Vec::new();
            for var in variables.iter() {
                match &mut parse_variables(var) {
                    Some(result) => {
                        var_list.append(result)
                    }
                    None => return Some(Reply::Error(format!("Error: Invalid Key Provided: {}", var))),
                }
            }
            if var_list.len() == 0{
                let cell_value = runner.run(&HashMap::new());
                database_insert(cell_position, CellRef::new(cell_value, None));
            }
            else {
                database_insert(cell_position, CellRef::new(CellValue::None, Some(String::from(args_list[1]))));
            }
            update_incoming_edges(var_list,cell_position);

            match find_topology_sort_of_weakly_component(cell_position){
                Ok(topological_order) => {
                    for cell in topological_order.iter(){
                        let cell_value = database_get_value(cell);
                        match cell_value.dependency {
                            Some(expr) => {
                                let mut error_flag = false;
                                let runner = CommandRunner::new(&expr);
                                let var_list = runner.find_variables();
                                let mut variables = HashMap::new();
                                for id in var_list.iter() {
                                    match parse_cell_range(id) {
                                        Some(cell_arg) => {variables.insert(id.clone(), cell_arg);}
                                        None => {
                                            error_flag = true;
                                            break
                                        }
                                    }
                                }
                                if error_flag{
                                    database_insert(*cell, CellRef::new(CellValue::Error("Depends on an Error".to_string()), Some(expr)));
                                }
                                else {
                                    database_insert(*cell, CellRef::new(runner.run(&variables),Some(expr)));
                                }
                            }
                            None => ()
                        }
                    }
                }
                Err(topo_error) => {
                    if let CycleDetected(cell_self_ref) = topo_error{
                        for cell in cell_self_ref.iter(){
                            let cell_value = database_get_value(cell);
                            database_insert(*cell, CellRef::new(CellValue::Error(
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
            let cell_ref = database_get_value(&cell_position);
            if let Some(_) = cell_ref.dependency{
                if let CellValue::Error(e) = cell_ref.cell_value{
                    return Reply::Error(e);
                }
            }
            return Reply::Value(args_list[0].to_string(), cell_ref.cell_value);
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

fn parse_variables(range: &str) -> Option<Vec<(u32, u32)>> {
    let parts: Vec<&str> = range.split('_').collect();
    let mut result = Vec::new();

    match parts.len() {
        1 => {
            if let Some(coords) = split_cell_id(parts[0]) {
                result.push(coords);
            }
            else { return None }
        },
        2 => {
            if let (Some(start), Some(end)) = (split_cell_id(parts[0]), split_cell_id(parts[1])) {
                for row in start.0..=end.0 {
                    for col in start.1..=end.1 {
                        result.push((row, col));
                    }
                }
            }
        },
        _ => { return None }
    }
    Some(result)
}

fn parse_cell_range(cell_id: &str) -> Option<CellArgument> {
    let parts: Vec<&str> = cell_id.split('_').collect();
    match parts.len() {
        1 => {
            split_cell_id(parts[0]).and_then(|index| {
                if let CellValue::Error(_)|CellValue::None = database_get_value(&index).cell_value{
                    return None
                }
                Some(CellArgument::Value(database_get_value(&index).cell_value))
            })
        },
        2 => {
            let start = split_cell_id(parts[0])?;
            let end = split_cell_id(parts[1])?;
            if start.0 == end.0 {
                Some(CellArgument::Vector((start.1..=end.1).map(|y| database_get_value(&(start.0, y)).cell_value).collect()))
            } else if start.1 == end.1 {
                Some(CellArgument::Vector((start.0..=end.0).map(|x| database_get_value(&(x, start.1)).cell_value).collect()))
            } else {
                let mut matrix = Vec::new();
                for x in start.0..=end.0 {
                    let mut row = Vec::new();
                    for y in start.1..=end.1 {
                        row.push(database_get_value(&(x, y)).cell_value);
                    }
                    matrix.push(row);
                }
                Some(CellArgument::Matrix(matrix))
            }
        },
        _ => { None }
    }
}