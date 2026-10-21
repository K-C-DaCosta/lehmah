use core::panic;
use oxypress_core::{
    collections::{flatforest::FlatForest, Ptr},
    rich_text::{
        RichCharacterRangeKind, RichContentTable, RichData, RichForest, RichKind, RichNode,
        RichOffset, RichRange, RichSelection, RichSelectionKind, RichTag, TextSelectionKind,
        TextState,
    },
};
use std::{
    cell::RefCell,
    char,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use web_sys::{
    js_sys::Array,
    wasm_bindgen::{self, prelude::Closure, JsCast, JsValue},
    window, HtmlElement, HtmlInputElement,
};
use yew::{html::Html, prelude::*};

use crate::entry_point::console_log;

const ZERO_WIDTH_SPACE: &str = "\u{200B}";

#[derive(PartialEq, Clone)]
pub struct ToolState {
    name: AttrValue,
    css_icon: AttrValue,
}

#[derive(Clone, PartialEq)]
pub struct TextAreaState {
    selected_tool: Option<usize>,
    tools: Vec<ToolState>,
    forest: Rc<RefCell<RichForest>>,
    forest_content: Rc<RefCell<RichContentTable>>,
    document: Ptr<RichTag>,
    focused_node: Ptr<RichTag>,
    ast_selection: RichSelection,
    update_counter: usize,
}

impl TextAreaState {
    pub fn new() -> Self {
        let mut forest = RichForest::new();
        let mut forest_content = HashMap::new();
        let document = forest.allocate_node(RichNode {
            kind: RichKind::Document,
        });

        let bold = forest.allocate_node(RichNode {
            kind: RichKind::Bold,
        });

        let text = forest.allocate_node(RichNode {
            kind: RichKind::Text,
        });

        forest.add_child(bold, text);
        forest.add_child(document, bold);

        forest_content
            .entry(text)
            .or_insert(RichData::Text(TextState {
                text: String::from("Hello World"),
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset("Hello World", 5)),
            }));

        /* Hardcoded AST:
         * --------------
         *  Document
         *  ..Bold
         *  ....Text
         *  ......'HELLO WORLD'
         * */

        Self {
            selected_tool: None,
            tools: vec![
                ToolState {
                    name: AttrValue::Static("bold-tool"),
                    css_icon: AttrValue::Static("gg-format-bold"),
                },
                ToolState {
                    name: AttrValue::Static("italics-tool"),
                    css_icon: AttrValue::Static("gg-format-italic"),
                },
                ToolState {
                    name: AttrValue::Static("underline-tool"),
                    css_icon: AttrValue::Static("gg-format-underline"),
                },
                ToolState {
                    name: AttrValue::Static("list-tool"),
                    css_icon: AttrValue::Static("gg-layout-list"),
                },
            ],
            forest: Rc::new(RefCell::new(forest)),
            forest_content: Rc::new(RefCell::new(forest_content)),
            document,
            focused_node: Ptr::null(),
            ast_selection: RichSelection::new(),
            update_counter: 0,
        }
    }

    pub fn set_tool(mut self, tool_idx: Option<usize>) -> Self {
        self.selected_tool = tool_idx;
        self
    }

    pub fn trigger_update(mut self) -> Self {
        self.update_counter += 1;
        self
    }

    pub fn set_focused_node(mut self, node: Ptr<RichTag>) -> Self {
        self.focused_node = node;
        self
    }

    pub fn flush_overlays(&self) {
        let document = self.document;
        {
            let forest = &mut *self.forest.borrow_mut();
            let forest_content = &mut *self.forest_content.borrow_mut();
            forest.traverse_all(document, |forest, node| {
                if let RichKind::Text = forest.get_data(node).kind {
                    let text_node_state = forest_content
                        .get_mut(&node)
                        .and_then(RichData::as_text_mut)
                        .expect("Failed to fetch text-node content");
                    text_node_state.selection = TextSelectionKind::Unselected;
                }
                false
            });
        }
    }
}
impl Default for TextAreaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Properties)]
struct TextAreaToolProps {
    tool_idx: usize,
    tool_info: ToolState,
    parent_state: UseStateHandle<TextAreaState>,
    #[prop_or_default]
    children: Html,
}

