use regex::Regex;
use std::path::PathBuf;

use crate::handlers::*;

/* handle \w:*, and read in more lines to complete the target */
pub struct TargetHandler {}

impl TargetHandler {
    pub fn handle(line: &str, stream: &mut Stream, context: &mut Context) -> Box<dyn ASTNode> {
        /* handle \w:*, and read in more lines to complete the target */
        let (target_name, dependencies) = line
            .split_once(':')
            .expect("TargetHandler: Expected ':' after target name");

        let target_name = target_name.trim_end(); // remove any leading space after target name

        let mut deps = Vec::new();
        let dependencies = dependencies.trim();

        for dependency in dependencies.split_whitespace() {
            deps.push(dependency.to_string());
        }

        let mut target_ast = Target {
            target_name: target_name.to_string(),
            defined_in: stream.get_current_file().expect(
                "TargetHandler: If a target was read, then there must be a file from it was read",
            ),
            deps,
            steps: Vec::new(),
        };

        /* To keep track of `cd` statements, will be helpful to get relative locations later in cargo subcommands */
        let mut current_dir = context.root_makefile_dir.clone();

        loop {
            let line = stream.peek_next_line();

            if !line.starts_with('\t') {
                break;
            }

            if !line.trim().is_empty() {
                target_ast.steps.push(TargetStepHandler::handle(
                    line,
                    Some(context),
                    &mut current_dir,
                ));
            }

            /* read in next line */
            stream.read_line();
        }

        Box::new(target_ast)
    }
}

/* Handles steps and categorizing them into different types */
pub struct GenericStepHandler {}

impl GenericStepHandler {
    pub fn handle(line: &str, stream: &mut Stream, context: &mut Context) -> Box<dyn ASTNode> {
        let line = line.trim();

        let regex_target = Regex::new(r"\w:.*$").unwrap();
        let regex_variable = Regex::new(r"\w+ *[\?:\+]?=").unwrap();

        if line.starts_with('#') {
            CommentHandler::handle(line, None)
        } else if line.starts_with("export") || line.starts_with("unexport") {
            /* NOTE: export statements must be handled before regex_variable, as it will regex_variable will also match 'export ...=...' */
            ExportHandler::handle(line, Some(context))
        } else if regex_target.is_match(line) {
            TargetHandler::handle(line, stream, context)
        } else {
            Box::new(TargetGenericStep::new(line.to_string()))
        }
    }
}

/* Handles steps and categorizing them into different types */
struct TargetStepHandler {}

impl TargetStepHandler {
    fn handle(
        line: &str,
        context: Option<&mut Context>,
        current_dir: &mut PathBuf,
    ) -> Box<dyn ASTNode> {
        let line = line.trim();

        if line.starts_with('#') {
            CommentHandler::handle(line, None)
        } else if line.starts_with("export") || line.starts_with("unexport") {
            /* NOTE: export statements must be handled before regex_variable, as it will regex_variable will also match 'export ...=...' */
            ExportHandler::handle(line, context)
        } else if line.starts_with("cargo") {
            let mut it = line.split_whitespace().skip(1);

            let subcommand = it
                .next()
                .expect("TargetStepHandler: cargo: Expected a cargo subcommand");

            let manifest_path = line
                .split_once("--manifest-path")
                .map(|(_, second_part)|
                    /* `second_part` contains the manifest path, just after --manifest-path, ie. first word in `second_part` */
                    second_part
                        .trim_start()
                        .split_whitespace()
                        .next()
                        .expect("Expected a manifest path (Cargo.toml filepath), after --manifest-path")
                );

            let root_makefile_dir = context
                .as_ref()
                .expect("Expected reference to Context to get root makefile directory")
                .root_makefile_dir
                .clone();

            let directory = match manifest_path {
                Some(p) => current_dir
                    .join(p)
                    .strip_prefix(root_makefile_dir)
                    .ok()
                    .map(|p| {
                        p.parent()
                            .expect(&format!(
                                "ERROR: Manifest path doesn't have a parent: {:?}",
                                &p
                            ))
                            .to_str()
                            .expect("Expected UTF-8 encoded filenames")
                            .to_string()
                    }),
                None => None,
            };

            // #[cfg(debug_assertions)]
            // {
            // println!("Directory: {:?}", directory);
            // println!("CurrentPath: {:?}", current_dir);
            // println!("Manifest: {:?}", manifest_path);
            // println!("Root: {:?}", context.unwrap().root_makefile_dir);
            // }

            match subcommand {
                "build" => Box::new(Cargo {
                    subcommand: CargoSubCommand::BUILD,
                    directory,
                    complete_cmd: line.to_string(),
                }),
                "clean" => Box::new(Cargo {
                    subcommand: CargoSubCommand::CLEAN,
                    directory,
                    complete_cmd: line.to_string(),
                }),
                "run" => Box::new(Cargo {
                    subcommand: CargoSubCommand::RUN,
                    directory,
                    complete_cmd: line.to_string(),
                }),
                "update" => Box::new(Cargo {
                    subcommand: CargoSubCommand::UPDATE_DEPS,
                    directory,
                    complete_cmd: line.to_string(),
                }),
                _ => {
                    println!(
                        "⚠ Unknown cargo subcommand: {}... treating as simple string",
                        subcommand
                    );

                    Box::new(TargetGenericStep::new(line.to_string()))
                }
            }
        } else {
            /* Handle case of `cd` specially */
            if line.starts_with("cd") {
                let new_path = current_dir.join(
                    line.split_whitespace()
                        .skip(1)
                        .next() /* first word after cd statement */
                        .expect("Expected path after `cd` statement"),
                );

                if new_path.is_dir() {
                    /* replace current_dir's value with new_path, if it is valid, else ignore */
                    println!("Changed to {}", new_path.display());
                    current_dir.push(new_path);
                } else {
                    println!("Failed to cd into {}", new_path.display());
                    /* Ignoring a 'cd' */
                }
            }

            Box::new(TargetGenericStep::new(line.to_string()))
        }

        /*

        BUG: Makefile Line 90 is problematic for this... ie. export PATH=... && command, then it sets value of PATH as "\"value\" && command", and I cannot think of a better way to handle '&&'

        else if line.starts_with("export") || line.starts_with("unexport") {
            /* NOTE: export statements must be handled before regex_variable, as it will regex_variable will also match 'export ...=...' */
            ExportHandler::handle(line, Some(_c.expect("TargetStepHandler: export/unexport: Handling these requires access to the context, pass Some(context) instead of None")))
        }*/
    }
}
