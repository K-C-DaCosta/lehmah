use bitflags::Flags;
use core::panic;
use oxypress_core::{
    collections::{
        ringbuffer::{RingBuffer, RingError},
        Ptr,
    },
    rich_text::*,
};
use std::{cell::RefCell, collections::HashMap, ops::Deref, panic::Location, rc::Rc};
use web_sys::{
    wasm_bindgen::{self, JsCast, JsValue},
    FocusOptions, HtmlInputElement, Range,
};
use yew::{html::Html, platform::fmt::buffer, prelude::*};

use crate::entry_point::console_log;

#[derive(Clone, PartialEq, Properties)]
struct TextAreaToolProps {
    tool_idx: usize,
    tool_info: ToolState,
    input_ref: NodeRef,
    parent_state: UseStateHandle<TextAreaState>,
    #[prop_or_default]
    children: Html,
}

#[function_component]
fn TextAreaTool(p: &TextAreaToolProps) -> Html {
    let local_tool_info = p.tool_info.clone();
    let classes = [local_tool_info.name, local_tool_info.css_icon]
        .into_iter()
        .fold(String::new(), |mut acc, item| {
            acc.push(' ');
            acc.push_str(item);
            acc.push(' ');
            acc
        });
    let tool_idx = p.tool_idx;
    let input_ref = p.input_ref.clone();
    let tool_onclick = Callback::from({
        let parent_state_handler = p.parent_state.clone();
        move |e: web_sys::MouseEvent| {
            e.prevent_default();
            let hidden_input_elem = input_ref
                .cast::<HtmlInputElement>()
                .expect("Failed to get hidden input");
            let mut state = parent_state_handler.deref().clone();
            let effect_buffer = state.effects_buffer.clone();
            state = state.set_focused_tool(Some(tool_idx));
            state
                .execute_tool_by_idx(tool_idx, &mut effect_buffer.borrow_mut())
                .apply(|effect| {
                    wasm32_unknown_unknown::effect_handler(effect, hidden_input_elem.clone());
                });
            state.trigger_update();
            parent_state_handler.set(state);
        }
    });
    html! {
        if p.parent_state.tools[tool_idx].is_selected {
            <text-tool onclick={tool_onclick}  class="active-tool">
                <span class={classes}/>
                {p.children.clone()}
            </text-tool>
        }else{
           <text-tool onclick={tool_onclick}>
                <span class={classes}/>
                {p.children.clone()}
            </text-tool>
        }
    }
}

