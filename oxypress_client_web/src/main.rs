use oxypress_client_web::DomAddressable;
use oxypress_core::DomPointer;
use web_sys::{
    console,
    js_sys::{self, Function},
    wasm_bindgen::{prelude::Closure, JsCast, JsValue},
    Document, Element,
};
use yew::prelude::*;

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
fn App() -> Html {
    let state = use_state(|| DocumentEditorState::new());
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
        console_log(format!("{:?}", body_frame_elem.outer_html()));

        body_frame_elem
            .add_event_listener_with_callback(
                "select",
                Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
                    console_log("TEXT SELECTED!".into());
                })
                .as_ref()
                .unchecked_ref(),
            )
            .unwrap();
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

fn main() {
    yew::Renderer::<App>::new().render();
}
