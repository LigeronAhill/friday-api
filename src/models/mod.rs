mod currency;
pub use currency::*;

pub struct AppState {
    pub storage: crate::storage::Storage,
}
impl AppState {
    pub fn new(storage: crate::storage::Storage) -> Self {
        Self { storage }
    }
}
