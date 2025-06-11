use super::parse_dtext;
use crate::core::dtext::model::DTextSpan;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, not_line_ending},
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};

/// Parses `[b]bold[/b]`
pub(super) fn bold(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[b]"), parse_dtext, tag("[/b]")).parse(input)?;
    Ok((rest, DTextSpan::Bold(content)))
}

/// Parses `[i]italics[/i]`
pub(super) fn italics(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[i]"), parse_dtext, tag("[/i]")).parse(input)?;
    Ok((rest, DTextSpan::Italics(content)))
}

/// Parses `[s]Strikeout[/s]`
pub(super) fn strikeout(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[s]"), parse_dtext, tag("[/s]")).parse(input)?;
    Ok((rest, DTextSpan::Strikeout(content)))
}

/// Parses `[u]Underline[/u]`
pub(super) fn underline(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[u]"), parse_dtext, tag("[/u]")).parse(input)?;
    Ok((rest, DTextSpan::Underline(content)))
}

/// Parses `[sup]Superscript[/sup]`
pub(super) fn superscript(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[sup]"), parse_dtext, tag("[/sup]")).parse(input)?;
    Ok((rest, DTextSpan::Superscript(content)))
}

/// Parses `[sub]Subscript[/sub]`
pub(super) fn subscript(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[sup]"), parse_dtext, tag("[/sup]")).parse(input)?;
    Ok((rest, DTextSpan::Superscript(content)))
}

/// Parses `[spoiler]Spoilers[/spoiler]`
pub(super) fn spoiler(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) =
        delimited(tag("[spoiler]"), parse_dtext, tag("[/spoiler]")).parse(input)?;
    Ok((rest, DTextSpan::Superscript(content)))
}

/// Parses `\`inline code\``.
pub(super) fn inline_code(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(char('`'), parse_dtext, char('`')).parse(input)?;
    Ok((rest, DTextSpan::Superscript(content)))
}
