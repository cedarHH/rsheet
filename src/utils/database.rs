use lazy_static::lazy_static;
use dashmap::DashMap;

// Global variables for static lifecycle
// Storing Spreadsheets Using `DashMap`
lazy_static! {
    pub static ref DATABASE: DashMap<i32, String> = DashMap::new();
}
