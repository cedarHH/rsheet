use lazy_static::lazy_static;
use dashmap::DashMap;
use rsheet_lib::cells::{column_name_to_number, column_number_to_name};
use rsheet_lib::command_runner::{CellArgument, CellValue};

#[derive(Clone)]
pub struct CellRef {
    pub(crate) cell_value: CellValue,
    pub(crate) dependency: Option<String>,
}

impl CellRef {
    pub fn new(cell_value: CellValue, dependency: Option<String>) -> Self {
        CellRef {
            cell_value,
            dependency,
        }
    }
}

// Global variables for static lifecycle
// Storing Spreadsheets Using `DashMap`
lazy_static! {
    static ref DATABASE: DashMap<(u32, u32), CellRef> = DashMap::new();
}

pub fn database_get_value(key: &(u32, u32)) -> CellRef {
    DATABASE.get(key)
        .map(|entry| entry.clone())
        .unwrap_or(CellRef::new(CellValue::None,None))
}

pub fn database_insert(key:(u32, u32), value: CellRef) -> Option<CellRef>{
    DATABASE.insert(key, value)
}

pub fn split_cell_id(cell_id: &str) -> Option<(u32, u32)> {
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

pub fn pos_to_cell_id(position:&(u32,u32)) -> String {
    format!("{}{}",column_number_to_name(position.0),&position.1.to_string())
}

pub fn parse_to_indices(range: &str) -> Option<Vec<(u32, u32)>> {
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

pub fn get_cell_argument(cell_id: &str) -> Option<CellArgument> {
    let parts: Vec<&str> = cell_id.split('_').collect();
    match parts.len() {
        1 => {
            split_cell_id(parts[0]).and_then(|index| {
                if let CellValue::Error(_) = database_get_value(&index).cell_value{
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