use crate::ast::Context;
use crate::nodes::*;
use crate::stream::Stream;

pub trait Handler {
    /** When a handler doesn't need the context, pass None... else it is read-write */
    fn handle(line: &str, context: Option<&mut Context>) -> Box<dyn ASTNode>;
}

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

        loop {
            let line = stream.peek_next_line();

            if !line.starts_with('\t') {
                break;
            }

            if !line.trim().is_empty() {
                target_ast
                    .steps
                    .push(TargetStepHandler::handle(line, Some(context)));
            }

            /* read in next line */
            stream.read_line();
        }

        Box::new(target_ast)
    }
}

/* Handles steps and categorizing them into different types */
struct TargetStepHandler {}

impl Handler for TargetStepHandler {
    fn handle(line: &str, _c: Option<&mut Context>) -> Box<dyn ASTNode> {
        let line = line.trim();

        if line.starts_with('#') {
            CommentHandler::handle(line, None)
        } else {
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

/* handle where ./... some executable started */
pub struct ExecutableHandler {}

pub struct CommentHandler {}

impl Handler for CommentHandler {
    fn handle(line: &str, _c: Option<&mut Context>) -> Box<dyn ASTNode> {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            Box::new(Comment::new(trimmed))
        } else {
            panic!("CommentHandler: Can only handle lines starting with '#'");
        }
    }
}

/* handle 'cargo build', add as 'build dep' */
pub struct CargoCommandsHandler {}

pub struct ExportHandler {}

impl Handler for ExportHandler {
    fn handle(line: &str, context: Option<&mut Context>) -> Box<dyn ASTNode> {
        let context = context.expect(
            "ExportHandler requires the context to set or unset exported or unexported variables",
        );

        let (token, var_expr) = line
            .split_once(' ')
            .expect("Expected a name after 'export/unexport '");

        let var_expr = var_expr.trim();

        if token == "export" {
            // BUG: Makefile Line 90 && will get ignored
            let (var_name, var_value) = var_expr
                .split_once('=')
                .expect("Expected var=value expression after \"export\" token");

            context.set(var_name.to_string(), var_value.to_string());

            Box::new(ExportASTNode::new(
                var_name.to_string(),
                var_value.to_string(),
            ))
        } else if token == "unexport" {
            let var_name = var_expr;

            context.unset(var_name);
            Box::new(UnExportASTNode::new(var_name.to_string()))
        } else {
            panic!(
                "ExportHandler:\n\tUnknown token: {}\n\tLine: {}",
                token, line
            );
        }
    }
}
