#![feature(option_result_contains)]
use std::{env, path::Path, process::exit};

use regex::Regex;

mod ast;
mod handlers;
mod nodes;
mod stream;

use ast::AST;
use handlers::*;
use nodes::*;
use stream::Stream;

// https://users.rust-lang.org/t/show-value-only-in-debug-mode/43686/2
macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                dbg!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}

macro_rules! debugln {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                println!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}

fn main() {
    let mut args = env::args().skip(1); // Skip first argument (which is executable path)

    let makefile = match args.next() {
        Some(arg) => arg,
        None => {
            println!("Usage: ./makefile-parser path/to/Makefile");
            exit(22 /* EINVAL */);
        }
    };

    #[cfg(debug_assertions)]
    println!("Changing directory to: {:?}", Path::new(&makefile).parent());

    let root_dir = Path::new(&makefile)
        .parent()
        .expect("Failed to get parent directory of given Makefile path");

    std::env::set_current_dir(root_dir).expect("Failed to change directory");

    // starting with Makefile in $(cwd)
    let makefile = Path::new(&makefile)
        .file_stem()
        .expect("Given path must have a filename at end")
        .to_str()
        .expect("Path must be UTF-8 encoded characters only");

    let mut stream = Stream::new(makefile);
    let mut ast = AST::new(root_dir);

    let regex_target = Regex::new(r"\w:.*$").unwrap();
    let regex_variable = Regex::new(r"\w+ *[\?:\+]?=").unwrap();

    while stream.eof == false {
        let l = stream.read_line();
        let line = l.trim();

        // debug!(line);

        if line.starts_with('#') {
            ast.push(CommentHandler::handle(line, None));
        } else if line.starts_with("export") || line.starts_with("unexport") {
            /* NOTE: export statements must be handled before regex_variable, as it will regex_variable will also match 'export ...=...' */
            ast.nodes
                .push(ExportHandler::handle(line, Some(&mut ast.context)));
        } else if regex_variable.is_match(line) {
            // Modify context
            /* SAFETY: Regex matched so, it is of the form ARCH?=x86... so split at '=' must return Some() */
            let (var_name, var_value) = line.split_once('=').unwrap();

            let var_name = var_name.to_string();
            let var_value = var_value.to_string();

            ast.context.set(var_name, var_value);
        } else if line.starts_with("include") {
            ast.push(stream.handle(line, None));
        } else if regex_target.is_match(line) {
            let t = TargetHandler::handle(line, &mut stream, &mut ast.context);

            ast.push(t);
        } else if line.starts_with("ifeq") {
            let ifnode = IfHandler::handle(line, &mut stream, &mut ast.context);

            ast.push(ifnode);
        } else {
            println!("❗ Unhandled: {}", line);
        }
    }

    println!("{:?}", ast);
}
