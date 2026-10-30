use crate::collections::{Ptr, flatforest::FlatForest};
use bitflags::bitflags;
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

pub struct RichTag;

bitflags! {
    #[derive(Copy,Clone,PartialEq,Debug)]
    pub struct InlineStyling: u32 {
        const UNSTYLED= 0;
        const BOLD = 1<<0;
        const ITALICS = 1<<1;
        const UNDERLINE = 1<<2;
        const SPOILERS = 1<<3;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RichNodeKind {
    Document,
    Newline,
    Text(TextState),
}
impl RichNodeKind {
    pub fn as_text(&self) -> Option<&TextState> {
        if let Self::Text(state) = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn as_text_mut(&mut self) -> Option<&mut TextState> {
        if let Self::Text(state) = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn is_text_kind(&self) -> bool {
        matches!(self, Self::Text(_))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextState {
    pub content: String,
    pub styling: InlineStyling,
}

#[derive(Copy, Clone, PartialEq)]
pub struct RichSelection {
    pub start_node: Ptr<RichTag>,
    pub end_node: Ptr<RichTag>,
    pub start_offset: RichOffset,
    pub end_offset: RichOffset,
}

pub enum RichCharacterRangeKind {
    MonoCharacter,
    MultiCharacter,
}

pub enum RichSelectionKind {
    MonoNode(RichCharacterRangeKind),
    MultiNode,
}

impl RichSelection {
    pub fn new() -> Self {
        Self {
            start_node: Ptr::null(),
            end_node: Ptr::null(),
            start_offset: RichOffset::default(),
            end_offset: RichOffset::default(),
        }
    }

    pub fn kind(&self) -> RichSelectionKind {
        if self.start_node == self.end_node {
            let character_range = if self.start_offset == self.end_offset {
                RichCharacterRangeKind::MonoCharacter
            } else {
                RichCharacterRangeKind::MultiCharacter
            };
            RichSelectionKind::MonoNode(character_range)
        } else {
            RichSelectionKind::MultiNode
        }
    }
}

impl Default for RichSelection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct RichRange {
    pub start: RichOffset,
    pub end: RichOffset,
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct RichOffset {
    pub utf8: usize,
    pub utf16: usize,
}
impl RichOffset {
    pub fn from_utf16_offset(source_text: &str, utf16_offset: usize) -> Self {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for c in source_text.chars() {
            if utf16 >= utf16_offset {
                break;
            }
            utf8 += c.len_utf8();
            utf16 += c.len_utf16();
        }
        Self { utf8, utf16 }
    }

    pub fn from_utf8_offset(source_text: &str, utf8_offset: usize) -> Self {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for c in source_text.chars() {
            if utf8 >= utf8_offset {
                break;
            }
            utf8 += c.len_utf8();
            utf16 += c.len_utf16();
        }
        Self { utf8, utf16 }
    }

    pub fn update_from_utf8(&mut self, source_text: &str, utf8_offset: usize) {
        *self = Self::from_utf8_offset(source_text, utf8_offset);
    }

    pub fn update_from_utf16(&mut self, source_text: &str, utf16_offset: usize) {
        *self = Self::from_utf16_offset(source_text, utf16_offset);
    }
    pub fn zero() -> Self {
        Self { utf8: 0, utf16: 0 }
    }
    pub fn offset_by(&mut self, c: char) {
        self.utf8 += c.len_utf8();
        self.utf16 += c.len_utf16();
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TextSelectionKind {
    Caret(RichOffset),
    Range(RichRange),
    Unselected,
}
impl TextSelectionKind {
    pub fn as_caret(&self) -> Option<RichOffset> {
        if let &Self::Caret(caret_position) = self {
            Some(caret_position)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ExtraTextState {
    pub selection: TextSelectionKind,
}

#[derive(Clone, PartialEq)]
pub enum RichAuxData {
    Text(ExtraTextState),
    NOTHING,
}
impl RichAuxData {
    pub fn as_text(&self) -> Option<&ExtraTextState> {
        if let Self::Text(state) = self {
            Some(state)
        } else {
            None
        }
    }

    pub fn as_text_mut(&mut self) -> Option<&mut ExtraTextState> {
        if let Self::Text(state) = self {
            Some(state)
        } else {
            None
        }
    }
}

pub type RichForest = FlatForest<RichTag, RichNodeKind>;
pub type RichContentTable = HashMap<Ptr<RichTag>, RichAuxData>;

/*
 * BELOW IS A PROOF OF CONCEPT AST I'LL PROBABLY NOT USE
 */

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
