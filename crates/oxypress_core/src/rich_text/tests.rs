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
    pub fn simulate_hidden_input_backspace<F, PD>(
        &mut self,
        prevent_default: &mut bool,
        prevent_default_cb: PD,
        mut before_backspace: F,
    ) where
        F: FnMut(&mut Self, PD),
        PD: FnMut(),
    {
        before_backspace(self, prevent_default_cb);
        if *prevent_default {
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
        let mut prevent_default = false;
        let prevent_default_clone = &mut prevent_default as *mut bool; 
        let prevent_default_cb = || {
            unsafe {
                *prevent_default_clone = true; 
            }
        };
        self.simulate_hidden_input_backspace(
            &mut prevent_default,
            prevent_default_cb,
            |harness, mut prevent_default| {
                let focused_node = *harness.ta_state.focused_node();
                let document_selection = harness
                    .ta_state
                    .document_selection()
                    .expect("Failed to get document selection");
                let selection_is_caret = document_selection.start_node == focused_node
                    && document_selection.start_offset == document_selection.end_offset;
                if !selection_is_caret {
                    return;
                }

                let caret = document_selection.start_offset;
                let helpers = harness.ta_state.document_helpers();
                let focused_content = helpers
                    .try_get_text_state(focused_node, |text_state| text_state.content.clone())
                    .expect("Failed to get focused text state");
                let line = helpers.node_info(focused_node, |info| info.parent);
                let line_children = helpers.node_info(line, |info| info.children.to_vec());
                let focused_position = line_children
                    .iter()
                    .position(|node| *node == focused_node)
                    .expect("Focused node is not part of its line");

                if caret.utf8 > 0 {
                    let previous_character_start = focused_content[..caret.utf8]
                        .char_indices()
                        .next_back()
                        .map(|(index, _)| index)
                        .expect("Caret is not past the start of the content");
                    let focused_node_empties =
                        previous_character_start == 0 && focused_content.len() == caret.utf8;
                    if !focused_node_empties || line_children.len() == 1 {
                        return;
                    }

                    prevent_default();
                    let has_previous_sibling = focused_position > 0;
                    let anchor = if has_previous_sibling {
                        line_children[focused_position - 1]
                    } else {
                        line_children[focused_position + 1]
                    };
                    let anchor_caret = if has_previous_sibling {
                        text_node_end(&harness.ta_state, anchor)
                    } else {
                        RichOffset::zero()
                    };
                    helpers.delete_node(focused_node);
                    place_caret(&mut harness.ta_state, anchor, anchor_caret);
                    return;
                }

                prevent_default();

                if focused_content.is_empty() {
                    if line_children.len() > 1 {
                        let has_previous_sibling = focused_position > 0;
                        let anchor = if has_previous_sibling {
                            line_children[focused_position - 1]
                        } else {
                            line_children[focused_position + 1]
                        };
                        let anchor_caret = if has_previous_sibling {
                            text_node_end(&harness.ta_state, anchor)
                        } else {
                            RichOffset::zero()
                        };
                        helpers.delete_node(focused_node);
                        place_caret(&mut harness.ta_state, anchor, anchor_caret);
                        return;
                    }

                    let runner = harness.ta_state.runner(|_| None);
                    document_operations::merge_current_line_into_previous_line_at_caret(
                        &mut harness.ta_state,
                        focused_node,
                        document_selection,
                        runner,
                    );
                    return;
                }

                if focused_position == 0 {
                    let runner = harness.ta_state.runner(|_| None);
                    document_operations::merge_current_line_into_previous_line_at_caret(
                        &mut harness.ta_state,
                        focused_node,
                        document_selection,
                        runner,
                    );
                    return;
                }

                let previous_node = line_children[focused_position - 1];
                let previous_content = helpers
                    .try_get_text_state(previous_node, |text_state| text_state.content.clone())
                    .expect("Failed to get previous text state");
                let last_character_start = previous_content
                    .char_indices()
                    .next_back()
                    .map(|(index, _)| index)
                    .expect("Previous node content is not empty");

                if last_character_start == 0 {
                    let has_node_before_previous = focused_position >= 2;
                    let anchor = if has_node_before_previous {
                        line_children[focused_position - 2]
                    } else {
                        focused_node
                    };
                    let anchor_caret = if has_node_before_previous {
                        text_node_end(&harness.ta_state, anchor)
                    } else {
                        RichOffset::zero()
                    };
                    helpers.delete_node(previous_node);
                    place_caret(&mut harness.ta_state, anchor, anchor_caret);
                    return;
                }

                let mut trimmed_content = previous_content;
                trimmed_content.truncate(last_character_start);
                let caret = RichOffset::from_utf8_offset(&trimmed_content, last_character_start);
                helpers
                    .try_get_text_state_mut(previous_node, |text_state| {
                        text_state.content = trimmed_content;
                    })
                    .expect("Failed to set previous text state");
                place_caret(&mut harness.ta_state, previous_node, caret);
            },
        );
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

fn place_caret(state: &mut TextAreaState, node: Ptr<RichTag>, caret: RichOffset) {
    let helpers = state.document_helpers();
    state.flush_overlays();
    helpers
        .try_set_text_selection(node, TextSelectionKind::Caret(caret))
        .expect("Failed to set text selection");
    *state.focused_node_mut() = node;
    *state.document_selection_mut() =
        Some(RichSelection::mononode(node, RichRange::from_caret(caret)));
}

fn text_node_end(state: &TextAreaState, node: Ptr<RichTag>) -> RichOffset {
    let content = state
        .document_helpers()
        .try_get_text_state(node, |text_state| text_state.content.clone())
        .expect("Failed to get text state");
    RichOffset::from_utf8_offset(&content, content.len())
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
            let mut prevent_default = false;
            let mut doc = Harness::plain_line();
            let handler_nop = |_: &mut Harness, _| ();
            let mut prevent_default_cb_nop = || ();
            doc.click_on_caret();

            doc.simulate_hidden_input_backspace(
                &mut prevent_default,
                prevent_default_cb_nop,
                handler_nop,
            );
            assert_eq!(doc.notation(), to_flat(["Hell| World"]));
            doc.simulate_hidden_input_backspace(
                &mut prevent_default,
                prevent_default_cb_nop,
                handler_nop,
            );
            assert_eq!(doc.notation(), to_flat(["Hel| World"]));
            doc.simulate_hidden_input_backspace(
                &mut prevent_default,
                prevent_default_cb_nop,
                handler_nop,
            );
            assert_eq!(doc.notation(), to_flat(["He| World"]));
            doc.simulate_hidden_input_backspace(
                &mut prevent_default,
                prevent_default_cb_nop,
                handler_nop,
            );
            assert_eq!(doc.notation(), to_flat(["H| World"]));
            doc.simulate_hidden_input_backspace(
                &mut prevent_default,
                prevent_default_cb_nop,
                handler_nop,
            );
            assert_eq!(doc.notation(), to_flat(["| World"]));
            for _ in 0..512 {
                doc.simulate_hidden_input_backspace(
                    &mut prevent_default,
                    prevent_default_cb_nop,
                    handler_nop,
                );
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