#[function_component]
fn TextAreaTool(p: &TextAreaToolProps) -> Html {
    let local_tool_info = p.tool_info.clone();
    let classes = [
        local_tool_info.name.as_ref(),
        local_tool_info.css_icon.as_ref(),
    ]
    .into_iter()
    .enumerate()
    .fold(String::new(), |mut acc, (idx, item)| {
        if idx > 0 {
            acc.push(' ');
        }
        acc.push_str(item);
        acc
    });
    let tool_idx = p.tool_idx;
    let onclick = Callback::from({
        let parent_state = p.parent_state.clone();
        move |e: web_sys::MouseEvent| {
            e.prevent_default();
            parent_state.set(parent_state.deref().clone().set_tool(Some(tool_idx)));
        }
    });
    html! {
        if p.parent_state.selected_tool.eq(&Some(tool_idx)) {
            <text-tool {onclick}  class="active-tool">
                <span class={classes}/>
                {p.children.clone()}
            </text-tool>
        }else{
           <text-tool {onclick}>
                <span class={classes}/>
                {p.children.clone()}
            </text-tool>
        }
    }
}

#[function_component]
pub fn RichTextArea() -> Html {
    let state = use_state(TextAreaState::new);
    let input_ref = NodeRef::default();

    let tool_iter = state.tools.clone().into_iter().enumerate().map({
        let state = state.clone();
        move |(idx, tool_ctx)| {
            html! {
                <TextAreaTool parent_state={state.clone()} tool_idx={idx} tool_info={tool_ctx}/>
            }
        }
    });

    let oninput = Callback::from({
        let state = state.clone();
        let input_ref = input_ref.clone();
        move |_e: InputEvent| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let input_selection_start = input.selection_start().ok().flatten().unwrap_or(0);
                let input_selection_end = input.selection_end().ok().flatten().unwrap_or(0);
                let value = input.value();

                let focused_node = state.focused_node;
                if focused_node == Ptr::null() {
                    return;
                }
                let focused_node_kind = state.forest.borrow().get_data(focused_node).clone().kind;
                let mut forest_content = state.forest_content.borrow_mut();

                match focused_node_kind {
                    RichKind::Text => {
                        if let Some(RichData::Text(TextState { text, selection })) =
                            forest_content.get_mut(&focused_node)
                        {
                            text.clear();
                            text.push_str(value.as_str());
                            *selection = TextSelectionKind::Caret(RichOffset::from_utf16_offset(
                                &value,
                                input_selection_start as usize,
                            ));
                        }
                    }
                    RichKind::Newline => {}
                    _ => {}
                }
                console_log("ON_INPUT FIRED!".to_string());
                state.set(state.deref().clone().trigger_update());
            }
        }
    });
    let onselectionchange = Callback::from({
        let state_handler = state.clone();
        move |e: Event| {
            let mut state = state_handler.deref().clone();

            let focused_node = state.focused_node;
            let input = e
                .target_dyn_into::<HtmlInputElement>()
                .expect("Failed to cast to InputElement");
            console_log(String::from("Selection change detected"));

            let input_text = input.value();
            let selection_start = RichOffset::from_utf16_offset(
                &input_text,
                input.selection_start().ok().flatten().unwrap_or(0) as usize,
            );
            let selection_end = RichOffset::from_utf16_offset(
                &input_text,
                input.selection_end().ok().flatten().unwrap_or(0) as usize,
            );
            if let Some(text_state) = state
                .forest_content
                .borrow_mut()
                .get_mut(&focused_node)
                .and_then(RichData::as_text_mut)
            {
                if selection_start == selection_end {
                    text_state.selection = TextSelectionKind::Caret(selection_start);
                } else {
                    text_state.selection = TextSelectionKind::Range(RichRange {
                        start: selection_start,
                        end: selection_end,
                    })
                }
            }
            state = state.trigger_update();
            state_handler.set(state);
        }
    });
    let onkeydown = Callback::from({
        let state_handler = state.clone();
        move |e: KeyboardEvent| {
            let mut state = state_handler.deref().clone();

            let focused_node = state.focused_node;
            if focused_node == Ptr::null() {
                return;
            }
            console_log(format!("Input keycode: {}, node focused", e.key()));
            let input_elem = e
                .target_dyn_into::<HtmlInputElement>()
                .expect("Failed to cast to InputElement");

            let focused_node_type = state.forest.borrow().get_data(focused_node).clone();

            match focused_node_type.kind {
                RichKind::Text => {
                    if e.key() == "Enter" {
                        let new_text_node = {
                            let forest = &mut *state.forest.borrow_mut();
                            let forest_content = &mut *state.forest_content.borrow_mut();

                            let focused_node_state = forest_content
                                .get(&focused_node)
                                .and_then(RichData::as_text)
                                .cloned()
                                .expect("Failed to fetch text state");

                            let focused_parent = *forest.get_parent(focused_node);
                            let newline_element = forest.allocate_node(RichNode {
                                kind: RichKind::Newline,
                            });

                            let new_text_node = forest.allocate_node(RichNode {
                                kind: RichKind::Text,
                            });

                            let caret_position = focused_node_state
                                .selection
                                .as_caret()
                                .expect("Focused node expected to have a caret selection");
                            let lower_split =
                                String::from(&focused_node_state.text[..caret_position.utf8]);
                            let upper_split =
                                String::from(&focused_node_state.text[caret_position.utf8..]);

                            if let Some(text_state) = forest_content
                                .get_mut(&focused_node)
                                .and_then(RichData::as_text_mut)
                            {
                                text_state.text = lower_split;
                            }

                            forest_content.insert(
                                new_text_node,
                                RichData::Text(TextState {
                                    text: upper_split.clone(),
                                    selection: TextSelectionKind::Caret(RichOffset::zero()),
                                }),
                            );

                            input_elem.set_value(&upper_split);
                            forest.add_child_at(focused_parent, newline_element,focused_node,1);
                            forest.add_child_at(focused_parent, new_text_node,newline_element,1);

                            new_text_node
                        };
                        state = state.set_focused_node(new_text_node);
                        state.flush_overlays();
                        state
                            .forest_content
                            .borrow_mut()
                            .get_mut(&new_text_node)
                            .and_then(RichData::as_text_mut)
                            .expect("Failed to fetch text data")
                            .selection = TextSelectionKind::Caret(RichOffset::zero());
                    }
                }
                _ => panic!("NOT IMPLEMENTED"),
            }

            state = state.trigger_update();

            state_handler.set(state);
        }
    });

    html! {
        <>
            <rich-text-area>
                <input
                    ref={input_ref.clone()}
                    type="text"
                    {oninput}
                    {onselectionchange}
                    {onkeydown}
                />

                <tool-container>
                    {for tool_iter}
                </tool-container>
                <text-container>
                    <text-pure>
                        // START RECURSING AT THE ROOT OF THE AST
                        <RichDisplayNode
                            ta_state={state.clone()}
                            root={state.document}
                            forest={state.forest.clone()}
                            input_ref={input_ref.clone()}
                            overlays_enabled={false}
                        />
                    </text-pure>
                    <text-overlays>
                        <RichDisplayNode
                            ta_state={state.clone()}
                            root={state.document}
                            forest={state.forest.clone()}
                            input_ref={input_ref.clone()}
                            overlays_enabled={true}
                        />
                    </text-overlays>
                </text-container>
            </rich-text-area>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct RichDisplayNodeProps {
    pub ta_state: UseStateHandle<TextAreaState>,
    pub root: Ptr<RichTag>,
    pub forest: Rc<RefCell<RichForest>>,
    pub input_ref: NodeRef,
    pub overlays_enabled: bool,
}

#[function_component]
fn RichDisplayNode(p: &RichDisplayNodeProps) -> Html {
    let root = p.root;
    let forest = p.forest.clone();
    let ta_state = p.ta_state.clone();
    let input_ref = p.input_ref.clone();

    let root_node = forest.borrow().get_data(root).clone();
    let child_list = forest.borrow().get_children(root).clone();
    let child_iterator = child_list.iter().copied().map(|child_ptr| {
        html! {
            <RichDisplayNode
                ta_state={ta_state.clone()}
                root={child_ptr}
                forest={forest.clone()}
                input_ref={input_ref.clone()}
                overlays_enabled={p.overlays_enabled}
            />
        }
    });

    match root_node.kind {
        RichKind::Document => {
            html! {
                <>
                    <rt-document data-ast-id={root.to_index().to_string()}>
                        {for child_iterator}
                    </rt-document>
                </>
            }
        }
        RichKind::Bold => {
            html! {
                <>
                    <rt-bold data-ast-id={root.to_index().to_string()} >
                        {for child_iterator}
                    </rt-bold>
                </>
            }
        }
        RichKind::Text => {
            let clear_overlays = {
                let ta_state = ta_state.clone();
                Callback::from(move |_e: MouseEvent| {
                    ta_state.deref().flush_overlays();
                })
            };

            // 1. Callback to focus the hidden input when a custom text node is clicked
            let hand_over_control_to_hidden_input = {
                let input_ref = input_ref.clone();
                let state_handler = ta_state.clone();
                Callback::from(move |_e: MouseEvent| {
                    let mut ta_state = state_handler.deref().clone();

                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        if let Ok(selection) = compute_text_selection() {
                            match selection.kind() {
                                RichSelectionKind::MonoNode(character_range) => {
                                    let node = selection.start_node;
                                    if let Some(state) = ta_state
                                        .forest_content
                                        .borrow_mut()
                                        .get_mut(&node)
                                        .and_then(RichData::as_text_mut)
                                    {
                                        match character_range {
                                            RichCharacterRangeKind::MonoCharacter => {
                                                state.selection = TextSelectionKind::Caret(
                                                    selection.start_offset,
                                                );
                                            }
                                            RichCharacterRangeKind::MultiCharacter => {
                                                state.selection =
                                                    TextSelectionKind::Range(RichRange {
                                                        start: selection.start_offset,
                                                        end: selection.end_offset,
                                                    })
                                            }
                                        }

                                        input.set_value(state.text.as_str());
                                        input
                                            .set_selection_start(Some(
                                                selection.start_offset.utf16 as u32,
                                            ))
                                            .expect("Failed to set start selection");
                                        input
                                            .set_selection_end(Some(
                                                selection.end_offset.utf16 as u32,
                                            ))
                                            .expect("Failed to set start selection");
                                    }
                                }
                                RichSelectionKind::MultiNode => {}
                            }

                            let _ = input.focus();
                        }

                        ta_state = ta_state.set_focused_node(root);
                        state_handler.set(ta_state);
                    }
                })
            };

            let text_state = ta_state
                .forest_content
                .borrow()
                .get(&root)
                .and_then(RichData::as_text)
                .cloned()
                .expect("Failed to fetch Text state");

            if !p.overlays_enabled {
                return html! {
                    <>
                        <rt-text
                            data-ast-id={root.to_index().to_string()}
                            onmousedown={clear_overlays}
                            onmouseup={hand_over_control_to_hidden_input}
                        >
                            {text_state.text}
                        </rt-text>
                    </>
                };
            }

            match text_state.selection {
                TextSelectionKind::Unselected => {
                    html! {
                        <>
                            <rt-text data-ast-id={root.to_index().to_string()}>
                                {text_state.text}
                            </rt-text>
                        </>
                    }
                }
                TextSelectionKind::Caret(split_offset) => {
                    let split_idx = split_offset.utf8;
                    let lower = &text_state.text[0..split_idx];
                    let upper = &text_state.text[split_idx..];

                    html! {
                        <>
                            <rt-text data-ast-id={root.to_index().to_string()}>
                                <rt-caret>{lower}</rt-caret>{upper}
                            </rt-text>
                        </>
                    }
                }
                TextSelectionKind::Range(range) => {
                    let head = &text_state.text[0..range.start.utf8];
                    let selected = &text_state.text[range.start.utf8..range.end.utf8];
                    let tail = &text_state.text[range.end.utf8..];
                    html! {
                        <>
                            <rt-text data-ast-id={root.to_index().to_string()}>
                                {head}<rt-selection>{selected}</rt-selection>{tail}
                            </rt-text>
                        </>
                    }
                }
            }
        }
        RichKind::Newline => {
            html! {
                <>
                    <br/>{ZERO_WIDTH_SPACE}
                </>
            }
        }
    }
}

