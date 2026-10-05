use std::{ops::Deref, rc::Rc};
use yew::{prelude::*, Callback, Properties};

use crate::components::tabbed_frame::_TabElemParam::parent_state;

#[derive(Clone, PartialEq, Properties)]
pub struct TabElemParam<ParentState>
where
    ParentState: PartialEq + 'static,
{
    pub tab_idx: usize,
    pub active_tab_idx:usize,
    pub tab_name: AttrValue,
    pub tab_content: Callback<UseStateHandle<ParentState>, Html>,
    pub frame_state: UseStateHandle<TabbedFrameState>,
    pub parent_state: UseStateHandle<ParentState>,
}

#[derive(Clone)]
pub struct TabElemState<ParentState> {
    pub tab_name: AttrValue,
    pub tab_content: TabCallback<ParentState>,
    pub frame_state: UseStateHandle<TabbedFrameState>,
}

#[function_component]
pub fn TabElem<ParentState>(p: &TabElemParam<ParentState>) -> Html
where
    ParentState: PartialEq + 'static,
{
    // We only need local state if something *inside* TabElem changes independently.
    // For things passed down from props, read them straight from `p`.
    let frame_state = p.frame_state.clone();
    let tab_idx = p.tab_idx;
    let active_tab_idx = p.active_tab_idx;

    let onclick = Callback::from(move |_| {
        frame_state.set(TabbedFrameState {
            active_tab_idx: tab_idx,
        })
    });

    html! {
        if tab_idx != active_tab_idx {
            <tab-elem {onclick}>
                {p.tab_name.clone()}
            </tab-elem>
        }else{
            <tab-elem {onclick} class="tab_currently_active">
                {p.tab_name.clone()}
            </tab-elem>
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TabbedFrameState {
    active_tab_idx: usize,
}

pub struct TabCallback<ParentState> {
    inner: fn(UseStateHandle<ParentState>) -> Html,
}
impl<ParentState> TabCallback<ParentState> {
    pub fn new(inner: fn(UseStateHandle<ParentState>) -> Html) -> Self {
        Self { inner }
    }
}
impl<ParentState> Clone for TabCallback<ParentState> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<ParentState> Deref for TabCallback<ParentState> {
    type Target = fn(UseStateHandle<ParentState>) -> Html;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(PartialEq)]
pub struct TabEntry<ParentState> {
    pub tab_name: AttrValue,
    pub tab_content: Callback<UseStateHandle<ParentState>, Html>,
}

impl<ParentState> Clone for TabEntry<ParentState> {
    fn clone(&self) -> Self {
        Self {
            tab_name: self.tab_name.clone(),
            tab_content: self.tab_content.clone(),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct TabbedFrameParams<ParentState>
where
    ParentState: PartialEq,
{
    pub parent_state: UseStateHandle<ParentState>,
    #[prop_or_default]
    pub tabs: Vec<TabEntry<ParentState>>,
    #[prop_or_default]
    pub children: Option<Html>,
}

#[function_component]
pub fn TabbedFrame<ParentState>(p: &TabbedFrameParams<ParentState>) -> Html
where
    ParentState: PartialEq + 'static,
{
    let state = use_state(|| TabbedFrameState { active_tab_idx: 0 });

    let tab_elem_iter = p.tabs.clone().into_iter().enumerate().map({
        let state = state.clone();
        move |(idx, tab_entry)| {
            html! {
                <TabElem::<ParentState>
                    tab_idx={idx}
                    active_tab_idx={state.active_tab_idx}
                    parent_state={p.parent_state.clone()}
                    tab_name={tab_entry.tab_name}
                    tab_content={tab_entry.tab_content}
                    frame_state={state.clone()}
                />
            }
        }
    });
    let active_tab_content = p.tabs[state.active_tab_idx]
        .tab_content
        .emit(p.parent_state.clone());
    html! {
        <tabbed-frame>
            <tab-container>
                {for tab_elem_iter}
            </tab-container>
            <frame-body>
                {active_tab_content}
            </frame-body>
        </tabbed-frame>
    }
}
