/*!

A simple Nom parser for the `arith` language.

The `arith` language is simple Peano arithmetic endowed with an `if`-`then`-`else` construct, `true`, `false`, and
an `iszero` test function.

*/

use nom::{
  branch::alt,
  bytes::complete::{
    is_not,
    tag,
    take_until
  },
  character::complete::multispace1,
  combinator::{
    map,
    value
  },
  error::{
    ParseError,
    Error
  },
  IResult,
  multi::many0,
  sequence::{
    delimited,
    pair,
    terminated,
    tuple
  },
};

use crate::ast::{BTerm, Term};

/// Opens an inline comment.
const OPEN_COMMENT: &str = "/*";
/// Closes an inline comment.
const CLOSE_COMMENT: &str = "*/";
/// Begins an end-of-line comment.
const EOL_COMMENT: &str = "//";

fn parse_keyword(input: &str) -> IResult<&str, BTerm> {
  let (rest, head) = alt((
    tag("0"),
    tag("zero"),
    tag("true"),
    tag("false"),
  ))(input)?;

  let term = match head {

    | "0"
    | "zero" => {
      Term::Zero
    },

    "true" => Term::True,

    "false" => Term::False,

    _ => {
      unreachable!()
    }
  };

  Ok((rest, term.boxed()))
}

/// A convenience function that returns a term or a parse error.
pub fn parse(input: &str) -> Result<BTerm, nom::Err<Error<&str>>> {
  parse_term(input).map(|(_, term)| term)
}


pub fn parse_term(input: &str) -> IResult<&str, BTerm> {
  ws(
    | text | alt((
      parse_keyword,
      parse_succ,
      parse_pred,
      parse_iszero,
      parse_if
    ))(text)
  )(input)
}

/// Parses a unary function of the form `function(arg)`, returning `arg` parsed as a `BTerm`.
fn parse_function_application<'s>(function: &'static str, input: &'s str) -> IResult<&'s str, BTerm> {
  delimited(
    terminated(tag(function), tag("(")), // No whitespace allowed between function name and `(`.
    parse_term,
    tag(")")
  )(input)
}

/// Parses `succ(arg)`
fn parse_succ(input: &str) -> IResult<&str, BTerm> {
  let (rest, pred_term) = parse_function_application("succ", input)?;
  let term = Term::Succ(pred_term);

  Ok((rest, term.boxed()))
}

/// Parses `pred(arg)`
fn parse_pred(input: &str) -> IResult<&str, BTerm> {
  let (rest, succ_term): (&str, BTerm) = parse_function_application("pred", input)?;
  let term = Term::Pred(succ_term);

  Ok((rest, term.boxed()))
}

/// Parses `iszero(arg)`
fn parse_iszero(input: &str) -> IResult<&str, BTerm> {
  let (rest, succ_term): (&str, BTerm) = parse_function_application("iszero", input)?;
  let term = Term::IsZero(succ_term);

  Ok((rest, term.boxed()))
}


/// Parses `if guard then branch_true else branch_false`
fn parse_if(input: &str) -> IResult<&str, BTerm> {
  let (rest, (_if, guard, _then, branch_true, _else, branch_false))
    = tuple((
      tag("if"),
      parse_term,
      tag("then"),
      parse_term,
      tag("else"),
      parse_term
    ))(input)?;

  let term = Term::If {
    guard,
    branch_true,
    branch_false
  };

  Ok((rest, term.boxed()))
}



// region Auxiliary parsers for ignorables.


/// Noms surrounding whitespace, including newlines and comments.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
  where
      F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  move |i| {
    delimited(
      &pskip,
      &inner,
      &pskip
    )(i)
  }
}

/// Noms trailing whitespace, including newlines and comments.
#[allow(dead_code)]
fn wst<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O, E>
  where
      F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  move |i| {
    terminated(
      &inner,
      &pskip
    )(i)
  }
}

/// Noms whitespace, including newlines and comments, returning `()`.
pub fn pskip<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
{
  map(
    many0(
      alt((map(multispace1, |_| ()), pinline_comment, peol_comment))
    ),
    |_| ()
  )(i)
}


/// Noms eol comments, excluding newlines, returning `()`.
pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
{
  value(
    (), // Output is thrown away.
    pair(tag(EOL_COMMENT), is_not("\n\r"))
  )(i)
}


/// Noms block comments, excluding surrounding whitespace, returning `()`.
pub fn pinline_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
{
  map(
    tuple((
      tag(OPEN_COMMENT),
      take_until(CLOSE_COMMENT),
      tag(CLOSE_COMMENT)
    )),
    |_| () // Output is thrown away.
  )(i)
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
