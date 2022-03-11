use std::fmt::{Debug, Formatter};

/**
 * Ignore comments for now
 */

pub trait ASTNode
where
    Self: Debug,
{
}

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

pub struct Target {
    pub target_name: String,
    pub deps: Vec<String>,
    pub steps: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for Target {}

impl Debug for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("Target: {}\n", self.target_name))?;
        f.write_str(&format!("\t\t\tDeps: {:?}\n", self.deps))?;
        f.write_str(&format!("\t\t\tSteps:\n"))?;

        for (i, step) in self.steps.iter().enumerate() {
            f.write_str(&format!("\t\t\t\t{}: {:?}\n", i, step))?;
        }

        Ok(())
    }
}

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
    line: String,
}

impl ASTNode for TargetGenericStep {}

impl Debug for TargetGenericStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("{}", self.line))
    }
}

impl TargetGenericStep {
    pub fn new(line: String) -> Self {
        TargetGenericStep { line }
    }
}
