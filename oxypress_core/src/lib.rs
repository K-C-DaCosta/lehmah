use std::{default,fmt::Debug};

use serde::{Deserialize, Serialize};


pub mod log;
pub mod prelude;

#[test]
fn chrono_test(){

    print!("Chrono = {}",123);

}

/// # Serializeable reference to DOM node
/// - always relative to root of document (HTML node)
#[derive(Serialize,Deserialize,Debug)]
pub struct DomPointer {
    pub links:Vec<u32>,
}

impl DomPointer {
    pub fn new() ->Self{
        Self { links: vec![] }
    }
}

impl Default  for DomPointer{
    fn default() -> Self {
        Self::new()
    }
}