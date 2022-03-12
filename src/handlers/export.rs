use super::Handler;
use crate::ast::Context;
use crate::nodes::{ASTNode, ExportASTNode, UnExportASTNode};

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
