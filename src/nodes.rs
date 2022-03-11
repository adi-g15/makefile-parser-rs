use std::fmt::Debug;

/**
 * Ignore comments for now
 */

pub trait ASTNode where Self: Debug {}

#[derive(Debug)]
pub struct Comment {
    comment: String,
}

impl Comment {
    pub fn new(comment: &str) -> Self {
        Comment {
            comment: comment.to_string(),
        }
    }
}

impl ASTNode for Comment {}

#[derive(Debug)]
pub struct Target {
    pub target_name: String,
    pub steps: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for Target {}

#[derive(Debug)]
pub struct IncludeASTNode {
    pub include_path: String,
}

impl ASTNode for IncludeASTNode {}

#[derive(Debug)]
pub struct ExportASTNode {
    name: String,
    value: String,
}

impl ExportASTNode {
    pub fn new(name: String, value: String) -> Self {
        ExportASTNode { name, value }
    }
}

impl ASTNode for ExportASTNode {}

#[derive(Debug)]
pub struct UnExportASTNode {
    name: String,
}

impl UnExportASTNode {
    pub fn new(name: String) -> Self {
        UnExportASTNode { name }
    }
}

impl ASTNode for UnExportASTNode {}

pub struct TargetGenericStep {
    // line: 
}
