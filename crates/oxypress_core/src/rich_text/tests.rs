use super::*;

struct Harness {
    ta_state: TextAreaState,
}
impl Harness {
    fn bold_line() -> Self {
        let mut area = TextAreaState::new().with_sample_document();
        Self { ta_state: area }
    }
    fn empty_line() -> Self {
        let mut area = TextAreaState::new();
        let helpers = area.document_helpers();
        /* Hardcoded AST:
         * --------------
         *  Document
         *  ..Line
         *  ....Text('')
         *  -------------
         * */
        let sample_text = "";
        let document = helpers.create_document_node();
        let line_container = helpers.create_line_node();
        let text_node = helpers.create_text_node(
            TextState {
                content: String::from(sample_text),
                styling: InlineStyling::UNSTYLED,
            },
            ExtraTextState {
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset(sample_text, 0)),
            },
        );
        helpers.forest_mut().add_child(document, line_container);
        helpers.forest_mut().add_child(line_container, text_node);
        area.document = document;
        Self { ta_state: area }
    }
    fn multiple_styles_in_on_line() -> Self {
        let mut area = TextAreaState::new();
        let helpers = area.document_helpers();
        /* Hardcoded AST:
         * --------------
         *  Document
         *  ..Line
         *  ....Text('Hel')
         *  ....Text('lo Wo')
         *  ....Text('rld')
         *  -------------
         * */
        let sample_text = "Hello World";
        let document = helpers.create_document_node();
        let line_container = helpers.create_line_node();
        let left_text_node = helpers.create_text_node(
            TextState {
                content: String::from(&sample_text[0..3]),
                styling: InlineStyling::UNSTYLED,
            },
            ExtraTextState {
                selection: TextSelectionKind::Unselected,
            },
        );
        let middle_text_node = helpers.create_text_node(
            TextState {
                content: String::from(&sample_text[3..8]),
                styling: InlineStyling::ITALICS,
            },
            ExtraTextState {
                selection: TextSelectionKind::Unselected,
            },
        );
        let tail_text_node = helpers.create_text_node(
            TextState {
                content: String::from(&sample_text[8..]),
                styling: InlineStyling::BOLD,
            },
            ExtraTextState {
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset(sample_text, 2)),
            },
        );

        helpers.forest_mut().add_child(document, line_container);
        helpers
            .forest_mut()
            .add_child(line_container, left_text_node);
        helpers
            .forest_mut()
            .add_child(line_container, middle_text_node);
        helpers
            .forest_mut()
            .add_child(line_container, tail_text_node);

        area.document = document;
        Self { ta_state: area }
    }
    fn plain_line() -> Self {
        let mut area = TextAreaState::new();
        let helpers = area.document_helpers();
        /* Hardcoded AST:
         * --------------
         *  Document
         *  ..Line
         *  ....Text('HELLO WORLD')
         *  -------------
         * */
        let sample_text = "Hello World";
        let document = helpers.create_document_node();
        let line_container = helpers.create_line_node();
        let text_node = helpers.create_text_node(
            TextState {
                content: String::from(sample_text),
                styling: InlineStyling::UNSTYLED,
            },
            ExtraTextState {
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset(sample_text, 5)),
            },
        );
        helpers.forest_mut().add_child(document, line_container);
        helpers.forest_mut().add_child(line_container, text_node);
        area.document = document;
        Self { ta_state: area }
    }
    fn split_line() -> Self {
        let mut area = TextAreaState::new();
        let helpers = area.document_helpers();

        let sample_text = "Hello World";
        let document = helpers.create_document_node();
        let line_container = helpers.create_line_node();
        let text_node = helpers.create_text_node(
            TextState {
                content: String::from(&sample_text[0..sample_text.len() - 1]),
                styling: InlineStyling::UNSTYLED,
            },
            ExtraTextState {
                selection: TextSelectionKind::Unselected,
            },
        );
        let right_node = helpers.create_text_node(
            TextState {
                content: String::from(&sample_text[sample_text.len() - 1..]),
                styling: InlineStyling::BOLD,
            },
            ExtraTextState {
                selection: TextSelectionKind::Caret(RichOffset::from_utf8_offset(sample_text, 1)),
            },
        );

        helpers.forest_mut().add_child(document, line_container);
        helpers.forest_mut().add_child(line_container, text_node);
        helpers.forest_mut().add_child(line_container, right_node);
        area.document = document;
        Self { ta_state: area }
    }
    pub fn click_on_caret(&mut self) {
        let area = &mut self.ta_state;
        let document = area.document;
        let helpers = area.document_helpers();
        let (local_focused_node, caret_range) = Self::find_text_node_with_caret(document, &helpers);
        let local_document_selection = RichSelection::mononode(local_focused_node, caret_range);
        *area.focused_node_mut() = local_focused_node;
        *area.document_selection_mut() = Some(local_document_selection);
    }

    // # Description
    // this function simulates what happens when you press
    // backspace on an `HtmlInputElement`, it only cares
    // about how the caret moves WITHIN the node
    pub fn simulate_hidden_input_backspace<F: Fn(&mut Self, &mut bool)>(
        &mut self,
        before_backspace: F,
    ) {
        let mut prevent_default = false;
        before_backspace(self, &mut prevent_default);
        if prevent_default {
            return;
        }

        let ta_state = &mut self.ta_state;
        let focused_node = *ta_state.focused_node();
        let helpers = ta_state.document_helpers();

        let caret = helpers
            .try_get_extra_text_state(focused_node, |extra_text_state| extra_text_state.selection)
            .expect("Failed to get text selection")
            .as_caret()
            .expect("Expected a caret selection");

        if caret.utf8 == 0 {
            return;
        }

        let mut content = helpers
            .try_get_text_state(focused_node, |text_state| text_state.content.clone())
            .expect("Failed to get text state");

        let previous_character_start = content[..caret.utf8]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .expect("Caret is not past the start of the content");

        content.replace_range(previous_character_start..caret.utf8, "");
        let caret = RichOffset::from_utf8_offset(&content, previous_character_start);

        helpers
            .try_get_text_state_mut(focused_node, |text_state| text_state.content = content)
            .expect("Failed to set text state");
        helpers
            .try_set_text_selection(focused_node, TextSelectionKind::Caret(caret))
            .expect("Failed to set text selection");
        *ta_state.document_selection_mut() = Some(RichSelection::mononode(
            focused_node,
            RichRange::from_caret(caret),
        ));
    }

    pub fn backspace(&mut self) {
        self.simulate_hidden_input_backspace(|state, prevent_default| {
            //AI-AGENT NOTE: Handle all backspace logic here
        });
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

    fn find_text_node_with_caret(
        document: Ptr<RichTag>,
        helpers: &RichDocumentHelpers,
    ) -> (Ptr<RichTag>, RichRange) {
        // ususally the user clicks the node and nodes are focused in an JS/GUI event
        // handler but since we're not in a GUI enviroment this is pretty much the
        // next best thing we can do
        let (text_node_with_caret, _) = helpers
            .forest()
            .iter(document)
            .find(|(node, kind)| {
                helpers
                    .forest_content()
                    .get(node)
                    .and_then(RichAuxData::as_text)
                    .and_then(|ets| ets.selection.as_caret())
                    .is_some()
            })
            .expect("Failed to find text node with caret");

        let caret_range = helpers
            .try_get_extra_text_state(text_node_with_caret, |extra_text_state| {
                extra_text_state.selection
            })
            .expect("Failed to get selection")
            .as_caret()
            .map(RichRange::from_caret)
            .expect("Failed to convert to range");
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
        .map(|line| format!("{}\n", line))
        .collect()
}
mod split_line {
    use super::*;

    #[test]
    fn sample() {
        let doc = Harness::split_line();
        assert_eq!(doc.notation(), to_flat(["Hello Worl<b>d|</b>"]));
    }

    #[test]
    fn delete_single_character() {
        let mut doc = Harness::split_line();
        doc.click_on_caret();
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hello Worl|"]));
    }
}

mod multistyled_line {
    use super::*;

    #[test]
    fn sample() {
        let doc = Harness::multiple_styles_in_on_line();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo Wo</i><b>rl|d</b>"]));
    }

    #[test]
    fn backspace() {
        let mut doc = Harness::multiple_styles_in_on_line();
        doc.click_on_caret();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo Wo</i><b>rl|d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo Wo</i><b>r|d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo Wo</i><b>|d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo W|</i><b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo |</i><b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>lo|</i><b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel<i>l|</i><b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["Hel|<b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["He|<b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["H|<b>d</b>"]));
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["<b>|d</b>"]));
    }
}
mod empty_line {
    use crate::rich_text::tests::{to_flat, Harness};
    #[test]
    fn sample() {
        let doc = Harness::empty_line();
        assert_eq!(doc.notation(), to_flat(["|∅"]));
    }
    #[test]
    fn backspace() {
        let mut doc = Harness::empty_line();
        doc.click_on_caret();
        doc.backspace();
        assert_eq!(doc.notation(), to_flat(["|∅"]));
    }
}
mod bold_line {
    use super::*;

