use std::{
    fmt::{Debug, Formatter, Write},
    path::PathBuf,
};

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

#[derive(Debug)]
pub enum CargoSubCommand {
    BUILD,
    CLEAN,
    RUN,
    UPDATE_DEPS,
}

pub struct Cargo {
    pub subcommand: CargoSubCommand,
    pub complete_cmd: String,
    pub directory: Option<String>, // `None` signifies Self, building/cleaning the current directory
}

impl Debug for Cargo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!(
            "Cargo: {:?} {}\n",
            self.subcommand,
            self.directory.as_ref().unwrap_or(&"".to_string())
        ))?;
        f.write_str(&format!("\t\t\t\tOriginal: {:?}\n", self.complete_cmd))
    }
}

impl ASTNode for Cargo {}

pub struct Target {
    pub target_name: String,
    pub deps: Vec<String>,
    pub defined_in: PathBuf,
    pub steps: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for Target {}

impl Debug for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("Target: {}\n", self.target_name))?;
        f.write_str(&format!("\t\t\tDeps: {:?}\n", self.deps))?;
        f.write_str(&format!("\t\t\tDefined in: {:?}\n", self.defined_in))?;
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

pub struct IfASTNode {
    pub condition: String,
    pub is_not_eq: bool, // is it 'ifneq' (true) or 'ifeq' (false)
    pub steps: Vec<Box<dyn ASTNode>>,
    pub elseif_: Option<Box<IfASTNode>>,
    pub else_: Option<Box<ElseASTNode>>,
}

pub struct ElseASTNode {
    pub steps: Vec<Box<dyn ASTNode>>,
}

impl ASTNode for IfASTNode {}
impl ASTNode for ElseASTNode {}

impl Debug for IfASTNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!(
            "If{}: {}\n",
            if self.is_not_eq { "NotEq" } else { "Eq" },
            self.condition
        ))?;
        f.write_str(&format!("\t\t\tSteps:\n"))?;

        for (i, step) in self.steps.iter().enumerate() {
            f.write_str(&format!("\t\t\t\t{}: {:?}\n", i, step))?;
        }

        if let Some(elseif_) = &self.elseif_ {
            f.write_str(&format!("\t\tElse {:?}", elseif_))?;
        }

        if let Some(else_) = &self.else_ {
            f.write_str(&format!("\t\t{:?}", else_))?;
        }

        Ok(())
    }
}

impl Debug for ElseASTNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("Else:\n"))?;
        f.write_str(&format!("\t\t\tSteps:\n"))?;

        for (i, step) in self.steps.iter().enumerate() {
            f.write_str(&format!("\t\t\t\t{}: {:?}\n", i, step))?;
        }

        Ok(())
    }
}

pub struct TargetGenericStep {
    line: String,
}

impl ASTNode for TargetGenericStep {}

impl Debug for TargetGenericStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut it = self.line.split("&&");

        /* SAFETY: First .split().next() must always succeed, given a non-empty string */
        let command1 = it.next().unwrap();
        let command2 = it.next();

        match command2 {
            None => {
                /* Single command */
                f.write_str(&format!("{}", command1))
            }
            Some(command2) => {
                /* Multiple commands separated with '&&' */
                f.write_str(&format!("{} && \\", command1))?;
                f.write_str(&format!("\n\t\t\t\t{}", command2))?;

                while let Some(command) = it.next() {
                    f.write_str(&format!("&& \\\n\t\t\t\t{}", command))?;
                }

                f.write_char('\n')
            }
        }
    }
}

impl TargetGenericStep {
    pub fn new(line: String) -> Self {
        TargetGenericStep { line }
    }
}
