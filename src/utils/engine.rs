use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::CommandRunner;
use rsheet_lib::replies::Reply;
use crate::utils::database::{CellRef, database_get_value, database_insert, get_cell_argument, parse_to_indices, pos_to_cell_id, split_cell_id};
use crate::utils::dependency_manager::{find_topology_sort_of_weakly_component, update_incoming_edges};
use crate::utils::dependency_manager::TopoError::CycleDetected;

pub struct Transaction {
    request: Vec<String>,
    responder: Sender<Option<Reply>>,
}

impl Transaction{
    pub fn new(request: Vec<String>, responder: Sender<Option<Reply>>) -> Self{
        Transaction {
            request,
            responder
        }
    }
}

pub fn execute_transactions(rx: mpsc::Receiver<Transaction>){
    for transaction in rx {
        let request = transaction.request;
        if let Some(cell_position) = split_cell_id(&request[0]){
            let runner = CommandRunner::new(&request[1]);
            let variables = runner.find_variables();
            let mut var_list = Vec::new();
            for var in variables.iter() {
                match &mut parse_to_indices(var) {
                    Some(result) => {
                        var_list.append(result)
                    }
                    None => {
                        transaction.responder.send(Some(Reply::Error(format!("Error: Invalid Key Provided: {}", var)))).unwrap();
                        break
                    },
                }
            }
            if var_list.len() == 0{
                let cell_value = runner.run(&HashMap::new());
                database_insert(cell_position, CellRef::new(cell_value, None));
            }
            else {
                database_insert(cell_position, CellRef::new(CellValue::None, Some(String::from(&request[1]))));
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
                                    match get_cell_argument(id) {
                                        Some(cell_arg) => {variables.insert(id.clone(), cell_arg);}
                                        None => ()
                                    }
                                }
                                database_insert(*cell, CellRef::new(runner.run(&variables),Some(expr)));
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
        else { transaction.responder.send(Some(Reply::Error(format!("Error: Invalid Key Provided: {}", request[0])))).unwrap()  }
        transaction.responder.send(None).unwrap()
    }
}