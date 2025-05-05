use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, not_line_ending},
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};

use crate::core::dtext::model::DTextSpan;

/// Entry point for DText parser
pub fn parse_dtext(input: &str) -> IResult<&str, Vec<DTextSpan>> {
    many0(parse_span).parse(input)
}

/// Parses one DText span at a time.
fn parse_span(input: &str) -> IResult<&str, DTextSpan> {
    alt((
        parse_bold,
        parse_italics,
        parse_post_reference,
        parse_plain_text,
    ))
    .parse(input)
}

/// Parses `[b]bold[/b]`
fn parse_bold(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[b]"), parse_dtext, tag("[/b]")).parse(input)?;
    Ok((rest, DTextSpan::Bold(content)))
}

/// Parses `[i]italics[/i]`
fn parse_italics(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, content) = delimited(tag("[i]"), parse_dtext, tag("[/i]")).parse(input)?;
    Ok((rest, DTextSpan::Italics(content)))
}

fn parse_post_reference(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, (_, digits)) = (tag("post #"), digit1).parse(input)?;
    let id = digits.parse::<u32>().unwrap_or(0);
    Ok((rest, DTextSpan::PostLink(id)))
}

fn parse_plain_text(input: &str) -> IResult<&str, DTextSpan> {
    let (rest, text) = take_while1(|c| !matches!(c, '*' | 'p')).parse(input)?;
    Ok((rest, DTextSpan::Text(text.to_string())))
}