fn compute_text_selection() -> Result<RichSelection, wasm_bindgen::JsValue> {
    let selection = web_sys::window()
        .unwrap()
        .document()
        .ok_or(JsValue::from_str("No document"))?
        .get_selection()?
        .ok_or(JsValue::from_str("No selection"))?;

    let range = selection.get_range_at(0)?;
    let start_dom = range.start_container()?;
    let end_dom = range.end_container()?;

    let start_dom_name = start_dom.node_name();
    let end_dom_name = end_dom.node_name();
    console_log(format!("START_CONTAINER_NAME = {}", start_dom_name));
    console_log(format!("END_CONTAINER_NAME = {}", end_dom_name));
    if start_dom_name != "#text" || end_dom_name != "#text" {
        return Err(JsValue::from_str("SELECT-ALL NOT IMPLEMENTED"));
    }

    let start_dom_text = start_dom
        .text_content()
        .ok_or(JsValue::from_str("No start text content"))?;
    let end_dom_text = end_dom.text_content().ok_or("No end text content")?;

    let start_offset =
        RichOffset::from_utf16_offset(&start_dom_text, range.start_offset().unwrap() as usize);
    let end_offset =
        RichOffset::from_utf16_offset(&end_dom_text, range.end_offset().unwrap() as usize);

    let start_rich_ast = map_dom_node_to_rich_ast(start_dom);
    let end_rich_ast = map_dom_node_to_rich_ast(end_dom);

    Ok(RichSelection {
        start_node: start_rich_ast,
        end_node: end_rich_ast,
        start_offset,
        end_offset,
    })
}

