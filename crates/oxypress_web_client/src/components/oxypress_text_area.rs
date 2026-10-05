use std::rc::Rc;
use yew::{html::Html, prelude::*};

#[derive(PartialEq, Clone)]
pub struct ToolState {
    name: AttrValue,
}

#[derive(Clone, PartialEq)]
pub struct TextAreaState {
    selected_tool: Option<i32>,
    tools: Vec<ToolState>,
}
impl TextAreaState {
    pub fn new() -> Self {
        Self {
            selected_tool: None,
            tools: vec![
                ToolState {
                    name: AttrValue::Static("bold"),
                },
                ToolState {
                    name: AttrValue::Static("Italics"),
                },
                ToolState {
                    name: AttrValue::Static("Underline"),
                },
            ],
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
    tool_info: ToolState,
}

#[function_component]
fn TextAreaTool(p: &TextAreaToolProps) -> Html {
    html! {
        <text-tool>
            {p.clone().tool_info.name}
        </text-tool>
    }
}

#[function_component]
fn OxypressTextArea() -> Html {
    let state = use_state(|| TextAreaState::new());

    let tool_iter = state
        .tools
        .clone()
        .into_iter()
        .map(|tool_ctx| html! {   <TextAreaTool tool_info={tool_ctx}/>  });
    html! {
        <>
            <oxypress-text-area>
                <tool-container>
                    {for tool_iter}
                </tool-container>
                <text-container>
                    {"Hello world"}
                </text-container>
            </oxypress-text-area>
        </>
    }
}
