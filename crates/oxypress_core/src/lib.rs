use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod log;
pub mod prelude;
pub mod collections;
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

pub mod rich_text_core {
    use std::{cell::RefCell, ops::Deref, rc::Rc};
    pub enum Ast {
        Origin(Vec<Rc<Ast>>),
        Bold(Vec<Rc<Ast>>),
        Newline,
        Data(Rc<RefCell<String>>),
    }
    impl Ast {
        pub fn new_data<T: Into<String>>(data: T) -> Rc<Self> {
            Rc::new(Self::Data(Rc::new(RefCell::new(data.into()))))
        }
        pub fn new_newline() -> Rc<Self> {
            Rc::new(Self::Newline)
        }
        pub fn render_to(root: &Rc<Ast>, dest: &mut String) {
            match root.deref() {
                Self::Origin(children) => {
                    for node in children {
                        Self::render_to(node, dest);
                    }
                }
                Self::Newline => {
                    dest.push_str("<br/>");
                }
                Self::Data(data) => {
                    dest.push_str(data.borrow().as_ref());
                }
                Self::Bold(children) => {
                    dest.push_str("<strong>");
                    for node in children {
                        Self::render_to(node, dest);
                    }
                    dest.push_str("</string>");
                }
            }
        }
    }

    #[test]
    fn basic_building() {
        let mut ast = Ast::Origin(vec![
            Rc::new(Ast::Bold(vec![Ast::new_data("hello world")])),
            Rc::new(Ast::Newline),
        ]);
    }
}