#[function_component]
pub fn RichTextArea() -> Html {
    let state = use_state(|| TextAreaState::new().with_sample_document());
    let input_ref = NodeRef::default();

    let tool_iter = state.tools.clone().into_iter().enumerate().map({
        let state = state.clone();
        let input_ref = input_ref.clone();
        move |(idx, tool_ctx)| {
            html! {
                <TextAreaTool
                    parent_state={state.clone()}
                    tool_idx={idx}
                    tool_info={tool_ctx}
                    input_ref={input_ref.clone()}
                />
            }
        }
    });

    let sync_hidden_input_element_to_rich_ast_oninput = Callback::from({
        let state_handler = state.clone();
        let input_ref = input_ref.clone();
        move |_e: InputEvent| {
            let mut state = state_handler.deref().clone();
            let helpers = state.document_helpers();
            let input_elem = input_ref
                .cast::<HtmlInputElement>()
                .expect("Failed to case input ref");
            let input_selection_start = input_elem.selection_start().ok().flatten().unwrap_or(0);
            let input_selection_end = input_elem.selection_end().ok().flatten().unwrap_or(0);
            let value = input_elem.value();

            let focused_node = state.focused_node;
            if focused_node == Ptr::null() {
                return;
            }

            {
                let forest_content = &mut *state.forest_content.borrow_mut();
                let forest = &mut *state.forest.borrow_mut();
                let focused_node_kind = forest.get_data_mut(focused_node);
                match focused_node_kind {
                    RichNodeKind::Text(TextState { content, .. }) => {
                        if let Some(RichAuxData::Text(ExtraTextState { selection })) =
                            forest_content.get_mut(&focused_node)
                        {
                            content.clear();
                            content.push_str(value.as_str());
                            *selection = TextSelectionKind::Caret(RichOffset::from_utf16_offset(
                                &value,
                                input_selection_start as usize,
                            ));
                        }
                    }
                    RichNodeKind::Line => {}
                    _ => {}
                }
            }
            console_log("ON_INPUT FIRED!".into());
            state.trigger_update();

            state_handler.set(state);
        }
    });
    let sync_hidden_input_element_to_rich_ast_onselection_change = Callback::from({
        let state_handler = state.clone();
        move |e: Event| {
            console_log("ONSELECTIONCHANGE fired!".into());
            let mut state = state_handler.deref().clone();
            let helpers = state.document_helpers();

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
            let updated_document_selection = if let Some(text_state) = helpers
                .forest_content_mut()
                .get_mut(&focused_node)
                .and_then(RichAuxData::as_text_mut)
            {
                if selection_start == selection_end {
                    text_state.selection = TextSelectionKind::Caret(selection_start);
                    Some(RichSelection::mononode(
                        focused_node,
                        RichRange {
                            start: selection_start,
                            end: selection_end,
                        },
                    ))
                } else {
                    let range = RichRange {
                        start: selection_start,
                        end: selection_end,
                    };
                    text_state.selection = TextSelectionKind::Range(range);
                    Some(RichSelection::mononode(focused_node, range))
                }
            } else {
                None
            };

            *state.document_selection_mut() = updated_document_selection;
            let focused_node_styling = helpers
                .try_get_text_state(focused_node, |text_state| text_state.styling)
                .expect("Failed to get text state from focused node");
            update_toolbar_styles(&mut state, focused_node_styling);
            state.trigger_update();
            state_handler.set(state);
        }
    });
    let onkeydown = Callback::from({
        let state_handler = state.clone();
        move |e: KeyboardEvent| {
            let mut state = state_handler.deref().clone();
            let helpers = state.document_helpers();
            console_log(format!("Input keycode: {}, node focused", e.key()));
            let input_elem = e
                .target_dyn_into::<HtmlInputElement>()
                .expect("Failed to cast to InputElement");

            let document_selection = state.document_selection().expect("Failed to get selection");
            match document_selection.kind() {
                RichSelectionKind::MonoNode(RichCharacterRangeKind::MonoCharacter) => {
                    let focused_node = document_selection.start_node;
                    let focused_node_type =
                        helpers.try_node_data(focused_node, |node_kind| node_kind.clone());
                    match focused_node_type {
                        RichNodeKind::Text(focused_node_state) => match e.key().as_str() {
                            "Backspace" => {}
                            "Enter" => {
                                let effect_buffer = state.effects_buffer.clone();
                                create_new_line_at_caret(
                                    &mut state,
                                    focused_node,
                                    focused_node_state,
                                    document_selection,
                                    &mut effect_buffer.borrow_mut(),
                                )
                                .apply(|effect| {
                                    wasm32_unknown_unknown::effect_handler(
                                        effect,
                                        input_elem.clone(),
                                    );
                                });
                            }
                            "ArrowLeft" => {
                                let text_state = state
                                    .forest_content
                                    .borrow()
                                    .get(&focused_node)
                                    .and_then(RichAuxData::as_text)
                                    .cloned()
                                    .expect("Failed to get text state");

                                let caret_position = text_state.selection.as_caret();
                                if caret_position.is_some()
                                    && caret_position.unwrap() == RichOffset::zero()
                                {
                                    let effects_buffer = state.effects_buffer.clone();
                                    document_operations::move_caret_horizontal(
                                        &mut state,
                                        &mut effects_buffer.borrow_mut(),
                                        focused_node,
                                        CaretDirection::Left,
                                    )
                                    .apply(|effect| {
                                        wasm32_unknown_unknown::effect_handler(
                                            effect,
                                            input_elem.clone(),
                                        );
                                    });
                                    e.prevent_default();
                                }
                            }
                            "ArrowRight" => {
                                let text_state = state
                                    .forest_content
                                    .borrow_mut()
                                    .get_mut(&focused_node)
                                    .and_then(RichAuxData::as_text_mut)
                                    .cloned()
                                    .expect("Failed to get text state");

                                let caret_position = text_state.selection.as_caret();
                                if caret_position.is_some()
                                    && caret_position.unwrap().utf8
                                        == focused_node_state.content.len()
                                {
                                    let effects_buffer = state.effects_buffer.clone();
                                    document_operations::move_caret_horizontal(
                                        &mut state,
                                        &mut effects_buffer.borrow_mut(),
                                        focused_node,
                                        CaretDirection::Right,
                                    )
                                    .apply(|effect| {
                                        wasm32_unknown_unknown::effect_handler(
                                            effect,
                                            input_elem.clone(),
                                        );
                                    });
                                    e.prevent_default();
                                }
                            }
                            "ArrowDown" => {
                                e.prevent_default();
                                state = move_caret_vertical(
                                    state,
                                    focused_node,
                                    &input_elem,
                                    CaretDirection::Down,
                                );
                            }
                            "ArrowUp" => {
                                e.prevent_default();
                                state = move_caret_vertical(
                                    state,
                                    focused_node,
                                    &input_elem,
                                    CaretDirection::Up,
                                );
                            }
                            _ => {}
                        },
                        unhandled_case => {
                            console_log(format!("[Warning] Unhandled Case {:?}", unhandled_case));
                        }
                    }
                }
                _ => {
                    console_log("Unhandled cases".into());
                }
            }

            state.trigger_update();

            state_handler.set(state);
        }
    });
    let process_deferred_tasks = {
        let ta_state_handler = state.clone();
        Callback::from(move |event: web_sys::InputEvent| {
            let mut ta_state = ta_state_handler.deref().clone();
            let helpers = ta_state.document_helpers();

            console_log("ON BEFORE INPUT IS BEING FIRED".into());
            console_log(format!("data = \'{}\'", event.data().unwrap_or_default()));
            let incoming_text = event.data().unwrap_or_default();
            if incoming_text.is_empty() {
                return;
            }
            let hidden_input_element = event
                .target()
                .expect("Failed to get target")
                .dyn_into::<HtmlInputElement>()
                .expect("Failed to cast to Input Element");

            let mut accumulated_inserts =
                HashMap::<(Ptr<RichTag>, RichOffset), InlineStyling>::new();

            while let Ok(task) = ta_state.deferred_tasks.dequeue() {
                match task.task_kind {
                    DeferredDocumentTaskKind::InsertNodeAt {
                        node: head_node,
                        offset,
                        src_style, 
                        style_clicked: destination_style,
                    } => {
                        
                        accumulated_inserts
                            .entry((head_node, offset))
                            .and_modify(|styling| *styling ^= destination_style)
                            .or_insert(src_style^destination_style);
                    }
                    DeferredDocumentTaskKind::NOP => {}
                }
            }

            for ((node, offset), accumulated_styles) in accumulated_inserts {
                event.prevent_default();
                console_log(format!("accumulated_styles = {:?}", accumulated_styles));
                inner_operation(
                    &mut ta_state,
                    node,
                    offset,
                    accumulated_styles,
                    incoming_text.clone(),
                    &hidden_input_element,
                );
            }

            ta_state_handler.set(ta_state);
        })
    };

    use_effect_with(input_ref.clone(), {
        let input_ref = input_ref.clone();
        move |_| {
            let input_elem = input_ref
                .cast::<HtmlInputElement>()
                .expect("Failed to cast to HtmlInputElement");
            let closure = wasm_bindgen::closure::Closure::<dyn Fn(_)>::wrap(Box::new(
                move |e: web_sys::InputEvent| process_deferred_tasks.emit(e),
            ));
            let _ = input_elem
                .add_event_listener_with_callback("beforeinput", closure.as_ref().unchecked_ref());

            // CLEANS UP CLOSURE ON DROP
            move || {
                let _ = input_elem.remove_event_listener_with_callback(
                    "beforeinput",
                    closure.as_ref().unchecked_ref(),
                );
                std::mem::drop(closure)
            }
        }
    });

    html! {
        <>
            <rich-text-area>
                <input
                    ref={input_ref.clone()}
                    type="text"
                    oninput={sync_hidden_input_element_to_rich_ast_oninput}
                    onselectionchange={sync_hidden_input_element_to_rich_ast_onselection_change}
                    {onkeydown}
                />

                <tool-container>
                    {for tool_iter}
                </tool-container>
                <text-container>
                    <text-pure data-update-counter={state.update_counter.to_string()}>
                        // START RECURSING AT THE ROOT OF THE AST
                        <RichDisplayNode
                            ta_state={state.clone()}
                            root={state.document}
                            forest={state.forest.clone()}
                            input_ref={input_ref.clone()}
                            overlays_enabled={false}
                        />
                    </text-pure>
                    <text-overlays data-update-counter={state.update_counter.to_string()}>
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
fn inner_operation(
    ta_state: &mut TextAreaState,
    head_node: Ptr<RichTag>,
    offset: RichOffset,
    destination_style: InlineStyling,
    incoming_text: String,
    hidden_input_elem: &HtmlInputElement,
) {
    let helpers = ta_state.document_helpers();
    let line_container = helpers.node_info(head_node, |info| info.parent);

    let head_state = helpers
        .try_get_text_state(head_node, |text_state| text_state.clone())
        .expect("Failed to get styling");
    let head_styling = head_state.styling;
    let head = &head_state.content[0..offset.utf8];
    let tail = &head_state.content[offset.utf8..];

    helpers
        .try_get_text_state_mut(head_node, |text_state| {
            text_state.content.clear();
            text_state.content.push_str(head);
        })
        .expect("Failed to mutate TextState");
    helpers.get_aux_text_state_mut(head_node, |extra_text_state| {
        extra_text_state.selection = TextSelectionKind::Unselected
    });

    let incoming_node = helpers.create_text_node(
        TextState {
            content: incoming_text.clone(),
            styling: destination_style,
        },
        ExtraTextState {
            selection: TextSelectionKind::Unselected,
        },
    );

    let tail_node = helpers.create_text_node(
        TextState {
            content: tail.into(),
            styling: head_styling,
        },
        ExtraTextState {
            selection: TextSelectionKind::Unselected,
        },
    );

    ta_state
        .forest
        .borrow_mut()
        .add_child_at(line_container, incoming_node, head_node, 1);
    ta_state
        .forest
        .borrow_mut()
        .add_child_at(line_container, tail_node, incoming_node, 1);

    if head.is_empty() {
        helpers.delete_node(head_node);
    }
    if tail.is_empty() {
        helpers.delete_node(tail_node);
    }

    hidden_input_elem.set_value(incoming_text.as_str());

    *ta_state.focused_node_mut() = incoming_node;
    *ta_state.document_selection_mut() = Some(RichSelection::mononode(
        incoming_node,
        RichRange::from_caret(RichOffset::zero()),
    ));
}

pub fn create_new_line_at_caret<'a>(
    state: &'a mut TextAreaState,
    focused_node: Ptr<RichTag>,
    focused_node_state: TextState,
    document_selection: RichSelection,
    effect_buffer: &'a mut Vec<Effect>,
) -> EffectRunner<'a> {
    let helpers = state.document_helpers();
    let current_line_container = helpers.node_info(focused_node, |info| info.parent);
    let current_line_children =
        helpers.node_info(current_line_container, |info| info.children.to_vec());
    let focued_node_position_in_line_container = current_line_children
        .iter()
        .enumerate()
        .find(|(_, ptr)| **ptr == focused_node)
        .map(|(idx, _)| idx)
        .expect("Failed to find focused node in container");

    let new_line_container = helpers.create_line_node();

    let CaretSplitContents {
        left: lower_split,
        right: upper_split,
    } = RichDocumentHelpers::split_content_mono(&focused_node_state.content, document_selection)
        .expect("Failed to get monoselection")
        .as_caret_split()
        .expect("Failed to get caret split");

    helpers
        .try_get_text_state_mut(focused_node, |text_state| {
            text_state.content.clear();
            text_state.content.push_str(lower_split);
        })
        .expect("Failed to get text state ");

    let new_text_node = helpers.create_text_node(
        TextState {
            content: upper_split.to_string(),
            styling: focused_node_state.styling,
        },
        ExtraTextState {
            selection: TextSelectionKind::Caret(RichOffset::zero()),
        },
    );

    helpers.forest_mut().add_child_at(
        state.document,
        new_line_container,
        current_line_container,
        1,
    );
    helpers
        .forest_mut()
        .add_child(new_line_container, new_text_node);

    let nodes_to_the_right_of_focused =
        &current_line_children[focued_node_position_in_line_container + 1..];

    // move all nodes to the right of focused into the new line along with the new_text_node
    nodes_to_the_right_of_focused
        .iter()
        .copied()
        .for_each(|node_after_focused| {
            helpers
                .forest_mut()
                .add_child(new_line_container, node_after_focused);
        });

    // All of the nodes to the right of focus are in the new line and are stale
    // in the original line so we delete them
    helpers.node_info_mut(current_line_container, |info| {
        let stale_nodes = nodes_to_the_right_of_focused.len();
        for _ in 0..stale_nodes {
            info.children.pop();
        }
    });

    effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetValue(
        upper_split.to_string(),
    )));
    effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetSelectionRange {
        start: 0,
        end: 0,
    }));

    *state.focused_node_mut() = new_text_node;
    *state.document_selection_mut() = Some(RichSelection::mononode(
        new_text_node,
        RichRange::from_caret(RichOffset::zero()),
    ));
    state.flush_overlays();
    helpers
        .try_set_text_selection(new_text_node, TextSelectionKind::Caret(RichOffset::zero()))
        .expect("Failed to set text data");

    let is_focused_text_node_empty = || lower_split.is_empty();
    let current_line_contains_an_empty_text_node_but_multiple_text_nodes_exist = || {
        is_focused_text_node_empty()
            && helpers.node_info(current_line_container, |info| info.children.len()) > 1
    };

    if current_line_contains_an_empty_text_node_but_multiple_text_nodes_exist() {
        helpers.delete_node(focused_node);
    }

    let is_new_focused_text_node_empty = || upper_split.is_empty();
    let new_line_contains_empty_text_node_but_multiple_text_nodes_exist = || {
        is_new_focused_text_node_empty()
            && helpers.node_info(new_line_container, |info| info.children.len()) > 1
    };
    if new_line_contains_empty_text_node_but_multiple_text_nodes_exist() {
        helpers.delete_node(new_text_node);
        let alt_new_focused_node = helpers
            .node_info(new_line_container, |info| info.children.first().copied())
            .expect("Failed to get first node");

        state.flush_overlays();
        *state.focused_node_mut() = alt_new_focused_node;
        *state.document_selection_mut() = Some(RichSelection::mononode(
            alt_new_focused_node,
            RichRange::default(),
        ));
        helpers
            .try_set_text_selection(
                alt_new_focused_node,
                TextSelectionKind::Caret(RichOffset::zero()),
            )
            .expect("Failed to set selection");
        effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetValue(
            helpers
                .try_get_text_state(alt_new_focused_node, |text_state| {
                    text_state.content.clone()
                })
                .expect("Failed to get content"),
        )));
        effect_buffer.push(Effect::HiddenInput(HiddenInputOp::SetSelectionRange {
            start: 0,
            end: 0,
        }));
    }
    EffectRunner::with_buffer(effect_buffer)
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

    match root_node {
        RichNodeKind::Document => {
            html! {
                <>
                    <rt-document data-ast-id={root.to_string()}>
                        {for child_iterator}
                    </rt-document>
                </>
            }
        }
        RichNodeKind::Text(TextState {
            content: root_text,
            styling: root_styling,
            ..
        }) => {
            let clear_stale_overlays_and_state = {
                let ta_state_handle = ta_state.clone();
                Callback::from(move |_e: MouseEvent| {
                    let mut ta_state = ta_state_handle.deref().clone();
                    console_log("DOCUMENT DUMP:".into());
                    console_log(ta_state.tree_dump());
                    console_log("NOTATION DUMP:".into());
                    console_log(ta_state.to_notation());
                    ta_state.flush_overlays();
                    ta_state.flush_tasks();
                    *ta_state.document_selection_mut() = None;
                    ta_state_handle.set(ta_state);
                })
            };

            // 1. Callback to focus the hidden input when a custom text node is clicked
            let hand_over_control_to_hidden_input = {
                let input_ref = input_ref.clone();
                let state_handler = ta_state.clone();
                Callback::from(move |_e: MouseEvent| {
                    let mut ta_state = state_handler.deref().clone();
                    let focus_options = FocusOptions::new();
                    focus_options.set_prevent_scroll(true);

                    console_log("RT-TEXT ON MOUSEUP FIRED".into());

                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        let clicked_text_node_is_empty = ta_state
                            .forest
                            .borrow()
                            .get_data(root)
                            .as_text()
                            .map(|state| &state.content)
                            .map(String::is_empty)
                            .unwrap_or(false);

                        if clicked_text_node_is_empty {
                            // SKIP `COMPUTE_DOCUMENT_SELECTION(..)`
                            // Note:
                            // I'm trying to remember *why* i do this...
                            // I think its because `compute_document_selection` grabs
                            // weird nodes, but in order for me to actually confirm i'd need to
                            // remove this skip-condition and I just kinda wanna move on.
                            // Note:
                            // Yeah but im pretty sure im right. As of writing this comment
                            // an "Empty" node shoudn't have a text node, so when we click on
                            // the empty `rt-text` node, `compute_document_select(..)` grabs nodes that
                            // aren't relevant for what this editor is trying to do.
                            let _ = input.focus_with_options(&focus_options);
                            input.set_value("");
                            if let Some(state) = ta_state
                                .forest_content
                                .borrow_mut()
                                .get_mut(&root)
                                .and_then(RichAuxData::as_text_mut)
                            {
                                state.selection = TextSelectionKind::Caret(RichOffset::zero());
                            }
                        } else if let Ok(selection) = compute_document_selection() {
                            match selection.kind() {
                                RichSelectionKind::MonoNode(character_range) => {
                                    let node = selection.start_node;
                                    let node_state = ta_state
                                        .forest
                                        .borrow()
                                        .get_data(node)
                                        .as_text()
                                        .cloned()
                                        .expect("Failed to get node state");
                                    if let Some(state) = ta_state
                                        .forest_content
                                        .borrow_mut()
                                        .get_mut(&node)
                                        .and_then(RichAuxData::as_text_mut)
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

                                        input.set_value(&node_state.content);
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
                                    update_toolbar_styles(&mut ta_state, node_state.styling);
                                }
                                RichSelectionKind::MultiNode => {
                                    apply_highlights(&ta_state, selection);
                                    update_toolbar_to_display_common_styles_multinode(
                                        &mut ta_state,
                                        &selection,
                                    );
                                    ta_state.trigger_update();
                                }
                            }
                            *ta_state.document_selection_mut() = Some(selection);

                            let _ = input.focus_with_options(&focus_options);
                        }

                        *ta_state.focused_node_mut() = root;
                        state_handler.set(ta_state);
                    }
                })
            };

            let text_state = ta_state
                .forest_content
                .borrow()
                .get(&root)
                .and_then(RichAuxData::as_text)
                .cloned()
                .expect("Failed to fetch Text state");
            let mut aux_classes = String::new();
            if root_text.is_empty() {
                aux_classes.push_str(" expand_empty ");
            }

            if !p.overlays_enabled {
                // NOTE: SHOULD DUMP SIMILAR HTML TO `TextSelectionKind::Unselected`
                return apply_styles(root_styling, move || {
                    html! {
                        <>
                            <rt-text
                                data-ast-id={root.to_string()}
                                onmousedown={clear_stale_overlays_and_state.clone()}
                                onmouseup={hand_over_control_to_hidden_input.clone()}
                                class={aux_classes.clone()}
                            >
                                {root_text.clone()}
                            </rt-text>
                        </>
                    }
                });
            }

            match text_state.selection {
                TextSelectionKind::Unselected => apply_styles(root_styling, move || {
                    html! {
                        <>
                            <rt-text
                                data-ast-id={root.to_string()}
                                class={aux_classes.clone()}
                            >
                                {root_text.clone()}
                            </rt-text>
                        </>
                    }
                }),
                TextSelectionKind::Caret(split_offset) => apply_styles(root_styling, move || {
                    let split_idx = split_offset.utf8;
                    let lower = &root_text[0..split_idx];
                    let upper = &root_text[split_idx..];

                    html! {
                        <>
                            <rt-text data-ast-id={root.to_string()}>
                                <rt-caret>{lower}</rt-caret>{upper}
                            </rt-text>
                        </>
                    }
                }),
                TextSelectionKind::Range(range) => apply_styles(root_styling, move || {
                    let head = &root_text[0..range.start.utf8];
                    let selected = &root_text[range.start.utf8..range.end.utf8];
                    let tail = &root_text[range.end.utf8..];
                    html! {
                        <>
                            <rt-text data-ast-id={root.to_string()}>
                                {head}<rt-selection>{selected}</rt-selection>{tail}
                            </rt-text>
                        </>
                    }
                }),
            }
        }
        RichNodeKind::Line => {
            html! {
                <>
                    <rt-line data-ast-id={root.to_string()}>
                        {for child_iterator}
                    </rt-line>
                </>
            }
        }
    }
}
pub fn apply_highlights(state: &TextAreaState, selection: RichSelection) {
    let helpers = state.document_helpers();
    let highlight_start_node = || {
        let end_offset = helpers
            .try_get_text_state(selection.start_node, TextState::get_end_offset)
            .expect("Failed to get text state");
        helpers
            .get_aux_text_state_mut(selection.start_node, |extra_text_state| {
                extra_text_state
                    .set_range_selection_with_offsets(selection.start_offset, end_offset)
            })
            .expect("Failed to get aux text state");
    };
    let highligh_end_node = || {
        helpers
            .get_aux_text_state_mut(selection.end_node, |extra_text_state| {
                extra_text_state
                    .set_range_selection_with_offsets(RichOffset::zero(), selection.end_offset);
            })
            .expect("Failed to get aux text state");
    };
    let highlight_in_between_start_and_finish = || {
        let start_node = selection.start_node;
        let end_node = selection.end_node;
        let mut forest_content = state.forest_content.borrow_mut();

        state
            .forest
            .borrow_mut()
            .iter_exclusive_mut(state.document, start_node, end_node)
            .for_each(|(root, data)| {
                let full_text_range = data
                    .as_text()
                    .map(TextState::get_full_range)
                    .unwrap_or(RichRange::default());

                // NOTE: DONT SET SELECTIONS FOR EMPTY NODES IT MESSES UP
                // OVERLAY HIGHLIGHTS
                let is_empty_node = data
                    .as_text()
                    .map(|text_state| text_state.content.is_empty())
                    .unwrap_or(false);

                if !is_empty_node
                    && let Some(extra_ts) = forest_content
                        .get_mut(&root)
                        .and_then(RichAuxData::as_text_mut)
                {
                    extra_ts.set_range_selection(full_text_range);
                }
            });
    };
    highlight_start_node();
    highligh_end_node();
    highlight_in_between_start_and_finish();
}
#[track_caller]
fn compute_document_selection() -> Result<RichSelection, wasm_bindgen::JsValue> {
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

    let instantiation_point = Location::caller();
    Ok(RichSelection {
        start_node: start_rich_ast,
        end_node: end_rich_ast,
        start_offset,
        end_offset,
        debug_info: Some(RichSelectionDebugInfo {
            instantiation_point,
        }),
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

fn apply_styles<F>(styles: InlineStyling, lazy_base: F) -> yew::Html
where
    F: Fn() -> yew::Html,
{
    let mut result = lazy_base();
    for flag in InlineStyling::FLAGS.iter() {
        let available_style = *flag.value();
        if styles.contains(available_style) {
            result = match available_style {
                InlineStyling::BOLD => html! { <rt-bold>{result}</rt-bold> },
                InlineStyling::ITALICS => html! { <rt-italics>{result}</rt-italics>},
                InlineStyling::SPOILERS => html! { <rt-spoilers>{result}</rt-spoilers>},
                InlineStyling::UNDERLINE => html! { <rt-underline>{result}</rt-underline> },
                InlineStyling::UNSTYLED => result,
                _ => panic!("Unknown style"),
            };
        }
    }
    result
}

fn move_caret_vertical(
    mut state: TextAreaState,
    focused_node: Ptr<RichTag>,
    input_elem: &HtmlInputElement,
    query_direction: CaretDirection,
) -> TextAreaState {
    let extra_focused_node_state = state
        .forest_content
        .borrow()
        .get(&focused_node)
        .and_then(RichAuxData::as_text)
        .cloned()
        .expect("Failed to fetch text state from currently focused node");
    let caret_position = extra_focused_node_state
        .selection
        .as_caret()
        .expect("Failed to get caret offset");

    let rt_text_elem = find_rich_text_node_in_dom(focused_node);
    let rt_text_text_node = rt_text_elem
        .child_nodes()
        .get(0)
        .expect("Failed to get the #text node inside of the rt-text element");

    let (query_point_x, query_point_y) =
        compute_query_point_from_text_node(&rt_text_text_node, caret_position, query_direction);

    if let Some(query_element) = element_from_point(query_point_x, query_point_y) {
        console_log(format!("Element found: \"{}\"", query_element.outer_html()));
        console_log(format!(
            "query_element name = \"{}\"",
            query_element.node_name()
        ));

        let valid_query_element_name =
            |query_element: &web_sys::Element| query_element.node_name() == "RT-TEXT";
        if !valid_query_element_name(&query_element) {
            console_log("Vertical caret movement cancelled".into());
            return state;
        }

        let text_node = query_element
            .child_nodes()
            .get(0)
            .expect("Failed to get text_node");

        let caret_position = get_caret_position_from_point(text_node, query_point_x, query_point_y);

        let find_rich_node_ptr_from_element = |query_element: web_sys::Element| {
            Ptr::<RichTag>::from(
                query_element
                    .get_attribute("data-ast-id")
                    .expect("Failed to get ast-id")
                    .parse::<usize>()
                    .expect("Failed to parse Id"),
            )
        };

        let vertically_adjacent_node = find_rich_node_ptr_from_element(query_element);
        *state.focused_node_mut() = vertically_adjacent_node;
        *state.document_selection_mut() = Some(RichSelection::mononode(
            vertically_adjacent_node,
            RichRange::from_caret(caret_position),
        ));
        state.flush_overlays();
        if let Some(extra_state) = state
            .forest_content
            .borrow_mut()
            .get_mut(&vertically_adjacent_node)
            .and_then(RichAuxData::as_text_mut)
        {
            extra_state.selection = TextSelectionKind::Caret(caret_position);
        }

        let vertically_adjacent_text_state = state
            .forest
            .borrow()
            .get_data(vertically_adjacent_node)
            .as_text()
            .cloned()
            .expect("Failed to get text node");
        input_elem.set_value(&vertically_adjacent_text_state.content);
        input_elem
            .set_selection_range(caret_position.utf16 as u32, caret_position.utf16 as u32)
            .expect("Failed to set selection range");

        console_log("Vertical caret movement, completed".into());
    } else {
        console_log("Point query found nothing".into());
    }
    state
}

pub fn compute_query_point_from_text_node(
    text_node: &web_sys::Node,
    caret_position: RichOffset,
    direction: CaretDirection,
) -> (f32, f32) {
    let wrapped_lines = compute_wrapped_lines_in_text_node(text_node);
    let text = text_node
        .text_content()
        .expect("Failed to get text content");

    let mut caret_is_on_broken_whitespace = false;
    for RichRange { start, end } in wrapped_lines {
        let char_list = text[start.utf8..end.utf8].chars().collect::<Vec<_>>();
        let mut cursor = start;
        for (c_idx, &c) in char_list.iter().enumerate() {
            cursor.offset_by(c);
            if caret_position == cursor && c.is_whitespace() && c_idx == char_list.len() - 1 {
                caret_is_on_broken_whitespace = true;
                break;
            }
        }
        if caret_is_on_broken_whitespace {
            break;
        }
    }

    let range = web_sys::Range::new().expect("Failed to create range");

    range
        .set_start(text_node, caret_position.utf16 as u32)
        .expect("Failed to set start range");

    if caret_is_on_broken_whitespace
        && let Some(adjacent_char) = text[caret_position.utf8..].chars().nth(1)
    {
        let mut end = caret_position;
        end.offset_by(adjacent_char);

        range
            .set_end(text_node, end.utf16 as u32)
            .expect("Failed to set end range");
    } else {
        range
            .set_end(text_node, caret_position.utf16 as u32)
            .expect("Failed to set end range");
    }

    let mut cur_caret_rect = RichRect::from(&range.get_bounding_client_rect());

    console_log(format!("query_point caret_rect = {}", cur_caret_rect));

    if cur_caret_rect.h < f32::EPSILON {
        if let Some(text_container) = text_node
            .parent_node()
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        {
            let dom_aabb = text_container.get_bounding_client_rect();
            // FALLBACK RECT FOR WHEN THE TEXT_NODE RECT HAS NO HEIGHT
            // ADDED BECAUSE EMPTY LINES HAVE NO HEIGHT SOMETIMES
            cur_caret_rect = RichRect {
                x: dom_aabb.x() as f32,
                y: dom_aabb.y() as f32,
                w: 0.0,
                h: dom_aabb.height() as f32,
            };
        } else {
            panic!("Warning: Container element not found!");
        }
    }
    let caret_center_x = cur_caret_rect.x + cur_caret_rect.w * 0.5;
    let caret_center_y = cur_caret_rect.y + cur_caret_rect.h * 0.5;
    let offset_y = cur_caret_rect.h * direction.to_sign_vertical();
    (caret_center_x, caret_center_y + offset_y)
}
pub fn element_from_point(x: f32, y: f32) -> Option<web_sys::Element> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("Failed to get document");
    document.element_from_point(x, y)
}

pub fn find_rich_text_node_in_dom(focused_node: Ptr<RichTag>) -> web_sys::Element {
    let window = web_sys::window().expect("Failed to get window");
    let document = window.document().expect("Failed to get document");
    let focused_node_query = format!(
        "rich-text-area > text-container > text-pure rt-text[data-ast-id=\"{}\"]",
        focused_node.to_index()
    );

    console_log(format!("Executing Query: \"{}\"", focused_node_query));
    let results = document
        .query_selector(&focused_node_query)
        .expect("Failed to execute query");
    results.expect("Failed to find rt-text node in the dom")
}

pub fn get_caret_position_from_point(
    text_node: web_sys::Node,
    query_point_x: f32,
    query_point_y: f32,
) -> RichOffset {
    let text = text_node
        .text_content()
        .expect("Failed to get text content");

    let wrapped_lines = compute_wrapped_lines_in_text_node(&text_node);
    let range = Range::new().unwrap();

    let starting_row_aabb = wrapped_lines
        .first()
        .map(|&RichRange { start, end }| {
            range.set_start(&text_node, start.utf16 as u32).unwrap();
            range.set_end(&text_node, end.utf16 as u32).unwrap();
            RichRect::from(&range.get_bounding_client_rect())
        })
        .expect("Failed to get first row");

    let target_row_idx =
        ((query_point_y - starting_row_aabb.y).abs() / starting_row_aabb.h) as usize;

    // print all rows
    for RichRange { start, end } in wrapped_lines.iter().copied() {
        range.set_start(&text_node, start.utf16 as u32).unwrap();
        range.set_end(&text_node, end.utf16 as u32).unwrap();
        let aabb = RichRect::from(&range.get_bounding_client_rect());
        console_log(format!(
            "Row: {} aabb:{}",
            &text[start.utf8..end.utf8],
            aabb
        ));
    }

    console_log(format!(
        "Query point = (x:{},y:{})",
        query_point_x, query_point_y
    ));
    console_log(format!("target_row_idx = {}", target_row_idx));

    let mut min_distance = f32::INFINITY;
    let mut best = RichOffset::zero();
    for (current_row_idx, RichRange { start, end }) in wrapped_lines.iter().copied().enumerate() {
        let row_text = &text[start.utf8..end.utf8];
        let mut end = start;
        range.set_start(&text_node, start.utf16 as u32).unwrap();
        let char_list = row_text.chars().collect::<Vec<_>>();
        for (char_pos, &c) in char_list.iter().enumerate() {
            end.offset_by(c);
            range.set_end(&text_node, end.utf16 as u32).unwrap();
            let aabb = RichRect::from(&range.get_bounding_client_rect());
            let (east_mid_x, east_mid_y) = aabb.compute_east_midpoint();
            let dx = east_mid_x - query_point_x;
            let dy = east_mid_y - query_point_y;
            let dist = dx * dx + dy * dy;
            if dist < min_distance && target_row_idx == current_row_idx {
                if char_pos == char_list.len() - 1 && c.is_whitespace() {
                    // This particular character is NOT rendered
                    // in the browser, processing it, results in jumps
                    // that make no visual sense if we jump to it.
                } else {
                    min_distance = dist;
                    best = end;
                }
            }
            /*if current_row_idx == target_row_idx {
                console_log(format!("aabb = {}", aabb));
                console_log(format!("current_row = \"{}\"", range.to_string()));
                console_log(format!("distance = {}", dist));
            }*/
        }
    }

    console_log(format!("final min_distance ={}", min_distance));

    best
}
fn compute_wrapped_lines_in_text_node(text_node: &web_sys::Node) -> Vec<RichRange> {
    let text = text_node
        .text_content()
        .expect("Failed to get text content");

    let range = web_sys::Range::new().expect("Failed to create range");
    range
        .set_start(text_node, 0)
        .expect("Failed to set range start");
    range.set_end(text_node, 0).expect("Faield to set end");

    // mainly computed for the dimensions, we need to know
    // how "tall" in pixels a single character is AND we need to know where the top-left corner of
    // the text begins. The single range query is used for that
    let caret_rect = RichRect::from(&range.get_bounding_client_rect());

    let mut start = RichOffset::zero();
    let mut end = RichOffset::zero();
    let mut prev_end = RichOffset::zero();
    let line_height = caret_rect.h;

    console_log(format!("Starting line height is:{}", line_height));
    console_log(format!("caret_rect = {}", caret_rect));

    let mut wrapped_lines: Vec<_> = vec![];
    for c in text.chars() {
        range.set_start(text_node, start.utf16 as u32).unwrap();
        range
            .set_end(text_node, end.utf16 as u32)
            .expect("Failed to set range end");
        let aabb = RichRect::from(&range.get_bounding_client_rect());
        if (aabb.h - line_height).abs() > 1.0 {
            wrapped_lines.push(RichRange {
                start,
                end: prev_end,
            });
            start = prev_end;
            range
                .set_start(text_node, start.utf16 as u32)
                .expect("Failed to update start range");
            range
                .set_end(text_node, end.utf16 as u32)
                .expect("Failed to update end range");
        }
        prev_end = end;
        end.offset_by(c);
    }
    //flush last line
    wrapped_lines.push(RichRange { start, end });
    wrapped_lines
}

pub fn update_toolbar_to_display_common_styles_multinode(
    ta_state: &mut TextAreaState,
    selection: &RichSelection,
) {
    let common_selected_styling = compute_common_stylings_in_selection(ta_state, selection);
    update_toolbar_styles(ta_state, common_selected_styling);
    console_log(format!("common styles = {:?}", common_selected_styling));
}

#[derive(Copy, Clone, PartialEq)]
pub struct RichRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl RichRect {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.x -= padding;
        self.y -= padding;
        self.w += padding;
        self.h += padding;
        self
    }
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let rx = x - self.x;
        let ry = y - self.y;
        rx > 0.0 && ry > 0.0 && rx < self.w && ry < self.h
    }
    pub fn compute_east_midpoint(&self) -> (f32, f32) {
        (self.x + self.w, self.y + self.h * 0.5)
    }
}
impl Default for RichRect {
    fn default() -> Self {
        Self::new()
    }
}
impl From<&web_sys::DomRect> for RichRect {
    fn from(value: &web_sys::DomRect) -> Self {
        Self {
            x: value.x() as f32,
            y: value.y() as f32,
            w: value.width() as f32,
            h: value.height() as f32,
        }
    }
}
impl std::fmt::Display for RichRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x:{},y:{},w:{},h:{}]", self.x, self.y, self.w, self.h)
    }
}

pub mod wasm32_unknown_unknown {
    use super::*;
    pub fn effect_handler(effect: &Effect, hidden_input_elem: HtmlInputElement) {
        match effect {
            Effect::HiddenInput(op_kind) => match op_kind {
                &HiddenInputOp::SetSelectionRange { start, end } => {
                    let _ = hidden_input_elem.set_selection_range(start, end);
                }
                HiddenInputOp::SetValue(val) => {
                    hidden_input_elem.set_value(val);
                }
                HiddenInputOp::Focus {
                    prevent_scroll,
                    focus_visible,
                } => {
                    let options = FocusOptions::new();
                    options.set_prevent_scroll(*prevent_scroll);
                    options.set_focus_visible(*focus_visible);
                    hidden_input_elem
                        .focus_with_options(&options)
                        .expect("Failed to focuse");
                }
            },
            Effect::ConsoleLog(message) => {
                console_log(message.into());
            }
        }
    }
}
