mod comment;
mod executable;
mod export;
mod ifeq;
mod target;

pub use comment::CommentHandler;
pub use export::ExportHandler;
pub use ifeq::IfHandler;
pub use target::{GenericStepHandler, TargetHandler};

use crate::ast::Context;
use crate::nodes::*;
use crate::stream::Stream;

/* As of now, not all handlers implement this trait, some have different arguments for handle function */
pub trait Handler {
    /** When a handler doesn't need the context, pass None... else it is read-write */
    fn handle(line: &str, context: Option<&mut Context>) -> Box<dyn ASTNode>;
}
