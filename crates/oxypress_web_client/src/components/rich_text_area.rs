use std::{cell::RefCell, ops::Deref, rc::Rc};
use yew::{html::Html, prelude::*};

#[derive(PartialEq, Clone)]
pub struct ToolState {
    name: AttrValue,
    css_icon: AttrValue,
}

#[derive(Clone, PartialEq)]
pub struct TextAreaState {
    selected_tool: Option<usize>,
    tools: Vec<ToolState>,
}
impl TextAreaState {
    pub fn new() -> Self {
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
        }
    }
    pub fn set_tool(mut self, tool_idx: Option<usize>) -> Self {
        self.selected_tool = tool_idx;
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
    let tool_iter = state.tools.clone().into_iter().enumerate().map({
        move |(idx, tool_ctx)| {
            html! {
                <TextAreaTool parent_state={state.clone()} tool_idx={idx} tool_info={tool_ctx}/>
            }
        }
    });
    html! {
        <>
            <rich-text-area>
                <tool-container>
                    {for tool_iter}
                </tool-container>
                <text-container contenteditable="">
                    {"Hello world"}
                </text-container>
            </rich-text-area>
        </>
    }
}