pub fn map_dom_node_to_rich_ast(node: web_sys::Node) -> Ptr<RichTag> {
    let mut root = node.clone();
    while let Some(parent) = root.parent_node() {
        if root.node_type() == web_sys::Node::TEXT_NODE {
            root = parent;
            continue;
        }

        let root_elem = root
            .dyn_into::<web_sys::Element>()
            .expect("Failed to cast to Element");

        if let Some(ast_id) = root_elem.get_attribute("data-ast-id") {
            return Ptr::from(ast_id.parse::<usize>().expect("Failed to parse ast id"));
        } else {
            // The reason the attribute might not be read sometimes is because
            // The user can double-click and the browser will automatically
            // put the start and end nodes on a DOM node that contains ALL text.
            // I handle that case somewhere higher up,
            // first node is always expected to be a TEXT_NODE
            // I'm leaving these logs in for debugging purposes just incase I miss
            // other cases soemwhere else
            console_log("WTF! Failed to read ast ID".to_string());
            console_log(root_elem.outer_html());
            console_log(format!("origin node type = {}", node.node_name()));
            console_log(format!(
                "origin value = {}",
                node.node_value().unwrap_or(String::from("ERROR"))
            ));
            panic!("Unexpected browser selection behavior examine trace.");
        }
    }
    Ptr::null()
}
