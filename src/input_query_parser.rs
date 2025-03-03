use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::combinator::recognize;
use nom::sequence::delimited;
use nom::{IResult, Parser};

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

#[cfg(test)]
mod tests {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_valid_quoted() {
        let input = "'hello'";
        let expected = "'hello'";
        let result = parse_quoted(input);
        assert_eq!(result, Ok(("", expected)));
    }

    #[test]
    fn test_full_unquoted_input() {
        let input = "hello";
        let result = parse_unquoted(input);
        assert_eq!(result, Ok(("", "hello")));
    }

    #[test]
    fn test_valid_quoted_with_extra_text() {
        let input = "'world' extra";
        let expected = "'world'";
        let result = parse_quoted(input);
        assert_eq!(result, Ok((" extra", expected)));
    }

    #[test]
    fn test_unquoted_until_semicolon() {
        let input = "hello;world";
        let result = parse_unquoted(input);
        assert_eq!(result, Ok((";world", "hello")));
    }

    #[test]
    fn test_empty_quoted() {
        let input = "''";
        let expected = "''";
        let result = parse_quoted(input);
        assert_eq!(result, Ok(("", expected)));
    }

    #[test]
    fn test_missing_closing_quote() {
        let input = "'incomplete";
        let result = parse_quoted(input);
        // expect an error because the closing quote is missing.
        assert!(
            result.is_err(),
            "Expected error for missing closing quote, got {:?}",
            result
        );
    }

    #[test]
    fn test_no_starting_quote() {
        let input = "no quote'";
        let result = parse_quoted(input);
        // expect an error because the input doesn't start with a quote.
        assert!(
            result.is_err(),
            "Expected error for missing starting quote, got {:?}",
            result
        );
    }

    #[test]
    fn test_starts_with_forbidden_char() {
        let input = "'hello";
        // if the input starts with a forbidden character, the parser should fail.
        let result = parse_unquoted(input);
        assert!(
            result.is_err(),
            "Expected error for input starting with a forbidden character, got {:?}",
            result
        );
    }

    #[test]
    fn test_sql_query_with_separator() {
        let input = "SELECT * FROM read_csv('foods_hash.csv', separator='#');";
        // mind the dropped trailing semicolon
        let expected = "SELECT * FROM read_csv('foods_hash.csv', separator='#')";
        let result = parse_until_semicolon(input);
        assert_eq!(result, Ok(("", (expected.to_string(), true))));
    }
    #[test]
    fn test_only_semicolon() {
        let input = ";";
        let result = parse_until_semicolon(input);
        assert_eq!(result, Ok(("", (String::new(), true))));
    }

    #[test]
    fn test_no_semicolon() {
        let input = "SELECT * FROM table";
        let result = parse_until_semicolon(input);
        assert_eq!(result, Ok(("", (input.to_string(), false))));
    }

    #[test]
    fn test_trailing_text_after_semicolon() {
        let input = "SELECT * FROM table; extra text";
        let result = parse_until_semicolon(input);
        // Parsing stops at the semicolon, so only "SELECT * FROM table" is accumulated.
        assert_eq!(
            result,
            Ok((" extra text", ("SELECT * FROM table".to_string(), true)))
        );
    }

    #[test]
    fn test_mixed_quoted_and_unquoted_with_semicolon() {
        let input = "SELECT 'data;more' FROM table;";
        let result = parse_until_semicolon(input);
        // The segments are:
        //   unquoted: "SELECT "
        //   quoted: "'data;more'"
        //   unquoted: " FROM table"
        // The semicolon that terminates the query is not part of the result.
        let expected = "SELECT 'data;more' FROM table".to_string();
        assert_eq!(result, Ok(("", (expected, true))));
    }

    #[test]
    fn test_mixed_quoted_and_unquoted_without_semicolon() {
        let input = "INSERT INTO table VALUES ('value1', 'value;2')";
        let result = parse_until_semicolon(input);
        // all segments are accumulated because no semicolon terminates the query.
        let expected = "INSERT INTO table VALUES ('value1', 'value;2')".to_string();
        assert_eq!(result, Ok(("", (expected, false))));
    }
}
