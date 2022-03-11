use crate::ast::Context;
use crate::handlers::*;
use crate::nodes::{ASTNode, IncludeASTNode};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;

/*`Stream` struct is both an ASTNode and a Handler */
pub struct Stream {
    /* An abstract class over file I/O to support include operations, without reading complete files */
    next_line: String,
    /* Each element is a pair of Line iterator, and file path */
    lineiterators_stack: Vec<(Lines<BufReader<File>>, PathBuf)>,
    pub eof: bool,
}

impl Stream {
    pub fn new(filename: &str) -> Self {
        /* Initialise an empty stream */
        let mut stream = Stream {
            next_line: String::new(),
            lineiterators_stack: Vec::new(),

            /* By default, we have not yet reached EOF */
            eof: false,
        };

        stream.include_file(filename); // read in first file

        stream
    }

    pub fn get_current_file(&self) -> Option<PathBuf> {
        self.lineiterators_stack.last().map(|pair| pair.1.clone())
    }

    /**
     * @brief Sets self.next_line to the next line
     * Or set EOF if no more lines can be read
     *
     * In either case, self.next_line will be overriden, or in latter case it will be emptied
     */
    fn read_in_next_line(&mut self) {
        self.next_line = loop {
            if self.lineiterators_stack.is_empty() {
                /* No more lines to read... so EOF */
                self.eof = true;

                /* Empty string */
                break String::new();
            }

            /* Treated as a stack, the most recently added will be read first */
            /* SAFETY: Just checked above that self.lineiterators_stack is NOT empty... so .last() cannot be None */
            let line_iter = &mut self.lineiterators_stack.last_mut().unwrap().0;

            /* loop until either we find a non-empty line, OR EOF is encountered */
            match line_iter.next() {
                Some(res) => {
                    let mut s = res.expect("Failed to read file");
                    if !s.trim().is_empty() {
                        /* If this line ends with a '\', read in the next line and join it, this may recurse deep depending on how many consecutive lines end with a '\' */
                        if !s.trim_start().starts_with('#') && s.ends_with('\\') {
                            s.pop(); // remove the '\' character
                            self.read_in_next_line();

                            if self.peek_next_line().trim().starts_with('#') {
                                #[cfg(debug_assertions)]
                                println!("⚠ Wierd syntax: Ignoring a comment line, since previous line ends at '\'. Line: {}", self.peek_next_line());

                                /* re-read next line, skipping the current one */
                                self.read_in_next_line();
                            }

                            /* If the next line that was read is empty... that means end this recursion, next line was empty.
                             * This check is needed, because self.read_in_next_line by default, ignores empty lines, so an '\' followed by an empty line will be skipped and read in some other next lines, while actually this line should logically end with the empty line also */
                            if self.peek_next_line().trim().is_empty() {
                                break s;
                            }

                            s += self.peek_next_line().trim();
                        }
                        break s;
                    }
                }
                None => {
                    /* No lines could be read from current iterator, so pop it from stack */
                    self.lineiterators_stack.pop();
                }
            };
        }
    }

    pub fn read_line(&mut self) -> String {
        /* Cannot move out of mutable borrowed values... ie. cannot move self.next_line, while self is a reference (mutable or immutable) */
        let old_line = self.next_line.clone();

        /*
         * @note: In case current statement is a include, don't read in next line from current file... the next line should be of the included file... so self.read_in_next_line() must be called inside Stream::include_file
         * */
        if !old_line.trim().starts_with("include") {
            self.read_in_next_line();
        }

        old_line
    }

    /* @note It will return same string as self.read_line(), just that the self.next_line will not change after this call... so this is kind of read-only no-updation version of self.read_line */
    pub fn peek_next_line(&self) -> &str {
        &self.next_line
    }

    /** @note: After this, the given filepath will be at top of files/line_iterators stack, so it will be the file to be read in next self.read_in_next_lines() calls*/
    fn include_file(&mut self, filepath: &str) {
        let file = File::open(filepath).expect("Failed to open file");
        let reader = BufReader::new(file);

        let line_iter = reader.lines();

        self.lineiterators_stack
            .push((line_iter, PathBuf::from(filepath)));

        // Read in next line from the newly included file
        self.read_in_next_line()
    }
}

impl Stream {
    pub fn handle(&mut self, line: &str, _c: Option<&mut Context>) -> Box<dyn ASTNode> {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.len() < 2 || !tokens[0].contains(&"include") {
            panic!("Stream handler handles input strings of form \"include \"filename\"\"");
        }

        if tokens.len() > 2 {
            let mut it = tokens.iter().skip(2);
            println!(
                "❗ Ignoring tokens after include statement: {:?}",
                it.collect::<Vec<&&str>>()
            );
        }

        /* generally a relative filepath is given in makefile */
        let filepath = tokens[1];
        self.include_file(filepath);

        Box::new(IncludeASTNode {
            include_path: filepath.to_string(),
        })
    }
}
