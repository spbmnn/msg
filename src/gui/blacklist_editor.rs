use iced::{
    advanced::text::highlighter::PlainText,
    widget::{text::Highlighter, text_editor, TextEditor},
    Length,
};

use crate::app::Message;

pub struct BlacklistEditor {
    content: text_editor::Content,
}

impl BlacklistEditor {
    pub fn new(initial_text: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(initial_text),
        }
    }

    pub fn view(&mut self) -> TextEditor<'_, PlainText, Message> {
        text_editor(&mut self.content).height(120)
    }

    pub fn text(&self) -> String {
        self.content.text().to_string()
    }
}
