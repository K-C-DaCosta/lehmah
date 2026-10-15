use oxypress_core::{
    collections::{flatforest::FlatForest, Ptr},
    rich_text_core::{RichForest, RichKind, RichNode, RichTag},
};
use std::{cell::RefCell, char, ops::Deref, rc::Rc};
use web_sys::{
    wasm_bindgen::{JsCast, JsValue},
    HtmlElement, HtmlInputElement,
};
use yew::{html::Html, prelude::*};

use crate::entry_point::console_log;

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
    root: Ptr<RichTag>,
    focused_node: Ptr<RichTag>,
    update_counter: usize,
}

impl TextAreaState {
    pub fn new() -> Self {
        let mut forest = RichForest::new();
        let root = forest.allocate_node(RichNode {
            kind: RichKind::Origin,
            aux_data: String::new(),
        });

        let bold = forest.allocate_node(RichNode {
            kind: RichKind::Bold,
            aux_data: String::new(),
        });

        let text = forest.allocate_node(RichNode {
            kind: RichKind::Text,
            aux_data: String::from("Hello world"),
        });

        forest.add_child(bold, text);
        forest.add_child(root, bold);
        /*
         *  .
         *  ..Bold
         *  ....Text('HELLO WORLD')
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
            root,
            focused_node: Ptr::null(),
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

    let onkeydown = Callback::from({
        let state = state.clone();
        move |e: web_sys::KeyboardEvent| {
            e.prevent_default();
            console_log(format!("Counter = {:?}", state.update_counter));
            state.set(state.deref().clone().trigger_update());
        }
    });

    let oninput = Callback::from({
        let state = state.clone();
        let input_ref = input_ref.clone();
        move |_e: InputEvent| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let cursor_pos = input.selection_start().ok().flatten().unwrap_or(0);
                let value = input.value();

                let focused_node = state.focused_node;
                if focused_node == Ptr::null() {
                    return;
                }

                let mut forest = state.forest.borrow_mut();
                forest.get_data_mut(focused_node).aux_data.clear();
                forest
                    .get_data_mut(focused_node)
                    .aux_data
                    .push_str(value.as_str());

                state.set(state.deref().clone().trigger_update());
            }
        }
    });

    html! {
        <>
            <rich-text-area>
                <input ref={input_ref.clone()} type="text" {oninput}   />
                <tool-container>
                    {for tool_iter}
                </tool-container>
                <text-container>
                    // START RECURSING AT THE ROOT OF THE AST
                    <RichDisplayNode
                        ta_state={state.clone()}
                        root={state.root}
                        forest={state.forest.clone()}
                        input_ref={input_ref.clone()}
                    />
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
            />
        }
    });

    match root_node.kind {
        RichKind::Origin => {
            html! {
                <>
                    {for child_iterator}
                </>
            }
        }
        RichKind::Bold => {
            html! {
                <>
                    <rt-bold>
                        {for child_iterator}
                    </rt-bold>
                </>
            }
        }
        RichKind::Text => {
            let onkeydown = Callback::from({
                move |e: web_sys::KeyboardEvent| {
                    let forest = forest.clone();
                    let root = root;
                    forest
                        .borrow_mut()
                        .get_data_mut(root)
                        .aux_data
                        .push_str(e.key().as_str());

                    console_log(format!(
                        "text is now: {}",
                        forest.borrow().get_data(root).aux_data
                    ));
                }
            });
            // 1. Callback to focus the hidden input when a custom text node is clicked
            let on_text_click = {
                let input_ref = input_ref.clone();
                let ta_state = ta_state.clone();
                Callback::from(move |_e: MouseEvent| {
                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        ta_state.set(ta_state.deref().clone().set_focused_node(root));
                        let _ = input.focus();
                    }
                })
            };

            html! {
                <>
                    <rt-text onclick={on_text_click}>
                        {root_node.aux_data}
                    </rt-text>
                </>
            }
        }
        RichKind::Newline => {
            html! {
                <>
                    <br/>
                </>
            }
        }
    }
}
