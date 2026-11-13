use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod collections;
pub mod log;
pub mod prelude;
pub mod rich_text;

/// # Serializeable reference to DOM node
/// - always relative to root of document (HTML node)
#[derive(Serialize, Deserialize, Debug)]
pub struct DomPointer {
    pub links: Vec<u32>,
}

impl DomPointer {
    pub fn new() -> Self {
        Self { links: vec![] }
    }
}

impl Default for DomPointer {
    fn default() -> Self {
        Self::new()
    }
}

