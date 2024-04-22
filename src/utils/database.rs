use lazy_static::lazy_static;
use dashmap::DashMap;
use log::__private_api::Value;
use rsheet_lib::command_runner::CellValue;

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