use std::{ops::Deref, rc::Rc};

use super::dom_pointer::*;
use crate::components::tabbed_frame::{TabCallback, TabElemParam, TabEntry, TabbedFrame};
use bitflags::bitflags;
use oxypress_core::DomPointer;
use web_sys::{
    console,
    js_sys::{self, Function},
    wasm_bindgen::{self, prelude::Closure, JsCast, JsValue},
    Document, Element, NodeFilter, TreeWalker,
};
use yew::{prelude::*, Callback, Properties};

fn console_log(text: String) {
    console::log_1(&JsValue::from_str(&text));
}
#[derive(Clone)]
struct RadioComponentState {
    count: i32,
    station: AttrValue,
}
impl RadioComponentState {
    pub fn new() -> Self {
        Self {
            count: 0,
            station: AttrValue::Static("Night City"),
        }
    }
    pub fn offset_count(mut self, offset: i32) -> Self {
        self.count += offset;
        self
    }
}

#[derive(Properties, PartialEq, Default)]
pub struct RadioProps {
    #[prop_or_default]
    pub is_loading: bool,
    #[prop_or_default]
    pub channels: u32,
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn RadioComponent(p: &RadioProps) -> Html {
    let state = use_state(|| RadioComponentState::new());
    let onclick = Callback::from({
        let state = state.clone();
        move |_| {
            state.set(state.deref().clone().offset_count(1));
        }
    });
    html! {
        <radio-component>
            {"This is a radio component"}<br/>
            {"loading = "}{p.is_loading}<br/>
            {"radio count:"}{state.count}<br/>
            <input {onclick} type="button" value="mutate"/><br/>
            {p.children.clone()}
        </radio-component>
    }
}

#[derive(Clone, PartialEq)]
pub struct AppState {
    count: i32,
}
impl AppState {
    pub fn increment(&self) -> Self {
        let old_self = self;
        let mut new_self = old_self.clone();
        new_self.count = old_self.count + 1;
        new_self
    }
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| AppState { count: 0 });
    let onclick = Callback::from({
        let state = state.clone();
        move |_| {
            state.set(state.increment());
        }
    });

    let tabs = vec![
        TabEntry {
            tab_name: AttrValue::Static("Tab 1"),
            tab_content: Callback::from(|parent_state| {
                html! {
                    <>{"This is the content for tab1"}</>
                }
            }),
        },
        TabEntry {
            tab_name: AttrValue::Static("Tab 2"),
            tab_content: Callback::from({
                |parent_state: UseStateHandle<AppState>| {
                    html! {
                        <>
                            <RadioComponent/>
                        </>
                    }
                }
            }),
        },
        TabEntry {
            tab_name: AttrValue::Static("Tab 3"),
            tab_content: Callback::from(|parent_state: UseStateHandle<AppState>| {
                html! {
                    <>{"This is the content for Tab 3:[count:"}{parent_state.count}{"]"}</>
                }
            }),
        },
    ];

    html! {
        <>
            <TabbedFrame::<AppState> {tabs} parent_state={state.clone()}/>
            <div class="test">
                <input {onclick} type="button" value="Increment"/><br/>
                <p>{"hello world"}{state.count}</p>
            </div>

        </>
    }
}
