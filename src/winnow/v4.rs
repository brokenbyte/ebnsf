use winnow::ascii::{alpha1, multispace0, space1};
use winnow::combinator::{alt, delimited, separated};
use winnow::prelude::*;
use winnow::token::take_while;
use winnow::ModalResult;

// Terminal parser: parses "alphanum..."
pub fn terminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('"', rule_name, '"').map(|content| format!("\"{}\"", content)).parse_next(input)
}

// Nonterminal parser: parses <alphanum...>
pub fn nonterminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('<', rule_name, '>').map(|content| format!("<{}>", content)).parse_next(input)
}

// Atom parser: chooses between terminal, nonterminal, or group
pub fn atom<'a>(input: &'a mut &str) -> ModalResult<String> {
    alt((terminal, nonterminal, group)).parse_next(input)
}

// Group parser: parses (sequence) recursively
pub fn group<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('(', sequence.map(|inner: Vec<String>| format!("({})", inner.join(" "))), ')').parse_next(input)
}

// Sequence parser: space-separated list of atoms
pub fn sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    separated(1.., atom, space1).parse_next(input)
}

// Top-level parser: trims whitespace and parses a sequence
pub fn parse_sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    (multispace0).void().parse_next(input)?;  // Skip leading whitespace
    sequence.parse_next(input)
}

// Helper: rule_name for terminals/nonterminals (alphanumeric starting with letter)
pub fn rule_name<'a>(input: &'a mut &str) -> ModalResult<String> {
    let rule = (
        alpha1,
        take_while(0.., ('a'..='z', 'A'..='Z', '0'..='9', ' ', '_', '-')),
    )
        .take()
        .parse_next(input)?
        .to_string();

    Ok(rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sequence() {
        let mut input = "<foo> \"bar\" <baz>";
        assert_eq!(parse_sequence(&mut input).unwrap(), vec!["<foo>", "\"bar\"", "<baz>"]);

        let mut input2 = "<foo> <woo> <boo> \"baz\"";
        assert_eq!(parse_sequence(&mut input2).unwrap(), vec!["<foo>", "<woo>", "<boo>", "\"baz\""]);

        let mut input3 = "\"honk\" \"bonk\" \"tonk\" <wonk>";
        assert_eq!(parse_sequence(&mut input3).unwrap(), vec!["\"honk\"", "\"bonk\"", "\"tonk\"", "<wonk>"]);

        let mut input4 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz>";
        assert_eq!(parse_sequence(&mut input4).unwrap(), vec!["<foo>", "<bar>", "(\"hello\" <baz>)", "\"world\"", "<buz>"]);
    }
}