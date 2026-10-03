/*!

If the executable is given a file name as an argument, it will attempt to use the file as input.
Otherwise, it will read input from STDIN.

*/

mod parse;
mod ast;
mod evaluate;

use std::io::{Error as IOError, Write};
use std::path::PathBuf;

use auto_args::AutoArgs;

use evaluate::evaluate;
use parse::parse;


#[derive(AutoArgs)]
struct CommandLineArguments {
  /// The source file. If omitted, input is read from STDIN.
  input_file: Option<PathBuf>,
}

fn main() -> Result<(), IOError>{
  let args = CommandLineArguments::from_args();
  let source_text: String;

  source_text = // the value of the following `if`:
    if let Some(file_path) = args.input_file {
      std::fs::read_to_string(file_path)?
    } else {
      print!("arith> ");
      std::io::stdout().flush()?; // Ensure prompt is printed before input is read.
      let mut input = String::new();
      std::io::stdin().read_line(&mut input)?;
      input
    };

  // We have source_text
  match parse(source_text.as_str()) {
    Ok(term) => {
      println!("Parsed as: {}", term);

      let evaluated = evaluate(term);
      println!("Evaluated to: {}", evaluated);
    }
    Err(e) => {
      println!("Could not parse input. {}", e);
    }
  };

  // This `Ok` indicates the input was readable, not that the input was valid code.
  Ok(())
}