    #[test]
    fn sample() {
        let mut doc = Harness::bold_line();
        #[rustfmt::skip]
        assert_eq!(doc.notation(), to_flat([
            "<b>Hello| World</b>"
        ]));
    }

    #[test]
    fn enter_line_at_caret() {
        let mut doc = Harness::bold_line();
        doc.click_on_caret();
        doc.enter();

        #[rustfmt::skip]
        assert_eq!(
            doc.notation(),
            to_flat([
            "<b>Hello</b>", 
            "<b>| World</b>"
            ])
        );
    }

    #[test]
    fn enter_line_at_caret_three_times() {
        let mut doc = Harness::bold_line();
        doc.click_on_caret();
        doc.enter();
        doc.enter();
        doc.enter();
        #[rustfmt::skip]
        assert_eq!(
            doc.notation(),
            to_flat([
            "<b>Hello</b>",
            "<b>∅</b>",
            "<b>∅</b>",
            "<b>| World</b>"
            ])
        );
    }
}
mod plain_line {
    use super::*;
    #[test]
    fn sample() {
        let doc = Harness::plain_line();
        #[rustfmt::skip]
        assert_eq!(doc.notation(), to_flat([
            "Hello| World"
        ]));
    }

    mod backspace {
        use super::*;

        #[test]
        fn backspace() {
            let mut doc = Harness::plain_line();
            doc.click_on_caret();
            doc.enter();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello", 
                    "| World"
            ]));
            doc.enter();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello",
                    "∅",
                    "| World"
            ]));

            doc.backspace();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello", 
                    "| World"
            ]));

            doc.backspace();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello| World"
            ]));

            doc.backspace();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hell| World"
            ]));

            doc.backspace();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hel| World"
            ]));
        }

        #[test]
        fn backspace_inherits_hidden_input_behaviour() {
            let mut doc = Harness::plain_line();
            doc.click_on_caret();
            doc.backspace();
            assert_eq!(doc.notation(), to_flat(["Hell| World"]));
            doc.backspace();
            assert_eq!(doc.notation(), to_flat(["Hel| World"]));
            doc.backspace();
            assert_eq!(doc.notation(), to_flat(["He| World"]));
            doc.backspace();
            assert_eq!(doc.notation(), to_flat(["H| World"]));
            doc.backspace();
            assert_eq!(doc.notation(), to_flat(["| World"]));
            for _ in 0..512 {
                doc.backspace();
            }
            assert_eq!(doc.notation(), to_flat(["| World"]));
        }
        #[test]
        fn simulate_hidden_input() {
            let mut doc = Harness::plain_line();
            let handler_nop = |_: &mut Harness, _: &mut bool| ();
            doc.click_on_caret();
            doc.simulate_hidden_input_backspace(handler_nop);
            assert_eq!(doc.notation(), to_flat(["Hell| World"]));
            doc.simulate_hidden_input_backspace(handler_nop);
            assert_eq!(doc.notation(), to_flat(["Hel| World"]));
            doc.simulate_hidden_input_backspace(handler_nop);
            assert_eq!(doc.notation(), to_flat(["He| World"]));
            doc.simulate_hidden_input_backspace(handler_nop);
            assert_eq!(doc.notation(), to_flat(["H| World"]));
            doc.simulate_hidden_input_backspace(handler_nop);
            assert_eq!(doc.notation(), to_flat(["| World"]));
            for _ in 0..512 {
                doc.simulate_hidden_input_backspace(handler_nop);
            }
            assert_eq!(doc.notation(), to_flat(["| World"]));
        }
    }

    mod enter {
        use super::*;
        fn enter_line_at_caret() {
            let mut doc = Harness::plain_line();
            doc.click_on_caret();
            doc.enter();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello", 
                    "| World"
            ]));
        }

        #[test]
        fn enter_line_at_caret_three_times() {
            let mut doc = Harness::plain_line();
            doc.click_on_caret();
            doc.enter();
            doc.enter();
            doc.enter();
            #[rustfmt::skip]
            assert_eq!(doc.notation(), to_flat([
                    "Hello", 
                    "∅", 
                    "∅", 
                    "| World"
            ]));
        }
    }
}
