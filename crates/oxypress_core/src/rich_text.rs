use crate::collections::{flatforest::FlatForest, ringbuffer::RingBuffer, Ptr};
use bitflags::{bitflags, Flags};
use std::{cell::RefCell, collections::HashMap, panic, rc::Rc};

#[derive(Debug)]
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
    Line,
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
impl TextState {
    pub fn get_end_offset(&self) -> RichOffset {
        RichOffset::from_utf8_offset(&self.content, self.content.len())
    }

    pub fn get_full_range(&self) -> RichRange {
        RichRange {
            start: RichOffset::zero(),
            end: self.get_end_offset(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RichSelectionDebugInfo {
    pub instantiation_point: &'static panic::Location<'static>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RichSelection {
    pub start_node: Ptr<RichTag>,
    pub end_node: Ptr<RichTag>,
    pub start_offset: RichOffset,
    pub end_offset: RichOffset,
    pub debug_info: Option<RichSelectionDebugInfo>,
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
    #[track_caller]
    pub fn multinode(
        start_node: Ptr<RichTag>,
        end_node: Ptr<RichTag>,
        start_offset: RichOffset,
        end_offset: RichOffset,
    ) -> Self {
        let location = panic::Location::caller();
        Self {
            start_node,
            end_node,
            start_offset,
            end_offset,
            debug_info: Some(RichSelectionDebugInfo {
                instantiation_point: location,
            }),
        }
    }

    #[track_caller]
    pub fn mononode(node: Ptr<RichTag>, range: RichRange) -> Self {
        let location = panic::Location::caller();
        Self {
            start_node: node,
            end_node: node,
            start_offset: range.start,
            end_offset: range.end,
            debug_info: Some(RichSelectionDebugInfo {
                instantiation_point: location,
            }),
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

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct RichRange {
    pub start: RichOffset,
    pub end: RichOffset,
}
impl RichRange {
    pub fn from_caret(caret: RichOffset) -> Self {
        Self {
            start: caret,
            end: caret,
        }
    }
    pub fn from_str_unbounded(content: &str) -> Self {
        Self {
            start: RichOffset::zero(),
            end: RichOffset::from_utf8_offset(content, content.len()),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug, Hash, Eq)]
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

#[derive(Copy, Clone, PartialEq, Debug)]
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
    pub fn as_range(&self) -> Option<RichRange> {
        if let &Self::Range(range) = self {
            Some(range)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ExtraTextState {
    pub selection: TextSelectionKind,
}
impl ExtraTextState {
    pub fn set_range_selection_with_offsets(&mut self, start: RichOffset, end: RichOffset) {
        self.selection = TextSelectionKind::Range(RichRange { start, end });
    }
    pub fn set_range_selection(&mut self, range: RichRange) {
        self.selection = TextSelectionKind::Range(range);
    }
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

#[derive(Clone, PartialEq)]
pub struct TextAreaState {
    pub focused_tool: Option<usize>,
    pub tools: Vec<ToolState>,
    pub forest: Rc<RefCell<RichForest>>,
    pub forest_content: Rc<RefCell<RichContentTable>>,
    pub effects_buffer: Rc<RefCell<Vec<Effect>>>,
    pub document: Ptr<RichTag>,
    pub focused_node: Ptr<RichTag>,
    pub update_counter: usize,
    pub document_selection: Option<RichSelection>,
    pub deferred_tasks: RingBuffer<[DeferredDocumentTask; 16]>,
}

impl TextAreaState {
    pub fn new() -> Self {
        Self {
            focused_tool: None,
            tools: vec![
                ToolState::new()
                    .with_name("bold-tool")
                    .with_css_icon("gg-format-bold")
                    .with_tool_kind(ToolKind::Bold),
                ToolState::new()
                    .with_name("italics-tool")
                    .with_css_icon("gg-format-italic")
                    .with_tool_kind(ToolKind::Italics),
                ToolState::new()
                    .with_name("underline-tool")
                    .with_css_icon("gg-format-underline")
                    .with_tool_kind(ToolKind::Underline),
                ToolState::new()
                    .with_name("list-tool")
                    .with_css_icon("gg-layout-list"),
            ],
            forest: Rc::new(RefCell::new(RichForest::new())),
            forest_content: Rc::new(RefCell::new(RichContentTable::new())),
            document: Ptr::null(),
            focused_node: Ptr::null(),
            update_counter: 0,
            document_selection: None,
            effects_buffer: Rc::new(RefCell::new(Vec::new())),
            deferred_tasks: RingBuffer::with_backing(std::array::from_fn(|_| {
                DeferredDocumentTask::with_kind(DeferredDocumentTaskKind::NOP)
            })),
        }
    }
    pub fn with_sample_document(mut self) -> Self {
        /* Hardcoded AST:
         * --------------
         *  Document
         *  ..Line
         *  ....Text('HELLO WORLD'|Bold)
         * */
        let sample_text = "Hello World";
        let helpers = self.document_helpers();
        let document = helpers.create_document_node();
        let line_container = helpers.create_line_node();
        let text_node = helpers.create_text_node(
            TextState {
                content: String::from(sample_text),
                styling: InlineStyling::BOLD,
            },
            ExtraTextState {
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset(sample_text, 5)),
            },
        );
        helpers.forest_mut().add_child(document, line_container);
        helpers.forest_mut().add_child(line_container, text_node);
        self.document = document;
        self
    }

    #[track_caller]
    pub fn enqueue_task(
        &mut self,
        incoming_task: DeferredDocumentTask,
    ) -> Result<(), crate::collections::ringbuffer::RingError> {
        self.deferred_tasks.enqueue(incoming_task)
    }
    pub fn flush_tasks(&mut self) {
        self.deferred_tasks.clear();
    }

    pub fn set_focused_tool(mut self, tool_idx: Option<usize>) -> Self {
        self.focused_tool = tool_idx;
        self
    }

    pub fn trigger_update(&mut self) {
        self.update_counter += 1;
    }

    pub fn focused_node(&self) -> &Ptr<RichTag> {
        &self.focused_node
    }

    pub fn focused_node_mut(&mut self) -> &mut Ptr<RichTag> {
        &mut self.focused_node
    }

    pub fn flush_overlays(&self) {
        let document = self.document;
        {
            let forest = &mut *self.forest.borrow_mut();
            let forest_content = &mut *self.forest_content.borrow_mut();
            forest.traverse_all_mut(document, |forest, node| {
                if let RichNodeKind::Text(_text_state) = forest.get_data(node) {
                    let text_node_state = forest_content
                        .get_mut(&node)
                        .and_then(RichAuxData::as_text_mut)
                        .expect("Failed to fetch text-node content");
                    text_node_state.selection = TextSelectionKind::Unselected;
                }
                false
            });
        }
    }

    pub fn clear_document_selection(&mut self) {
        *self.document_selection_mut() = None;
    }

    pub fn execute_tool_by_idx<'a>(
        &mut self,
        tool_idx: usize,
        effects_buffer: &'a mut Vec<Effect>,
    ) -> EffectRunner<'a> {
        let mut local_tool_state = self.tools[tool_idx].clone();
        local_tool_state.execute(self, tool_idx, effects_buffer);
        //update state with potentially mutated values
        self.tools[tool_idx] = local_tool_state;
        EffectRunner::with_buffer(effects_buffer)
    }

    pub fn document_helpers(&self) -> RichDocumentHelpers {
        RichDocumentHelpers {
            document: self.document,
            forest: self.forest.clone(),
            forest_content: self.forest_content.clone(),
        }
    }

    pub fn document_selection(&self) -> Option<RichSelection> {
        self.document_selection
    }

    #[track_caller]
    pub fn document_selection_mut(&mut self) -> &mut Option<RichSelection> {
        &mut self.document_selection
    }

    pub fn to_notation(&self) -> String {
        let mut notation_buffer = String::new();
        let helper = self.document_helpers();
        Self::to_notation_helper(&helper, self.document, &mut notation_buffer);
        notation_buffer
    }
    fn to_notation_helper(helper: &RichDocumentHelpers, root: Ptr<RichTag>, output: &mut String) {
        if root.is_null() {
            return;
        }
        let node_kind = helper.try_node_data(root, |kind| kind.clone());
        let child_len = helper.node_info(root, |info| info.children.len());
        match node_kind {
            RichNodeKind::Document => {
                for child_idx in 0..child_len {
                    let child_ptr = helper.node_info(root, |info| info.children[child_idx]);
                    Self::to_notation_helper(helper, child_ptr, output);
                }
            }
            RichNodeKind::Line => {
                for child_idx in 0..child_len {
                    let child_ptr = helper.node_info(root, |info| info.children[child_idx]);
                    Self::to_notation_helper(helper, child_ptr, output);
                }
                output.push('\n');
            }
            RichNodeKind::Text(text_state) => {
                let styling = text_state.styling;
                let mut content = text_state.content;
                let extra_text_state = helper
                    .try_get_extra_text_state(root, |extra_text_state| extra_text_state.clone())
                    .expect("Failed to get extra state");

                if content.is_empty() {
                    let mut draw_sentinel_text_nodes = || {
                        content.push('.');
                    };
                    draw_sentinel_text_nodes();
                }

                match extra_text_state.selection {
                    TextSelectionKind::Caret(pos) => {
                        let lower = &content[0..pos.utf8];
                        let upper = &content[pos.utf8..];
                        output.push_str(&apply_notation_styles(styling, lower.into()));
                        output.push('|');
                        output.push_str(&apply_notation_styles(styling, upper.into()));
                    }
                    TextSelectionKind::Unselected => {
                        output.push_str(&apply_notation_styles(styling, content));
                    }
                    TextSelectionKind::Range(range) => {
                        let head = &content[..range.start.utf8];
                        let selected = &content[range.start.utf8..range.end.utf8];
                        let tail = &content[range.end.utf8..];
                        output.push_str(&apply_notation_styles(styling, head.into()));
                        output.push('[');
                        output.push_str(&apply_notation_styles(styling, selected.into()));
                        output.push(']');
                        output.push_str(&apply_notation_styles(styling, tail.into()));
                    }
                }
            }
        }
    }

    pub fn tree_dump(&self) -> String {
        let mut notation_buffer = String::new();
        let helper = self.document_helpers();
        Self::tree_dump_helper(&helper, self.document, &mut notation_buffer);
        notation_buffer
    }
    fn tree_dump_helper(helper: &RichDocumentHelpers, root: Ptr<RichTag>, output: &mut String) {
        if root.is_null() {
            return;
        }
        let node_kind = helper.try_node_data(root, |kind| kind.clone());
        let child_len = helper.node_info(root, |info| info.children.len());
        match node_kind {
            RichNodeKind::Document => {
                output.push_str("<doc>");
                for child_idx in 0..child_len {
                    let child_ptr = helper.node_info(root, |info| info.children[child_idx]);
                    Self::tree_dump_helper(helper, child_ptr, output);
                }
                output.push_str("</doc>");
            }
            RichNodeKind::Line => {
                output.push_str("<line>");
                for child_idx in 0..child_len {
                    let child_ptr = helper.node_info(root, |info| info.children[child_idx]);
                    Self::tree_dump_helper(helper, child_ptr, output);
                }
                output.push_str("</line>");
            }
            RichNodeKind::Text(text_state) => {
                let extra_text_state = helper
                    .try_get_extra_text_state(root, |extra_text_state| extra_text_state.clone())
                    .expect("Failed to get extra state");
                output.push_str("<text>");
                match extra_text_state.selection {
                    TextSelectionKind::Caret(pos) => {
                        let lower = &text_state.content[0..pos.utf8];
                        let upper = &text_state.content[pos.utf8..];
                        output.push_str(lower);
                        output.push('|');
                        output.push_str(upper);
                    }
                    TextSelectionKind::Unselected => {
                        output.push_str(text_state.content.as_str());
                    }
                    TextSelectionKind::Range(range) => {
                        let head = &text_state.content[..range.start.utf8];
                        let selected = &text_state.content[range.start.utf8..range.end.utf8];
                        let tail = &text_state.content[range.end.utf8..];
                        output.push_str(head);
                        output.push('[');
                        output.push_str(selected);
                        output.push(']');
                        output.push_str(tail);
                    }
                }
                output.push_str("</text>");
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct ClippedNodeInfo {
    node: Ptr<RichTag>,
    range: RichRange,
}
#[derive(Copy, Clone, PartialEq)]
pub enum ToolKind {
    Bold,
    Italics,
    Underline,
    Unimplemented,
}
impl From<ToolKind> for InlineStyling {
    fn from(value: ToolKind) -> Self {
        match value {
            ToolKind::Underline => Self::UNDERLINE,
            ToolKind::Bold => Self::BOLD,
            ToolKind::Italics => Self::ITALICS,
            _ => panic!("Unhandled conversion failed to convert ToolKind"),
        }
    }
}
#[derive(PartialEq, Clone)]
pub struct ToolState {
    pub name: &'static str,
    pub css_icon: &'static str,
    pub is_selected: bool,
    pub kind: ToolKind,
}
impl ToolState {
    pub fn new() -> Self {
        Self {
            name: "",
            css_icon: "",
            is_selected: false,
            kind: ToolKind::Unimplemented,
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn with_css_icon(mut self, css_icon: &'static str) -> Self {
        self.css_icon = css_icon;
        self
    }

    pub fn with_tool_kind(mut self, kind: ToolKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn execute(
        &mut self,
        ta_state: &mut TextAreaState,
        tool_idx: usize,
        effect_buffer: &mut Vec<Effect>,
    ) -> Option<()> {
        let helpers = ta_state.document_helpers();
        match self.kind {
            ToolKind::Bold | ToolKind::Italics | ToolKind::Underline => {
                let selection = ta_state.document_selection()?;
                let focused_node = ta_state.focused_node;
                effect_buffer.push(Effect::ConsoleLog(format!(
                    "selection = {:?},focused node ={:?}",
                    selection, focused_node
                )));
                match selection.kind() {
                    RichSelectionKind::MonoNode(RichCharacterRangeKind::MultiCharacter) => {
                        let clipped_node = self.apply_tool_styling_mononode(
                            ta_state,
                            effect_buffer,
                            selection,
                            styling_ops::toggle_style,
                        );
                        *ta_state.document_selection_mut() = Some(RichSelection::mononode(
                            clipped_node.node,
                            clipped_node.range,
                        ));
                        helpers
                            .try_get_extra_text_state_mut(clipped_node.node, |extra_text_state| {
                                extra_text_state.selection =
                                    TextSelectionKind::Range(clipped_node.range)
                            })
                            .expect("Failed to get extra text state");

                        self.toggle_selection();
                    }
                    RichSelectionKind::MonoNode(RichCharacterRangeKind::MonoCharacter) => {
                        let focused_styling = helpers
                            .try_get_text_state(focused_node, |text_state| text_state.styling)
                            .expect("Failed to get text styling");
                        ta_state
                            .enqueue_task(DeferredDocumentTask::with_kind(
                                DeferredDocumentTaskKind::InsertNodeAt {
                                    node: selection.start_node,
                                    offset: selection.start_offset,
                                    src_style: focused_styling,
                                    style_clicked: InlineStyling::from(self.kind),
                                },
                            ))
                            .expect("Failed to enqueue task");
                        self.toggle_selection();
                        effect_buffer.push(Effect::HiddenInput(HiddenInputOp::Focus {
                            prevent_scroll: true,
                            focus_visible: true,
                        }));
                    }
                    RichSelectionKind::MultiNode => {
                        let common_styles =
                            compute_common_stylings_in_selection(ta_state, &selection);

                        let keep_non_empty_text_nodes =
                            |(_, kind): &(Ptr<RichTag>, &mut RichNodeKind)| {
                                kind.is_text_kind() && !kind.as_text().unwrap().content.is_empty()
                            };

                        // NOTE: SOMETIMES `compute_document_selection(..)` have `start` and `end`
                        // ranges that select NO text in that range. `apply_tool_styling_mononode(..)`
                        // doesn't expect ranges with zero text actually being selected
                        let keep_non_empty_ranges =
                            |(_ptr, range, data): &(Ptr<RichTag>, RichRange, &mut RichNodeKind)| {
                                let text_state = data.as_text().expect("Failed to get text_state");
                                let selected_content =
                                    &text_state.content[range.start.utf8..range.end.utf8];
                                !selected_content.is_empty()
                            };

                        let selection_list = ta_state
                            .forest
                            .borrow_mut()
                            .iter_inclusive_mut(
                                ta_state.document,
                                selection.start_node,
                                selection.end_node,
                            )
                            .filter(keep_non_empty_text_nodes)
                            .map(|(ptr, data)| {
                                let range = helpers
                                    .forest_content()
                                    .get(&ptr)
                                    .and_then(|aux_data| aux_data.as_text())
                                    .and_then(|extra_text_state| {
                                        effect_buffer.push(Effect::ConsoleLog(format!(
                                            "Current selection = {:?}",
                                            extra_text_state.selection
                                        )));
                                        extra_text_state.selection.as_range()
                                    })
                                    .unwrap_or_else(|| {
                                        panic!("Failed to get selection range at ptr={}", ptr)
                                    });
                                (ptr, range, data)
                            })
                            .filter(keep_non_empty_ranges)
                            .map(|(ptr, range, _)| (ptr, range))
                            .collect::<Vec<_>>();
                        let clipped_list = selection_list
                            .iter()
                            .copied()
                            .map(|(ptr, range)| {
                                let mononode_selection = RichSelection::mononode(ptr, range);
                                self.apply_tool_styling_mononode(
                                    ta_state,
                                    effect_buffer,
                                    mononode_selection,
                                    |tool_state, current_styling| {
                                        let tool_style = InlineStyling::from(tool_state.kind);
                                        if common_styles.contains(tool_style) {
                                            current_styling ^ tool_style
                                        } else {
                                            current_styling | tool_style
                                        }
                                    },
                                )
                            })
                            .collect::<Vec<_>>();
                        for clipped_node in clipped_list.iter() {
                            helpers
                                .forest_content_mut()
                                .get_mut(&clipped_node.node)
                                .and_then(RichAuxData::as_text_mut)
                                .expect("Failed to get aux text data")
                                .selection = TextSelectionKind::Range(clipped_node.range);
                        }
                        let start_clipped = clipped_list
                            .first()
                            .copied()
                            .expect("Failed to get first clipped node");
                        let end_clipped = clipped_list
                            .last()
                            .copied()
                            .expect("Failed to get last clipped node");
                        let new_selection = RichSelection::multinode(
                            start_clipped.node,
                            end_clipped.node,
                            RichOffset::zero(),
                            end_clipped.range.end,
                        );
                        *ta_state.document_selection_mut() = Some(new_selection);

                        update_toolbar_styles(
                            ta_state,
                            compute_common_stylings_in_selection(ta_state, &new_selection),
                        );

                        let mut sync_global_state_with_local_state = || {
                            *self = ta_state.tools[tool_idx].clone();
                        };
                        sync_global_state_with_local_state();
                    }
                }
            }
            ToolKind::Unimplemented => {}
        }
        None
    }
    pub fn toggle_selection(&mut self) {
        self.is_selected = !self.is_selected;
    }

    fn apply_tool_styling_mononode<'a, F>(
        &'a self,
        ta_state: &mut TextAreaState,
        effect_buffer: &mut Vec<Effect>,
        selection: RichSelection,
        styling_op: F,
    ) -> ClippedNodeInfo
    where
        F: Fn(&'a Self, InlineStyling) -> InlineStyling,
    {
        let helpers = ta_state.document_helpers();

        let selected_node = selection.start_node;
        let selected_node_parent = *ta_state.forest.borrow().get_parent(selected_node);
        let selected_node_state = ta_state
            .forest
            .borrow()
            .get_data(selected_node)
            .as_text()
            .cloned()
            .expect("Failed to get text state");
        let selected_content = &selected_node_state.content;

        let RangeSplitContents {
            head,
            selected: middle,
            tail,
        } = RichDocumentHelpers::split_content_mono(selected_content, selection)
            .expect("Failed to split mono")
            .as_range_split()
            .expect("Failed to get range split");

        let old_styling = selected_node_state.styling;
        let new_styling = styling_op(self, old_styling); //

        helpers
            .try_get_text_state_mut(selected_node, |text_state| {
                text_state.content.clear();
                text_state.content.push_str(head);
                text_state.styling = old_styling;
            })
            .expect("Failed to get text_state");
        helpers
            .try_get_extra_text_state_mut(selected_node, |extra_state| {
                extra_state.selection = TextSelectionKind::Unselected;
            })
            .expect("Failed to get extra text state");

        let middle_node = helpers.create_text_node(
            TextState {
                content: String::from(middle),
                styling: new_styling,
            },
            ExtraTextState {
                selection: TextSelectionKind::Unselected,
            },
        );

        ta_state.forest.borrow_mut().add_child_at(
            selected_node_parent,
            middle_node,
            selected_node,
            1,
        );
        let tail_node = helpers.create_text_node(
            TextState {
                content: String::from(tail),
                styling: old_styling,
            },
            ExtraTextState {
                selection: TextSelectionKind::Unselected,
            },
        );

        ta_state
            .forest
            .borrow_mut()
            .add_child_at(selected_node_parent, tail_node, middle_node, 1);

        // some of the text nodes created can be
        // completely empty and needs to be pruned
        // from the document
        let clean_up_list = [
            (head, selected_node),
            (middle, middle_node),
            (tail, tail_node),
        ];

        clean_up_list
            .iter()
            .rev()
            .copied()
            .for_each(|(node_content, node_ptr)| {
                if node_content.is_empty() {
                    helpers.delete_node(node_ptr);
                    if node_ptr == middle_node {
                        effect_buffer.push(Effect::ConsoleLog(
                            "WARNING MIDDLE NODE IS BEING DELETED!!!".into(),
                        ));
                        effect_buffer.push(Effect::ConsoleLog(format!(
                            "selected content = \'{selected_content}\'"
                        )));
                        effect_buffer.push(Effect::ConsoleLog(format!(
                            "head = \'{head}\' middle = \'{middle}\' tail =\'{tail}\'"
                        )));
                    }
                }
            });

        /*let children = ta_state
            .forest
            .borrow()
            .get_children(selected_node_parent)
            .clone();
        console_log(format!("parent child len = {}", children.len()));*/
        ClippedNodeInfo {
            node: middle_node,
            range: RichRange::from_str_unbounded(middle),
        }
    }
}
pub fn update_toolbar_styles(ta_state: &mut TextAreaState, styles: InlineStyling) {
    const BY_UNSTYLED_FLAG: usize = 1;
    InlineStyling::FLAGS
        .iter()
        .skip(BY_UNSTYLED_FLAG)
        .enumerate()
        .for_each(|(flag_idx, flag)| {
            ta_state.tools[flag_idx].is_selected = styles.contains(*flag.value());
        });
}
pub fn compute_common_stylings_in_selection(
    ta_state: &TextAreaState,
    selection: &RichSelection,
) -> InlineStyling {
    fn filter_out_non_text_nodes_they_dont_matter(
        (_ptr, data_kind): (Ptr<RichTag>, &RichNodeKind),
    ) -> Option<&TextState> {
        data_kind.as_text()
    }

    fn filter_out_empty_nodes_the_styling_shouldnt_contribute_to_the_accumulator(
        text_state: &&TextState,
    ) -> bool {
        !text_state.content.is_empty()
    }

    ta_state
        .forest
        .borrow()
        .iter_inclusive(ta_state.document, selection.start_node, selection.end_node)
        .filter_map(filter_out_non_text_nodes_they_dont_matter)
        .filter(filter_out_empty_nodes_the_styling_shouldnt_contribute_to_the_accumulator)
        .fold(InlineStyling::all(), |acc, e| acc & e.styling)
}

pub mod styling_ops {
    use super::ToolState;
    use crate::rich_text::InlineStyling;

    pub fn toggle_style(
        selected_tool_state: &ToolState,
        old_styling: InlineStyling,
    ) -> InlineStyling {
        old_styling ^ InlineStyling::from(selected_tool_state.kind)
    }
}

impl Default for ToolState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct DeferredDocumentTask {
    pub created_at: panic::Location<'static>,
    pub task_kind: DeferredDocumentTaskKind,
}
impl DeferredDocumentTask {
    #[track_caller]
    pub fn with_kind(kind: DeferredDocumentTaskKind) -> Self {
        let location = panic::Location::caller();
        Self {
            created_at: *location,
            task_kind: kind,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum DeferredDocumentTaskKind {
    InsertNodeAt {
        node: Ptr<RichTag>,
        offset: RichOffset,
        src_style: InlineStyling,
        style_clicked: InlineStyling,
    },
    NOP,
}

#[derive(Clone, PartialEq, Debug)]
pub enum HiddenInputOp {
    SetSelectionRange {
        start: u32,
        end: u32,
    },
    SetValue(String),
    Focus {
        prevent_scroll: bool,
        focus_visible: bool,
    },
}

#[derive(Clone, PartialEq, Debug)]
pub enum Effect {
    HiddenInput(HiddenInputOp),
    ConsoleLog(String),
}
pub struct EffectRunner<'a> {
    effect_buffer: &'a mut Vec<Effect>,
}
impl<'a> EffectRunner<'a> {
    pub fn with_buffer(buffer: &'a mut Vec<Effect>) -> Self {
        Self {
            effect_buffer: buffer,
        }
    }
    pub fn apply<F>(&mut self, mut apply_effect: F)
    where
        F: FnMut(&Effect),
    {
        for effect in self.effect_buffer.iter() {
            apply_effect(effect);
        }
        self.effect_buffer.clear();
    }
}

pub struct NodeInfo<'a> {
    pub parent: Ptr<RichTag>,
    pub children: &'a Vec<Ptr<RichTag>>,
}
pub struct NodeInfoMut<'a> {
    pub parent: Ptr<RichTag>,
    pub children: &'a mut Vec<Ptr<RichTag>>,
}

pub struct RichDocumentHelpers {
    pub document: Ptr<RichTag>,
    pub forest: Rc<RefCell<RichForest>>,
    pub forest_content: Rc<RefCell<RichContentTable>>,
}

#[derive(Copy, Clone)]
pub struct CaretSplitContents<'a> {
    pub left: &'a str,
    pub right: &'a str,
}

#[derive(Copy, Clone)]
pub struct RangeSplitContents<'a> {
    pub head: &'a str,
    pub selected: &'a str,
    pub tail: &'a str,
}
pub enum SplitKind<'a> {
    CaretSplit(CaretSplitContents<'a>),
    RangeSplit(RangeSplitContents<'a>),
}
impl<'a> SplitKind<'a> {
    pub fn as_caret_split(&self) -> Option<CaretSplitContents<'a>> {
        if let Self::CaretSplit(contents) = self {
            Some(*contents)
        } else {
            None
        }
    }

    pub fn as_range_split(&self) -> Option<RangeSplitContents<'a>> {
        if let &Self::RangeSplit(contents) = self {
            Some(contents)
        } else {
            None
        }
    }
}

impl RichDocumentHelpers {
    pub fn node_info<F, FOUT>(&self, ptr: Ptr<RichTag>, cb: F) -> FOUT
    where
        F: Fn(NodeInfo<'_>) -> FOUT,
    {
        let forest = &*self.forest();
        let children = forest.get_children(ptr);
        let parent = *forest.get_parent(ptr);
        let info = NodeInfo { parent, children };
        cb(info)
    }
    pub fn node_info_mut<F, FOUT>(&self, ptr: Ptr<RichTag>, cb: F) -> FOUT
    where
        F: Fn(NodeInfoMut<'_>) -> FOUT,
    {
        let forest = &mut *self.forest_mut();
        let parent = *forest.get_parent_mut(ptr);
        let children = forest.get_children_mut(ptr);
        let info = NodeInfoMut { parent, children };
        cb(info)
    }

    pub fn create_text_node(&self, state: TextState, extra: ExtraTextState) -> Ptr<RichTag> {
        let forest = &mut *self.forest.borrow_mut();
        let forest_content = &mut *self.forest_content.borrow_mut();
        let ptr = forest.allocate_node(RichNodeKind::Text(state));
        forest_content.insert(ptr, RichAuxData::Text(extra));
        ptr
    }
    pub fn create_line_node(&self) -> Ptr<RichTag> {
        self.forest_mut().allocate_node(RichNodeKind::Line)
    }
    pub fn create_document_node(&self) -> Ptr<RichTag> {
        self.forest_mut().allocate_node(RichNodeKind::Document)
    }

    pub fn split_content_mono(
        content: &str,
        selection: RichSelection,
    ) -> Result<SplitKind<'_>, &'static str> {
        match selection.kind() {
            RichSelectionKind::MultiNode => Err("Operation not intended for the multinode case"),
            RichSelectionKind::MonoNode(RichCharacterRangeKind::MultiCharacter) => {
                let head = &content[0..selection.start_offset.utf8];
                let selected = &content[selection.start_offset.utf8..selection.end_offset.utf8];
                let tail = &content[selection.end_offset.utf8..];
                Ok(SplitKind::RangeSplit(RangeSplitContents {
                    head,
                    selected,
                    tail,
                }))
            }
            RichSelectionKind::MonoNode(RichCharacterRangeKind::MonoCharacter) => {
                let left = &content[0..selection.start_offset.utf8];
                let right = &content[selection.start_offset.utf8..];
                Ok(SplitKind::CaretSplit(CaretSplitContents { left, right }))
            }
        }
    }

    pub fn delete_node(&self, root: Ptr<RichTag>) {
        let forest = &mut *self.forest.borrow_mut();
        let forest_content = &mut *self.forest_content.borrow_mut();
        forest.free_and_then(root, |ptr| {
            forest_content.remove(&ptr);
        });
    }

    pub fn try_node_data<F, FOUT>(&self, ptr: Ptr<RichTag>, cb: F) -> FOUT
    where
        F: Fn(&RichNodeKind) -> FOUT,
    {
        cb(self.forest().get_data(ptr))
    }
    pub fn try_node_data_mut<F, FOUT>(&self, ptr: Ptr<RichTag>, mut cb: F) -> FOUT
    where
        F: FnMut(&mut RichNodeKind) -> FOUT,
    {
        cb(self.forest_mut().get_data_mut(ptr))
    }

    pub fn get_aux_data<F, Out>(&self, ptr: Ptr<RichTag>, cb: F) -> Option<Out>
    where
        F: FnOnce(&RichAuxData) -> Out,
    {
        self.forest_content.borrow().get(&ptr).map(cb)
    }

    pub fn get_aux_data_mut<F, Out>(&self, ptr: Ptr<RichTag>, cb: F) -> Option<Out>
    where
        F: FnOnce(&mut RichAuxData) -> Out,
    {
        self.forest_content.borrow_mut().get_mut(&ptr).map(cb)
    }

    pub fn forest_content_mut<'a>(&'a self) -> std::cell::RefMut<'a, RichContentTable> {
        self.forest_content.borrow_mut()
    }

    pub fn forest_content<'a>(&'a self) -> std::cell::Ref<'a, RichContentTable> {
        self.forest_content.borrow()
    }

    pub fn forest(&self) -> std::cell::Ref<'_, RichForest> {
        self.forest.borrow()
    }
    pub fn forest_mut(&self) -> std::cell::RefMut<'_, RichForest> {
        self.forest.borrow_mut()
    }

    pub fn get_aux_text_state_mut<F, Out>(&self, ptr: Ptr<RichTag>, cb: F) -> Option<Out>
    where
        F: FnOnce(&mut ExtraTextState) -> Out,
    {
        self.get_aux_data_mut(ptr, |aux_data| aux_data.as_text_mut().map(cb))
            .flatten()
    }

    pub fn try_get_extra_text_state<F, FOUT>(
        &self,
        ptr: Ptr<RichTag>,
        cb: F,
    ) -> Result<FOUT, &'static str>
    where
        F: Fn(&ExtraTextState) -> FOUT,
    {
        let forest_content = self.forest_content();
        let data = forest_content
            .get(&ptr)
            .ok_or("Pointer not found in table")?;
        let extra_text_state = data.as_text().ok_or("Node isn't a text node")?;
        Ok(cb(extra_text_state))
    }
    pub fn try_get_extra_text_state_mut<F, FOUT>(
        &self,
        ptr: Ptr<RichTag>,
        cb: F,
    ) -> Result<FOUT, &'static str>
    where
        F: Fn(&mut ExtraTextState) -> FOUT,
    {
        let mut forest_content = self.forest_content_mut();
        let data = forest_content
            .get_mut(&ptr)
            .ok_or("Pointer not found in table")?;
        let extra_text_state = data.as_text_mut().ok_or("Node isn't a text node")?;
        Ok(cb(extra_text_state))
    }

    pub fn try_set_text_selection(
        &self,
        ptr: Ptr<RichTag>,
        selection: TextSelectionKind,
    ) -> Result<(), &'static str> {
        let mut forest_content = self.forest_content_mut();
        let data = forest_content
            .get_mut(&ptr)
            .ok_or("Pointer not found in table")?;
        let extra_text_state = data.as_text_mut().ok_or("Node isn't a text node")?;
        extra_text_state.selection = selection;
        Ok(())
    }

    pub fn try_get_text_selection(
        &self,
        ptr: Ptr<RichTag>,
    ) -> Result<TextSelectionKind, &'static str> {
        let forest_content = self.forest_content();
        let data = forest_content
            .get(&ptr)
            .ok_or("Pointer not found in table")?;
        let extra_text_state = data.as_text().ok_or("Node isn't a text node")?;
        Ok(extra_text_state.selection)
    }

    pub fn try_get_text_state<F, Out>(&self, ptr: Ptr<RichTag>, f: F) -> Result<Out, &'static str>
    where
        F: FnOnce(&TextState) -> Out,
    {
        let forest = self.forest.borrow();
        let node_data = forest.get_data(ptr);
        let text_state = node_data.as_text().ok_or("Not a text node")?;
        Ok(f(text_state))
    }

    pub fn try_get_text_state_mut<F, Out>(
        &self,
        ptr: Ptr<RichTag>,
        f: F,
    ) -> Result<Out, &'static str>
    where
        F: FnOnce(&mut TextState) -> Out,
    {
        let mut forest = self.forest.borrow_mut();
        let node_data = forest.get_data_mut(ptr);
        let text_state = node_data.as_text_mut().ok_or("Not a text node")?;
        Ok(f(text_state))
    }
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_notation_styles(styles: InlineStyling, base: String) -> String {
    let mut result = base;
    for flag in InlineStyling::FLAGS.iter() {
        let available_style = *flag.value();
        if styles.contains(available_style) {
            result = match available_style {
                InlineStyling::BOLD => format!("<b>{result}</b>"),
                InlineStyling::ITALICS => format!("<i>{result}</i>"),
                InlineStyling::SPOILERS => format!("<s>{result}</s>"),
                InlineStyling::UNDERLINE => format!("<u>{result}</u>"),
                InlineStyling::UNSTYLED => result,
                _ => panic!("Unknown style"),
            };
        }
    }
    result
}

#[derive(Copy, Clone, PartialEq)]
pub enum CaretDirection {
    Up,
    Down,
    Left,
    Right,
}
impl CaretDirection {
    pub fn to_sign_vertical(&self) -> f32 {
        match self {
            Self::Down => 1.0,
            Self::Up => -1.0,
            _ => panic!("Unexpected caret direction"),
        }
    }
    pub fn to_sign_horizontal(&self) -> isize {
        match self {
            Self::Left => -1,
            Self::Right => 1,
            _ => panic!("Unexpected caret direction"),
        }
    }
}
fn collect_leaf_nodes(
    state: &TextAreaState,
    focused_node: Ptr<RichTag>,
) -> (Vec<Ptr<RichTag>>, usize) {
    let helpers = state.document_helpers();
    enum CollectionState {
        Start,
        AfterFocused,
    }
    let mut leafs: Vec<Ptr<RichTag>> = vec![];
    let mut focused_index = !0;
    let mut iterator_state = CollectionState::Start;
    const MAX_NODES_AFTER_FOCUS: usize = 3;
    let mut after_focus_node_count = 0;
    helpers
        .forest_mut()
        .traverse_all_mut(state.document, |forest, root| {
            let root_kind = forest.get_data(root);
            if root_kind.is_text_kind() {
                leafs.push(root);
            }
            match iterator_state {
                CollectionState::Start => {
                    if root_kind.is_text_kind() && focused_node == root {
                        focused_index = leafs.len() - 1;
                        iterator_state = CollectionState::AfterFocused;
                    }
                }
                CollectionState::AfterFocused => {
                    after_focus_node_count += 1;
                    if after_focus_node_count >= MAX_NODES_AFTER_FOCUS {
                        return true;
                    }
                }
            }
            false
        });
    (leafs, focused_index)
}

pub mod document_operations {
    use super::*;

    pub fn move_caret_horizontal<'a>(
        state: &mut TextAreaState,
        effect_buffer: &'a mut Vec<Effect>,
        focused_node: Ptr<RichTag>,
        direction: CaretDirection,
    ) -> EffectRunner<'a> {
        let helpers = state.document_helpers();
        // TODO: The way im collecting leaf nodes isn't optimal
        // at all, this could be improved, im leaving this alone
        // because this works for right now
        let (leaf_list, focused_index) = collect_leaf_nodes(state, focused_node);
        effect_buffer.push(Effect::ConsoleLog(format!("leaf list = {:?}", leaf_list)));
        let adjacent_node_idx = focused_index as isize + direction.to_sign_horizontal();

        let is_invalid_adjacent = |adjacent_node_idx: isize| {
            adjacent_node_idx < 0 || adjacent_node_idx >= leaf_list.len() as isize
        };

        if is_invalid_adjacent(adjacent_node_idx) {
            return EffectRunner::with_buffer(effect_buffer);
        }

        let new_focused_node = leaf_list[adjacent_node_idx as usize];
        let new_focused_text_state = helpers
            .try_get_text_state(new_focused_node, |text_state| text_state.clone())
            .expect("Failed to get text state");

        let caret = match direction {
            CaretDirection::Right => RichOffset::zero(),
            CaretDirection::Left => RichOffset::from_utf8_offset(
                &new_focused_text_state.content,
                new_focused_text_state.content.len(),
            ),
            _ => panic!("Only Left and Right are expected in this routine"),
        };

        state.flush_overlays();
        helpers
            .try_get_extra_text_state_mut(new_focused_node, |extra_text_state| {
                extra_text_state.selection = TextSelectionKind::Caret(caret);
            })
            .expect("Failed to set text selection");
        effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetValue(
            new_focused_text_state.content.clone(),
        )));
        effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetSelectionRange {
            start: caret.utf8 as u32,
            end: caret.utf8 as u32,
        }));
        *state.focused_node_mut() = new_focused_node;
        *state.document_selection_mut() = Some(RichSelection::mononode(
            new_focused_node,
            RichRange::from_caret(caret),
        ));
        EffectRunner::with_buffer(effect_buffer)
    }
}
