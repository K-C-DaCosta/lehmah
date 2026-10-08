use std::{ops::Deref, rc::Rc};
use yew::{prelude::*, Callback, Properties};


#[derive(Clone, PartialEq, Properties)]
pub struct TabElemParam<ExtState>
where
    ExtState: PartialEq + 'static,
{
    pub tab_idx: usize,
    pub active_tab_idx: usize,
    pub tab_name: AttrValue,
    pub tab_content: Callback<UseStateHandle<ExtState>, Html>,
    pub frame_state: UseStateHandle<TabbedFrameState>,
    pub parent_state: UseStateHandle<ExtState>,
}

#[function_component]
pub fn TabElem<ExtState>(p: &TabElemParam<ExtState>) -> Html
where
    ExtState: PartialEq + 'static,
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

#[derive(PartialEq)]
pub struct TabEntry<ExtState> {
    pub tab_name: AttrValue,
    pub tab_content: Callback<UseStateHandle<ExtState>, Html>,
}

impl<ExtState> Clone for TabEntry<ExtState> {
    fn clone(&self) -> Self {
        Self {
            tab_name: self.tab_name.clone(),
            tab_content: self.tab_content.clone(),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct TabbedFrameParams<ExtState>
where
    ExtState: PartialEq,
{
    pub parent_state: UseStateHandle<ExtState>,
    #[prop_or_default]
    pub tabs: Vec<TabEntry<ExtState>>,
    #[prop_or_default]
    pub children: Option<Html>,
}

#[function_component]
pub fn TabbedFrame<ExtState>(p: &TabbedFrameParams<ExtState>) -> Html
where
    ExtState: PartialEq + 'static,
{
    let state = use_state(|| TabbedFrameState { active_tab_idx: 0 });
    let tab_len = p.tabs.len();

    let tab_elem_iter = p.tabs.clone().into_iter().enumerate().map({
        let state = state.clone();
        move |(idx, tab_entry)| {
            html! {
                <TabElem::<ExtState>
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
    let onwheel = Callback::from({
        let state = state.clone();
        move |event: web_sys::WheelEvent| {
            event.prevent_default();
            let offset = event.delta_y().signum() as isize;
            let new_idx =
                (state.active_tab_idx as isize + offset).rem_euclid(tab_len as isize) as usize;
            state.set(TabbedFrameState {
                active_tab_idx: new_idx,
            });
        }
    });
    let active_tab_content = p
        .tabs
        .get(state.active_tab_idx)
        .map(|tab| tab.tab_content.emit(p.parent_state.clone()))
        .unwrap_or_else(|| html! {<>{"Warning: the frame is empty, probably not intended"}</>});
    html! {
        <tabbed-frame {onwheel}>
            <tab-container>
                {for tab_elem_iter}
            </tab-container>
            <frame-body>
                {active_tab_content}
            </frame-body>
        </tabbed-frame>
    }
}
