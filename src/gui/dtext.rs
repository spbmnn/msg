use crate::core::dtext::model::DTextSpan;
use iced::{
    font,
    widget::{
        text::{self, Rich, Span},
        Column, Text,
    },
    Element, Length,
};

pub fn render_dtext<'a>(spans: &'a [DTextSpan]) -> Text<'a, Rich<'a>> {
    let rich_spans: Vec<Rich<'a>> = spans.iter().flat_map(span_to_rich).collect();
    Text::with(&rich_spans)
}

fn span_to_rich<'a>(span: &'a DTextSpan) -> Vec<Rich<'a>> {
    match span {
        DTextSpan::Text(s) => vec![Rich::with_spans(Span::new(s.as_str()))],

        DTextSpan::Bold(children) => children
            .iter()
            .flat_map(span_to_rich)
            .map(|r| r.font(font::Weight::Bold))
            .collect(),

        DTextSpan::Italics(children) => children
            .iter()
            .flat_map(span_to_rich)
            .map(|r| r.font(font::Style::Italic))
            .collect(),
    }
}
