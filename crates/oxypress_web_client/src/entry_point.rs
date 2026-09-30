use super::dom_pointer::*;
use bitflags::bitflags;
use oxypress_core::DomPointer;
use web_sys::{
    Document, Element, NodeFilter, TreeWalker, console,
    js_sys::{self, Function},
    wasm_bindgen::{self, JsCast, JsValue, prelude::Closure},
};
use yew::prelude::*;

// The bitflags macro handles the struct definition and operator overloading.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct NodeFilterFlags: u32 {
        const SHOW_ALL = 0xFFFFFFFF;
        const SHOW_ELEMENT = 0x1;
        const SHOW_ATTRIBUTE = 0x2;
        const SHOW_TEXT = 0x4;
        const SHOW_CDATA_SECTION = 0x8;
        const SHOW_ENTITY_REFERENCE = 0x10;
        const SHOW_ENTITY = 0x20;
        const SHOW_PROCESSING_INSTRUCTION = 0x40;
        const SHOW_COMMENT = 0x80;
        const SHOW_DOCUMENT = 0x100;
        const SHOW_DOCUMENT_TYPE = 0x200;
        const SHOW_DOCUMENT_FRAGMENT = 0x400;
        const SHOW_NOTATION = 0x800;
    }
}
pub enum FilterResult {
    Accept = 1,
    Reject = 2,
    Skip = 3,
}

impl From<NodeFilterFlags> for u32 {
    fn from(value: NodeFilterFlags) -> Self {
        value.bits()
    }
}

struct DocumentEditorState {
    current_selected_node: DomPointer,
}

impl DocumentEditorState {
    pub fn new() -> Self {
        Self {
            current_selected_node: DomPointer::new(),
        }
    }
}

fn console_log(text: String) {
    console::log_1(&JsValue::from_str(&text));
}

#[function_component]
pub fn App() -> Html {
    let state = use_state(DocumentEditorState::new);
    let onclick = {};
    let filler_text = "foo bar";
    let bold_tool_text = "B";

    use_effect(move || {
        let body_frame_elem = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                document
                    .query_selector("document-body-frame")
                    .ok()
                    .flatten()
            })
            .unwrap();
        //console_log(format!("{:?}", body_frame_elem.outer_html()));
        //let onselect = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        //    console_log("TEXT SELECTED!".into());
        //});
        //body_frame_elem
        //    .add_event_listener_with_callback("select", onselect.as_ref().unchecked_ref())
        //    .unwrap();

        let bold_tool_elem = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector(".bold-tool").ok().flatten())
            .unwrap();

        let bold_onclick_closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
            event.prevent_default();
            console_log("BOLD CLICKED!!!".into());
            let selection = web_sys::window()
                .unwrap()
                .get_selection()
                .ok()
                .flatten()
                .unwrap();

            let selection_direction =
                js_sys::Reflect::get(selection.unchecked_ref(), &"direction".into())
                    .unwrap()
                    .as_string()
                    .unwrap();
            let selection_type = selection.type_();

            console_log(format!(
                "LOG: sel direction = {}, sel type = {} ",
                selection_direction, selection_type
            ));

            if !(selection_direction == "forward" || selection_direction == "none") {
                console_log("Warning:backwards selection isn't supported".into());
                return;
            }

            let start = selection.anchor_node().unwrap();
            let start_offset = selection.anchor_offset();
            let end = selection.focus_node().unwrap();
            let end_offset = selection.focus_offset();

            let domptr_start = DomPointer::addr_of(&start);
            let domptr_end = DomPointer::addr_of(&end);
            let start_and_end_are_same = start.is_same_node(Some(&end));

            let walker_selection = selection.clone();
            let walker_filer = Closure::<dyn FnMut(_) -> u32>::new(move |n: web_sys::Node| {
                let result = walker_selection
                    .contains_node(&n)
                    .ok()
                    .and_then(|val| val.then_some(FilterResult::Accept))
                    .unwrap_or(FilterResult::Reject);
                result as u32
            });

            let walker = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_tree_walker_with_what_to_show_and_filter(
                    start.parent_node().as_ref().unwrap(),
                    NodeFilterFlags::SHOW_ALL.into(),
                    Some(walker_filer.as_ref().unchecked_ref()),
                )
                .unwrap();

            // FIXME: This is a memory leak, find a better way to handle the filter
            walker_filer.forget();

            console_log(format!(
                "start = {:?}",
                domptr_start.resolve().unwrap().text_content()
            ));
            console_log(format!(
                "end = {:?}",
                domptr_end.resolve().unwrap().text_content()
            ));

            if start_and_end_are_same {
                console_log("Selection Detected, Starts and Ends are the same".into());

                //check if already bolded then unbold
                if start.parent_node().unwrap().node_name().to_uppercase() == "STRONG" {
                } else {
                    end.unchecked_ref::<web_sys::Text>()
                        .split_text(end_offset)
                        .unwrap();
                    let extracted_text_node = start
                        .unchecked_ref::<web_sys::Text>()
                        .split_text(start_offset)
                        .unwrap();

                    let document = web_sys::window().and_then(|w| w.document()).unwrap();
                    let strong_node = document.create_element("strong").unwrap();
                    strong_node.append_child(&extracted_text_node).unwrap();

                    end.parent_node()
                        .unwrap()
                        .insert_before(&strong_node, end.next_sibling().as_ref())
                        .unwrap();
                }
            }
        });

        bold_tool_elem
            .add_event_listener_with_callback(
                "click",
                bold_onclick_closure.as_ref().unchecked_ref(),
            )
            .unwrap();
        bold_onclick_closure.forget();
    });

    html! {
        <document-editor-frame>
            <document-editor-tools>
                <editor-tool class="bold-tool">
                    <gg-container>
                        <strong>{bold_tool_text}</strong>
                    </gg-container>
                </editor-tool>
                <editor-tool class="italics-tool">
                     <gg-container>
                         <a class="gg-format-italic" style="--ggs:1"/>
                     </gg-container>
                </editor-tool>
                <editor-tool class="underline-tool">
                    <gg-container>
                        <a class="gg-format-underline"/>
                    </gg-container>
                </editor-tool>
                <editor-tool class="list-tool">
                    <gg-container>
                        <a class="gg-layout-list"/>
                    </gg-container>
                </editor-tool>
            </document-editor-tools>
            <document-body-frame contenteditable="">

            </document-body-frame>
        </document-editor-frame>
    }
}
