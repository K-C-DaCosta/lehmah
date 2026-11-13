use super::*;

struct Harness {
    ta_state: TextAreaState,
}
impl Harness {
    fn sample() -> Self {
        let mut area = TextAreaState::new().with_sample_document();
        let document = area.document;
        let helpers = area.document_helpers();
        let (local_focused_node, caret_range) =
            Self::find_inital_text_node_with_caret(document, &helpers);
        let local_document_selection = RichSelection::mononode(local_focused_node, caret_range);
        *area.focused_node_mut() = local_focused_node;
        *area.document_selection_mut() = Some(local_document_selection);

        Self { ta_state: area }
    }

    pub fn enter(&mut self) {
        let ta_state = &mut self.ta_state;
        let local_focused_node = *ta_state.focused_node();
        let local_document_selection = ta_state
            .document_selection()
            .expect("Failed to get document selection");
        let runner = ta_state.runner(|_| None);
        document_operations::create_new_line_at_caret(
            ta_state,
            local_focused_node,
            local_document_selection,
            runner,
        );
    }

    fn find_inital_text_node_with_caret(
        document: Ptr<RichTag>,
        helpers: &RichDocumentHelpers,
    ) -> (Ptr<RichTag>, RichRange) {
        // find the text node initalized with the caret
        // ususally the user clicks the node and is the node comes into focus that way
        // but since were outside the browser searching for the node with the caret
        // is pretty much the best thing we can do
        let text_node_with_caret = helpers
            .forest()
            .iter(document)
            .find(|(node, kind)| {
                kind.is_text_kind()
                    && helpers
                        .forest_content()
                        .get(node)
                        .unwrap()
                        .as_text()
                        .and_then(|e| e.selection.as_caret())
                        .is_some()
            })
            .map(|(ptr, _)| ptr)
            .expect("Failed to find the inital text node with the caret");

        let caret_range = helpers
            .try_get_extra_text_state(text_node_with_caret, |extra_text_state| {
                extra_text_state.selection
            })
            .expect("Failed to get selection")
            .to_range_extended();
        (text_node_with_caret, caret_range)
    }
    fn notation(&self) -> String {
        self.ta_state.to_notation()
    }
}

pub fn to_flat<T: AsRef<[&'static str]>>(lines: T) -> String {
    lines
        .as_ref()
        .iter()
        .map(|line| format!("{}\n", line.trim()))
        .collect()
}

#[test]
fn sample_document() {
    let mut e = Harness::sample();
    #[rustfmt::skip]
        assert_eq!(e.notation(), to_flat([
            "<b>Hello</b>|<b> World</b>"
        ]));
}

#[test]
fn enter_line_at_caret() {
    let mut e = Harness::sample();
    e.enter();

    #[rustfmt::skip]
        assert_eq!(
            e.notation(),
            to_flat([
            "<b>Hello</b>", 
            "|<b> World</b>"
            ])
        );
}

#[test]
fn enter_line_at_caret_three_times() {
    let mut e = Harness::sample();
    e.enter();
    e.enter();
    e.enter();

    #[rustfmt::skip]
        assert_eq!(
            e.notation(),
            to_flat([
            "<b>Hello</b>",
            "<b>∅</b>",
            "<b>∅</b>",
            "|<b> World</b>"
            ])
        );
}
