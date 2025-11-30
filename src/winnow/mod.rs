use winnow::ascii::{
    alpha1, alphanumeric0, alphanumeric1, line_ending, multispace0, multispace1, space0, space1,
};
use winnow::combinator::{
    alt, backtrack_err, delimited, dispatch, empty, fail, opt, peek, preceded, repeat, terminated,
};
use winnow::error::InputError;
use winnow::prelude::*;
use winnow::token::{any, none_of, one_of, take_while};
use winnow::{error::ParserError, token::take_till};

/*
 * nonterminal = @{ lbrack ~ rule_name ~ rbrack }
 *
 * term = { (literal | nonterminal) ~ opt_modifier}
 *
 * literal = @{
 *     "\"" ~ not_quote_or_nl+ ~ "\"" |
 *     "'" ~ not_squote_or_nl+ ~ "'"
 * }
 *
 * not_quote_or_nl = {
 *     !(                // if the following text is not
 *         "\""          //     a quote
 *         | "\n"        //     or a newline
 *     )
 *     ~ ( "\\" ~ "\"" | ANY ) // then consume one character
 * }
 * not_squote_or_nl = {
 *     !(                // if the following text is not
 *         "\'"          //     a quote
 *         | "\n"        //     or a newline
 *     )
 *     ~ ( "\\" ~ "\'" | ANY ) // then consume one character
 * }
 */

pub fn literal<'a>(input: &'a mut &str) -> ModalResult<String> {
    let lit = alt((
        delimited('"', take_till(1.., ('"', '\n')), '"'),
        delimited('\'', take_till(1.., ('\'', '\n')), '\''),
    ))
    .parse_next(input)?
    .to_string();

    Ok(lit)
}

/*
 * rule_name = { ASCII_ALPHA ~ (ASCII_ALPHA | ASCII_DIGIT | "_" | " " | "-")* }
 */
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

pub fn term<'a>(input: &'a mut &str) -> ModalResult<String> {
    (alt((literal, nonterminal)), opt(modifier).take())
        .parse_next(input)
        .map(|(mut a, b)| {
            a.extend(b.chars());
            a
        })
}

/*
 * pub fn foo<'a>(input: &'a mut &str) -> ModalResult<String> {
 *     terminated(term, multispace0).parse_next(input)
 * }
 */

pub fn list<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    repeat(1.., terminated(term, multispace0)).parse_next(input)
}

pub fn expression<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    let mut l1 = list.parse_next(input)?;
    let rest: Vec<Vec<String>> =
        repeat(0.., preceded(('|', multispace0), list)).parse_next(input)?;
    let rest = rest.into_iter().flatten();
    l1.extend(rest.into_iter());

    Ok(l1)
}
pub fn group<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    alt((expression, delimited('(', expression, ')'))).parse_next(input)
}

pub fn sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    let mut x: Vec<Vec<String>> =
        repeat(1.., alt((term.map(|t| vec![t]), group))).parse_next(input)?;
    let x = x.into_iter().flatten().collect();
    Ok(x)
}

pub fn modifier<'a>(input: &'a mut &str) -> ModalResult<String> {
    alt(('?', '+', '*')).parse_next(input).map(|c| c.into())
}

pub fn nonterminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('<', rule_name, '>').parse_next(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(1, 1);
    }
}
