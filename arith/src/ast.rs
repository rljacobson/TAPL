use std::fmt::{Display, Formatter};

pub type BTerm = Box<Term>;

#[derive(Eq, PartialEq, Debug)]
pub enum Term {
  Zero,
  True,
  False,
  Succ(BTerm),
  Pred(BTerm),
  If{
    guard: BTerm,
    branch_true: BTerm,
    branch_false: BTerm,
  },
  IsZero(BTerm)
}

impl Display for Term {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let text = // the result of the following match
      match self {

        Term::Zero => "zero".to_string(),

        Term::True => "true".to_string(),

        Term::False => "false".to_string(),

        Term::Succ(term) => format!("succ({})", term),

        Term::Pred(term) => format!("pred({})", term),

        Term::If {guard, branch_true, branch_false} => {
          format!("if {} then {} else {}", guard, branch_true, branch_false)
        },

        Term::IsZero(term) => format!("iszero({})", term),

      };

    write!(f, "{}", text)
  }
}

impl Term {
  pub fn boxed(self) -> BTerm {
    Box::new(self)
  }
}
