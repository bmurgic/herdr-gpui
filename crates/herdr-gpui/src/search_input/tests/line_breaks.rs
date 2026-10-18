use super::*;

/// Pastes `text` into an empty input through the platform's text-entry
/// path, which is what the clipboard and the IME both reach.
fn pasted(text: &str, breaks_as_spaces: bool, cx: &mut gpui::TestAppContext) -> String {
    let (input, cx) = cx.add_window_view(|_, cx| SearchInput::new(cx));
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.set_breaks_as_spaces(breaks_as_spaces);
            input.replace_text_in_range(None, text, window, cx);
        })
    });
    input.read_with(cx, |input, _| input.text().to_owned())
}

#[gpui::test]
fn pasted_text_keeps_or_spaces_its_characters(cx: &mut gpui::TestAppContext) {
    for (text, expected) in [
        ("claude-sonnet-4-5", "claude-sonnet-4-5"),
        ("my\tmodel", "my model"),
        ("my\r\nmodel", "my model"),
        ("my\rmodel", "my model"),
        ("my\nmodel", "my model"),
    ] {
        assert_eq!(pasted(text, true, cx), expected, "{text:?}");
    }
}

#[gpui::test]
fn line_breaks_still_drop_by_default(cx: &mut gpui::TestAppContext) {
    assert_eq!(pasted("my\nmodel", false, cx), "mymodel");
}
