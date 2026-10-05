use std::{ops::Deref, rc::Rc};

use super::dom_pointer::*;
use crate::components::tabbed_frame::{TabElemParam, TabEntry, TabbedFrame};
use bitflags::bitflags;
use oxypress_core::DomPointer;
use web_sys::{
    console,
    js_sys::{self, Function},
    wasm_bindgen::{self, prelude::Closure, JsCast, JsValue},
    Document, Element, NodeFilter, TreeWalker,
};
use yew::{prelude::*, Callback, ContextProvider, Properties};

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

#[derive(Properties, PartialEq)]
pub struct RadioProps<ExtState>
where
    ExtState: PartialEq,
{
    #[prop_or_default]
    pub is_loading: bool,
    #[prop_or_default]
    pub channels: u32,
    #[prop_or_default]
    pub children: Html,
    pub get_count: Callback<UseStateHandle<ExtState>, i32>,
    pub onclick: Callback<UseStateHandle<ExtState>, ()>,
    pub ext_state: UseStateHandle<ExtState>,
}

#[function_component]
pub fn RadioComponent<ExtState>(p: &RadioProps<ExtState>) -> Html
where
    ExtState: PartialEq + 'static,
{
    let ext_state = p.ext_state.clone();
    let get_count = p.get_count.clone();
    let local_state = use_state(|| RadioComponentState::new());

    let onclick = Callback::from({
        let onclick = p.onclick.clone();
        let ext_state = ext_state.clone();
        move |_| {
            onclick.emit(ext_state.clone());
        }
    });

    html! {
        <radio-component>
            {"This is a radio component"}<br/>
            {"loading = "}{p.is_loading}<br/>
            {"radio count:"}{get_count.emit(ext_state.clone())}<br/>
            <input onclick={onclick} type="button" value="mutate"/><br/>
            {p.children.clone()}
        </radio-component>
    }
}

#[derive(Clone, PartialEq)]
pub struct AppState {
    count: i32,
    radio_count: i32,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            count: 0,
            radio_count: 0,
        }
    }
    pub fn increment(&self) -> Self {
        let old_self = self;
        let mut new_self = old_self.clone();
        new_self.count = old_self.count + 1;
        new_self
    }
    pub fn offset_radio(&self) -> Self {
        let mut old = self.clone();
        old.radio_count += 1;
        old
    }
}
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(|| AppState::new());

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
                            <RadioComponent::<AppState>
                                get_count={
                                    Callback::from(|state:UseStateHandle<AppState>|{
                                        state.radio_count
                                    })
                                }
                                ext_state={
                                    parent_state.clone()
                                }
                                onclick={
                                    Callback::from(|state:UseStateHandle<AppState>|{
                                        state.set(state.deref().offset_radio());
                                    })
                                }
                            />
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
        </>
    }
}
