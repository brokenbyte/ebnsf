use winnow::ModalResult;
use winnow::ascii::{alpha1, multispace0, multispace1, space0, space1};
use winnow::combinator::{alt, delimited, preceded, separated};
use winnow::prelude::*;
use winnow::token::take_while;

// Terminal parser: parses "alphanum..."
pub fn terminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('"', rule_name, '"')
        .map(|content| format!("\"{}\"", content))
        .parse_next(input)
}

// Nonterminal parser: parses <alphanum...>
pub fn nonterminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('<', rule_name, '>')
        .map(|content| format!("<{}>", content))
        .parse_next(input)
}

// Atom parser: chooses between terminal, nonterminal, or group
pub fn atom<'a>(input: &'a mut &str) -> ModalResult<String> {
    alt((terminal, nonterminal, group)).parse_next(input)
}

// Group parser: parses (sequence) recursively, supporting alternatives
pub fn group<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited(
        '(',
        parse_sequence.map(|alts: Vec<Vec<String>>| {
            if alts.len() == 1 {
                format!("({})", alts[0].join(" "))
            } else {
                format!(
                    "({})",
                    alts.iter()
                        .map(|seq| seq.join(" "))
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            }
        }),
        ')',
    )
    .parse_next(input)
}

// Sequence parser: space-separated list of atoms
pub fn sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    separated(1.., atom, space1).parse_next(input)
}

// Top-level parser: trims whitespace and parses sequences separated by |
pub fn parse_sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<Vec<String>>> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., sequence, delimited(multispace0, "|", multispace0)).parse_next(input)
}

// Parses a single rule: <rule_name> ::= <alternatives>
pub fn parse_rule<'a>(input: &'a mut &str) -> ModalResult<(String, Vec<Vec<String>>)> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    let (name_full, alts) = (
        nonterminal,
        preceded((space0, "::=", space0), parse_sequence),
    )
        .parse_next(input)?;
    // Extract name without <>
    let name: String = name_full
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string();
    Ok((name, alts))
}

// Parses multiple rules separated by whitespace/newlines
pub fn parse_grammar<'a>(input: &'a mut &str) -> ModalResult<Vec<(String, Vec<Vec<String>>)>> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., parse_rule, multispace1).parse_next(input)
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
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec!["<foo>", "\"bar\"", "<baz>"]]
        );

        let mut input2 = "<foo> <woo> <boo> \"baz\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec!["<foo>", "<woo>", "<boo>", "\"baz\""]]
        );

        let mut input3 = "\"honk\" \"bonk\" \"tonk\" <wonk>";
        assert_eq!(
            parse_sequence(&mut input3).unwrap(),
            vec![vec!["\"honk\"", "\"bonk\"", "\"tonk\"", "<wonk>"]]
        );

        let mut input4 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz>";
        assert_eq!(
            parse_sequence(&mut input4).unwrap(),
            vec![vec![
                "<foo>",
                "<bar>",
                "(\"hello\" <baz>)",
                "\"world\"",
                "<buz>"
            ]]
        );
    }

    #[test]
    fn test_parse_alternatives() {
        let mut input = "<foo> <bar> | \"baz\" <qux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec!["<foo>", "<bar>"], vec!["\"baz\"", "<qux>"]]
        );

        let mut input2 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz> | \"honk\" <bonk> (<tonk> \"shonk\" <donk>) \"lonk\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![
                vec!["<foo>", "<bar>", "(\"hello\" <baz>)", "\"world\"", "<buz>"],
                vec![
                    "\"honk\"",
                    "<bonk>",
                    "(<tonk> \"shonk\" <donk>)",
                    "\"lonk\""
                ]
            ]
        );
    }

    #[test]
    fn test_nested_groups() {
        let mut input = "<foo> (\"bar\" (<baz> \"qux\")) <quux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec!["<foo>", "(\"bar\" (<baz> \"qux\"))", "<quux>"]]
        );

        let mut input2 = "(<a> (\"b\" <c>)) | (<d> \"e\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec!["(<a> (\"b\" <c>))"], vec!["(<d> \"e\")"]]
        );
    }

    #[test]
    fn test_alternatives_in_groups() {
        let mut input = "<foo> (\"bar\" | \"baz\") <qux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec!["<foo>", "(\"bar\" | \"baz\")", "<qux>"]]
        );

        let mut input2 = "(<a> | <b>) (\"c\" | \"d\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec!["(<a> | <b>)", "(\"c\" | \"d\")"]]
        );
    }

    #[test]
    fn test_multiline_alternatives() {
        let mut input = "<foo> <bar>\n| <fizz> <buzz>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec!["<foo>", "<bar>"], vec!["<fizz>", "<buzz>"]]
        );
    }

    #[test]
    fn test_parse_rule() {
        let mut input = "<foo> ::= <bar> | \"baz\"";
        let result = parse_rule(&mut input).unwrap();
        assert_eq!(result.0, "foo");
        assert_eq!(
            result.1,
            vec![vec![String::from("<bar>")], vec![String::from("\"baz\"")]]
        );
    }

    #[test]
    fn test_parse_grammar() {
        let mut input = "<foo> ::= <bar>\n<baz> ::= \"qux\" | <quux>";
        let result = parse_grammar(&mut input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "foo");
        assert_eq!(result[1].0, "baz");
    }
}
