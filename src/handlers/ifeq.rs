use time::Instant;

use crate::ast::Context;
use crate::handlers::GenericStepHandler;
use crate::nodes::{ElseASTNode, IfASTNode};
use crate::stream::Stream;

// https://users.rust-lang.org/t/show-value-only-in-debug-mode/43686/2
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

/* handle if else conditions */
pub struct IfHandler {}

impl IfHandler {
    pub fn handle(line: &str, stream: &mut Stream, context: &mut Context) -> Box<IfASTNode> {
        let line = line.trim();

        let condition = line
            .split_once(' ')
            .expect("Expected space between \"ifeq\" and a condition")
            .1
            .to_string();

        let mut if_node = IfASTNode {
            condition,
            elseif_: None,
            else_: None,
            steps: Vec::new(),
        };

        /* Current `next_line` will be storing the line just next to passed `line` which is something like 'ifeq ...', so we are done with passed `line` (condition known) */
        let mut next_line = stream.peek_next_line().trim_start().to_string();

        let start = Instant::now();
        let mut line_count = 0;
        loop {
            debugln!("Line: {}", &next_line);
            if next_line.starts_with("endif") {
                /* endif encountered, current line is `endif`, so read in next line (ie. our work done) and exit */
                stream.read_line();
                break;
            }

            if next_line.starts_with("else") {
                let mut line = String::new();

                /* loop to join all but first word/token in `next_line` */
                for token in next_line.split_whitespace().skip(1) {
                    line += token;
                    line += " ";
                }

                /* Read in next line before recursing */
                stream.read_line();

                if line.starts_with("ifeq") {
                    /* else-ifeq block (with 'else' token removed)*/
                    if_node.elseif_ = Some(IfHandler::handle(&line, stream, context));
                } else {
                    /* Simple else block - Just read in the lines in else blocks */

                    let mut else_ = ElseASTNode { steps: Vec::new() };
                    loop {
                        let next_line = stream.peek_next_line().trim_start().to_string();

                        if next_line.starts_with("endif") {
                            /* endif encountered, if condition ends */
                            break;
                        }

                        #[cfg(debug_assertions)]
                        {
                            line_count += 1;
                        }
                        else_
                            .steps
                            .push(GenericStepHandler::handle(&next_line, stream, context));
                        stream.read_line();
                    }
                    if_node.else_ = Some(Box::new(else_));
                }
                break; // leave the outer loop too, since else block is handled, and else ifeq will recursively reach else too
            }

            #[cfg(debug_assertions)]
            {
                line_count += 1;
            }
            if_node
                .steps
                .push(GenericStepHandler::handle(&next_line, stream, context));

            stream.read_line();
            next_line = stream.peek_next_line().trim().to_string();
        }

        debugln!(
            "Duration in ifeq loop: {} Lines => {:?}",
            line_count,
            Instant::now() - start
        );

        Box::new(if_node)
    }
}
