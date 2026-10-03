/*!

A simple Nom parser for the `arith` language.

The `arith` language is simple Peano arithmetic endowed with an `if`-`then`-`else` construct, `true`, `false`, and
an `iszero` test function.

*/

use nom::{
  IResult,
  Parser,
  branch::alt,
  bytes::complete::{
    is_not,
    tag,
    take_until,
  },
  character::complete::multispace1,
  error::Error,
  multi::many0,
  sequence::{
    delimited,
    terminated
  },
  combinator::value
};

use crate::ast::{BTerm, Term};

/// Opens an inline comment.
const OPEN_COMMENT: &str = "/*";
/// Closes an inline comment.
const CLOSE_COMMENT: &str = "*/";
/// Begins an end-of-line comment.
const EOL_COMMENT: &str = "//";

/// A convenience function that returns a term or a parse error.
pub fn parse(input: &str) -> Result<BTerm, nom::Err<Error<&str>>> {
  parse_term.parse(input).map(|(_, term)| term)
}


pub fn parse_term(input: &str) -> IResult<&str, BTerm> {
  ws(alt((
    parse_keyword,
    parse_succ,
    parse_pred,
    parse_iszero,
    parse_if,
  ))).parse(input)
}

fn parse_keyword(input: &str) -> IResult<&str, BTerm> {
  alt((
    tag("0"),
    tag("zero"),
    tag("true"),
    tag("false"),
  ))
      .map(|head| match head {
        "0" | "zero" => Term::Zero.boxed(),
        "true" => Term::True.boxed(),
        "false" => Term::False.boxed(),
        _ => unreachable!(),
      })
      .parse(input)
}

/// Parses a unary function of the form `function(arg)`, returning `arg` parsed as a `BTerm`.
fn parse_function_application<'a>(
  function: &'static str,
) -> impl Parser<&'a str, Output = BTerm, Error = Error<&'a str>>
{
  delimited(
    terminated(tag(function), tag("(")),
    parse_term,
    tag(")"),
  )
}

/// Parses `succ(arg)`
fn parse_succ(input: &str) -> IResult<&str, BTerm> {
  parse_function_application("succ")
      .map(|term| Term::Succ(term).boxed())
      .parse(input)
}

/// Parses `pred(arg)`
fn parse_pred(input: &str) -> IResult<&str, BTerm> {
  parse_function_application("pred")
      .map(|term| Term::Pred(term).boxed())
      .parse(input)
}

/// Parses `iszero(arg)`
fn parse_iszero(input: &str) -> IResult<&str, BTerm> {
  parse_function_application("iszero")
      .map(|term| Term::IsZero(term).boxed())
      .parse(input)
}

/// Parses `if guard then branch_true else branch_false`
fn parse_if(input: &str) -> IResult<&str, BTerm> {
  (
    tag("if"),
    parse_term,
    tag("then"),
    parse_term,
    tag("else"),
    parse_term,
  )
      .map(|(_, guard, _, branch_true, _, branch_false)| {
        Term::If {
          guard,
          branch_true,
          branch_false,
        }
            .boxed()
      })
      .parse(input)
}


// region Auxiliary parsers for ignorables.


/// Noms surrounding whitespace, including newlines and comments.
fn ws<'a, O, P>(inner: P) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = Error<&'a str>>,
{
  delimited(pskip(), inner, pskip())
}

/// Noms trailing whitespace, including newlines and comments.
#[allow(dead_code)]
fn wst<'a, O, P>(inner: P) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = Error<&'a str>>,
{
  terminated(inner, pskip())
}

/// Noms whitespace, including newlines and comments, returning `()`.
pub fn pskip<'a>() -> impl Parser<&'a str, Output = (), Error = Error<&'a str>> {
  value(
    (),
    many0(alt((
      value((), multispace1),
      pinline_comment(),
      peol_comment(),
    ))),
  )
}


/// Noms eol comments, excluding newlines, returning `()`.
pub fn peol_comment<'a>() -> impl Parser<&'a str, Output = (), Error = Error<&'a str>> {
  value(
    (),
    (
      tag(EOL_COMMENT),
      is_not("\n\r"),
    ),
  )
}


/// Noms block comments, excluding surrounding whitespace, returning `()`.
pub fn pinline_comment<'a>() -> impl Parser<&'a str, Output = (), Error = Error<&'a str>> {
  value(
    (),
    (
      tag(OPEN_COMMENT),
      take_until(CLOSE_COMMENT),
      tag(CLOSE_COMMENT),
    ),
  )
}

// endregion



#[cfg(test)]
mod tests {
  use crate::parse::*;

  #[test]
  fn test_keywords() {
    assert_eq!(parse_keyword("true"), Ok(("", Term::True.boxed())));
    assert_eq!(parse_keyword("false"), Ok(("", Term::False.boxed())));
    assert_eq!(parse_keyword("zero"), Ok(("", Term::Zero.boxed())));
  }

  #[test]
  fn test_apply_functions() {
    let succ_expected   = Term::Succ(Term::Zero.boxed()).boxed();
    let pred_expected   = Term::Pred(Term::Zero.boxed()).boxed();
    let iszero_expected = Term::IsZero(Term::Zero.boxed()).boxed();

    assert_eq!(parse_term("   succ(zero  )"), Ok(("", succ_expected)));
    assert_eq!(parse_term("pred(  zero  )   "), Ok(("", pred_expected)));
    assert_eq!(parse_term("iszero( zero)   "), Ok(("", iszero_expected)));
  }


  #[test]
  fn test_if_statement() {
    let expected = Term::If {
      guard       : Term::True.boxed(),
      branch_true : Term::Zero.boxed(),
      branch_false: Term::False.boxed()
    };

    assert_eq!(parse_term("if true   then   zero else false "), Ok(("", expected.boxed())));
  }


  #[test]
  fn test_display() {
    let succ = Term::Succ(Term::True.boxed()).boxed();
    let pred = Term::Pred(Term::False.boxed()).boxed();

    // println!("Successor example: {}\nPredessesor example: {}", succ, pred)
    assert_eq!(succ.to_string(), "succ(true)".to_string());
    assert_eq!(pred.to_string(), "pred(false)".to_string());


    let if_statement = Term::If {
      guard: Term::True.boxed(),
      branch_true: Term::Zero.boxed(),
      branch_false: Term::False.boxed()
    };

    assert_eq!(
      if_statement.to_string(),
      "if true then zero else false".to_string()
    );
  }

}
