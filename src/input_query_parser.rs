use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    sequence::delimited,
    IResult,
    Parser
};
use nom::combinator::recognize;


fn parse_quoted(input: &str) -> IResult<&str, &str> {
    let mut parser = recognize(delimited(tag("'"), take_until("'"), tag("'")));
    parser.parse(input)
}

fn parse_unquoted(input: &str) -> IResult<&str, &str> {
    is_not("';")(input)
}

pub(crate) fn parse_until_semicolon(input: &str) -> IResult<&str, (String, bool)> {
    let mut result = String::new();
    let mut rest = input;

    loop {
        if let Ok((remaining, _)) = tag::<&str, &str, nom::error::Error<&str>>(";")(rest) {
            return Ok((remaining, (result, true)));
        }

        let mut parser = alt((parse_quoted, parse_unquoted));
        let (new_rest, segment) = parser.parse(rest)?;
        result.push_str(segment);
        rest = new_rest;

        if rest.is_empty() {
            break;
        }
    }
    Ok((rest, (result, false)))
}

