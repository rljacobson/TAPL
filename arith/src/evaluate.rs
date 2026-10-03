/*!

Evaluates the expression given by an AST.

The evaluation order differs from Pierce. In Pierce, evaluation is "little step": after every
reduction, evaluation is restarted. We evaluate recursively instead.

*/

use std::ops::Deref;

use crate::ast::{BTerm, Term};

/// Evaluates the given term.
pub fn evaluate(term: BTerm) -> BTerm {
  match *term {
    Term::Zero
    | Term::True
    | Term::False => term,

    Term::Succ(inner) => {
      // Note that `succ(pred(zero))` reduces to `succ(zero)`, not `zero`, because `pred(zero)` is reduced first, and
      // `pred` is saturating.
      let child = evaluate(inner);
      Term::Succ(child).boxed()
    }

    Term::Pred(inner) => {
      let child = evaluate(inner);
      match *child {

        Term::Succ(grand_child) => grand_child,

        // `pred` is saturating
        Term::Zero => Term::Zero.boxed(),

        other => Term::Pred(other.boxed()).boxed()

      }
    }

    Term::If { guard, branch_true, branch_false } => {
      // `guard` is always evaluated
      let guard = evaluate(guard);
      match *guard {

        Term::True => evaluate(branch_true),

        Term::False => evaluate(branch_false),

        // Neither branch is evaluated
        otherwise => {
          Term::If {
            guard: otherwise.boxed(),
            branch_true,
            branch_false
          }.boxed()
        }

      }
    }

    Term::IsZero(inner) => {
      let child = evaluate(inner);
      match *child {

        Term::Zero => Term::True.boxed(),

        Term::Succ(grandchild) if is_numeric(grandchild.as_ref()) => Term::False.boxed(),

        otherwise => Term::IsZero(otherwise.boxed()).boxed()

      }
    }
  }
}


fn is_numeric<T>(term: T) -> bool
  where T: Deref<Target = Term>
{
  match term.deref() {

    Term::Zero => true,

    Term::Succ(child) => is_numeric(child.as_ref()),

    _ => false

  }
}
