//! The way back from [`sicompass_sdk::tags::escape`], which is how a row keeps text
//! it did not write from being read as tags.
//!
//! `tags::strip_display` unescapes, but `tags::extract_input` hands back the raw
//! slice, and the SDK's own `unescape` is private. A value must not gain a backslash
//! on its way into a message or a saved setting, so what comes back from the app
//! goes through here wherever it can still be escaped.

/// Undo `tags::escape`.
///
/// Safe on text that was never escaped: only a backslash right before a bracket is
/// dropped, and `escape` never leaves a bracket without one.
pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(bracket) = chars.next_if(|next| matches!(next, '<' | '>'))
        {
            out.push(bracket);
            continue;
        }
        out.push(c);
    }
    out
}

/// What the user means by the `new` of a `commit_edit`.
///
/// `new` is what they typed, plain, and `old` the field as it was written, escaped.
/// The one exception is Enter on a field outside Insert mode, which only commits the
/// field: the app reads the value out of the row and passes the same raw slice as
/// both, so equal values are still escaped. Text with no bracket is the same either
/// way. Retyping a field to hold, literally, the escaped form of what it already
/// held looks identical and loses its backslash, and that is the price.
pub fn typed_value(old: &str, new: &str) -> String {
    if old == new {
        unescape(new)
    } else {
        new.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sicompass_sdk::tags;

    #[test]
    fn unescape_undoes_escape_whatever_the_text_holds() {
        for text in [
            "plain",
            "<b>x</b>",
            r"a\<b",
            r"a\\<b\>",
            r"ends in a backslash\",
            r"C:\notes\<new>",
            "",
        ] {
            assert_eq!(unescape(&tags::escape(text)), text, "{text:?}");
        }
    }

    #[test]
    fn unescape_leaves_text_that_was_never_escaped() {
        assert_eq!(unescape(r"C:\notes"), r"C:\notes");
        assert_eq!(unescape("a < b"), "a < b");
    }

    #[test]
    fn a_typed_value_is_taken_as_typed() {
        assert_eq!(typed_value(r"\<b\>", "<b>"), "<b>");
        assert_eq!(typed_value("", r"\<b\>"), r"\<b\>");
    }

    #[test]
    fn enter_on_a_field_commits_its_value_not_its_escapes() {
        assert_eq!(typed_value(r"\<b\>", r"\<b\>"), "<b>");
    }
}
