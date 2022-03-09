#![feature(option_result_contains)]
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
    process::exit,
};

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

/**
 * Ignore comments for now
 */

trait ASTNode {}

struct AST {
    /* @adig - Don't include include statements in this */
    nodes: Vec<Box<dyn ASTNode>>,
    context: Context,
}

impl AST {
    pub fn new() -> Self {
        AST {
            nodes: Vec::new(),
            context: Context::new(),
        }
    }

    pub fn push(&mut self, node: Box<dyn ASTNode>) {
        self.nodes.push(node);
    }
}

struct Context {
    /* Holds a global context... variables defined till now */
    mapping: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            mapping: HashMap::new(),
        }
    }

    pub fn get(&self, var_name: &str) -> Option<&String> {
        self.mapping.get(var_name)
    }

    /**
     * @note If the key was already present, then this call will 'update' the
     * value, previous value is lost
     */
    pub fn set(&mut self, var_name: &str, new_value: String) {
        self.mapping.insert(var_name.to_string(), new_value);
    }

    /**
     * @note If not present, then this call is simply a no-op
     */
    pub fn unset(&mut self, var_name: &str) {
        self.mapping.remove(var_name);
    }
}

trait Handler {
    /** When a handler doesn't need the context, pass None... else it is read-write */
    fn handle(&self, line: &str, context: Option<&mut Context>) -> Box<dyn ASTNode>;
}

/*Handlers */
struct Stream {
    /* An abstract class over file I/O to support include operations, without reading complete files */
    next_line: String,
    lineiterators_stack: Vec<Lines<BufReader<File>>>, // each line iterator refers to a file
    eof: bool,
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
        stream.read_in_next_line(); // read in first line

        stream
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
            let line_iter = self.lineiterators_stack.last_mut().unwrap();

            /* loop until either we find a non-empty line, OR EOF is encountered */
            match line_iter.next() {
                Some(res) => {
                    let s = res.expect("Failed to read file");
                    if !s.trim().is_empty() {
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
         * @note: Even if self.eof = true after this. The stream can again have self.eof = false, IF the current `next_line` is an include statement... so in next handle calls it will add another line iterator to the stack
         * */
        if ! old_line.trim().starts_with("include") {
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
        let reader = io::BufReader::new(file);

        let line_iter = reader.lines();

        self.lineiterators_stack.push(line_iter);
        /* TODO: Decide if needed to self.read_in_next_line */
    }
}

struct IncludeASTNode {
    pub include_path: String,
}

impl ASTNode for IncludeASTNode {}

impl Stream {
    fn handle(&mut self, line: &str, _c: Option<&mut Context>) -> Box<dyn ASTNode> {
        let tokens: Vec<&str> = line.split(' ').collect();

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

struct Comment {
    comment: String,
}

impl ASTNode for Comment {}

impl Comment {
    pub fn handle(line: &str) -> Box<dyn ASTNode> {
        /* quick and dirty way to allow outsiders to call without needing a 'Comment' object */
        Comment {
            comment: String::new(),
        }
        .handle(line, None)
    }
}

impl Handler for Comment {
    fn handle(&self, line: &str, _c: Option<&mut Context>) -> Box<dyn ASTNode> {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            /* NOTE: .starts_with() can also just be removed, using only strip_prefix, but kept for readability */
            /* SAFETY: .unwrap() is safe, because in this if block, it is already known that string has the prefix '#' */
            let comment = trimmed.strip_prefix('#').unwrap();

            Box::new(Comment {
                comment: comment.trim().to_string(),
            })
        } else {
            panic!("Comment handler can only handle lines starting with '#'");
        }
    }
}

struct Target {
    pub target_name: String,
    pub steps: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for Target {}

/* handle 'cargo build', add as 'build dep' */
struct CargoCommandsHandler {}

/* handle where ./... some executable started */
struct ExecutableHandler {}

/* handle \w:*, and read in more lines to complete the target */
struct TargetHandler {}

impl TargetHandler {
    pub fn handle(line: &str, stream: &mut Stream) -> Box<dyn ASTNode> {
        /* handle \w:*, and read in more lines to complete the target */
        let target_name = String::new();

        let target_ast = Target {
            target_name,
            steps: Vec::new(),
        };

        /* TODO: line is not at indent of 4, then done */
        loop {
            let line = stream.peek_next_line();

            if !line.starts_with('\t') {
                break;
            }

            /* read in next line */
            stream.read_line();
        }

        Box::new(target_ast)
    }
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

    std::env::set_current_dir(
        Path::new(&makefile)
            .parent()
            .expect("Failed to get parent directory of given Makefile path"),
    )
    .expect("Failed to change directory");

    // starting with Makefile in $(cwd)
    let makefile = Path::new(&makefile)
        .file_stem()
        .expect("Given path must have a filename at end")
        .to_str()
        .expect("Path must be UTF-8 encoded characters only");

    let mut stream = Stream::new(makefile);
    let mut ast = AST::new();

    while stream.eof == false {
        let l = stream.read_line();
        let line = l.trim();

        debug!(line);

        if line.starts_with('#') {
            ast.push(Comment::handle(line));
        } else if line.starts_with("include") {
            ast.push(stream.handle(line, None));
        } else if true
        /* check if line matches '\w:*' */
        {
            ast.push(TargetHandler::handle(line, &mut stream));
        } else {
            println!("❗ Unhandled: {}", line);
        }
    }
}
