use lazy_static::lazy_static;
use dashmap::DashMap;
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

pub fn database_get_value(key: &(u32, u32)) -> CellRef {
    DATABASE.get(key)
        .map(|entry| entry.clone())
        .unwrap_or(CellRef::new(CellValue::None,None))
}

// Global variables for static lifecycle
// Storing Spreadsheets Using `DashMap`
lazy_static! {
    pub static ref DATABASE: DashMap<(u32, u32), CellRef> = DashMap::new();
}
