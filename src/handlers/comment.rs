use super::Handler;
use crate::ast::Context;
use crate::nodes::{ASTNode, Comment};

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
