#[derive(Debug)]
pub enum DTextSpan {
    Text(String),
    Bold(Vec<DTextSpan>),
    Italics(Vec<DTextSpan>),
    Strikeout(Vec<DTextSpan>),
    Underline(Vec<DTextSpan>),
    Superscript(Vec<DTextSpan>),
    Subscript(Vec<DTextSpan>),
    Spoiler(Vec<DTextSpan>),
    InlineCode(String),
    ColoredText { color: String, text: Vec<DTextSpan> },

    //Link { href: String, label: Vec<DTextSpan> },
    PostLink(u32),
    //LineBreak,

    //Quote(Vec<DTextSpan>),
}
