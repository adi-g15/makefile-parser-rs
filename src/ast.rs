use crate::nodes::ASTNode;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter, Write};
use std::path::{Path, PathBuf};

pub struct AST {
    /* Holds a global context... variables defined till now */
    pub context: Context,
    pub nodes: Vec<Box<dyn ASTNode>>,
}

impl Debug for AST {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("AST: \n{:?}", self.context))?;

        f.write_str("\n\tNodes:\n")?;

        for node in &self.nodes {
            f.write_str(&format!("\t\t{:?}\n", node))?;
        }

        Ok(())
    }
}

impl AST {
    pub fn new(root_makefile_parent_dir: &Path) -> Self {
        AST {
            context: Context::new(root_makefile_parent_dir.to_path_buf()),
            nodes: Vec::new(),
        }
    }

    pub fn push(&mut self, node: Box<dyn ASTNode>) {
        self.nodes.push(node);
    }
}

pub struct Context {
    modifiables: Vec<String>,
    simple_expanded: Vec<String>,
    mapping: BTreeMap<String, String>,
    pub root_makefile_dir: PathBuf,
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("\tContext: \n")?;
        for (k, v) in &self.mapping {
            f.write_str(&format!(
                "\t\t{}\t{}: {}\n",
                k,
                if self.modifiables.contains(&k) {
                    '?'
                } else if self.simple_expanded.contains(&k) {
                    ':'
                } else {
                    ' '
                },
                v
            ))?;
        }

        f.write_char('\n')
    }
}

impl Context {
    pub fn new(root_makefile_dir: PathBuf) -> Self {
        Context {
            root_makefile_dir,
            modifiables: Vec::new(),
            simple_expanded: Vec::new(),
            mapping: BTreeMap::new(),
        }
    }

    pub fn get(&self, var_name: &str) -> Option<&String> {
        self.mapping.get(var_name)
    }

    /**
     * @note If the key was already present, then this call will 'update' the
     * value, previous value is lost
     */
    pub fn set(&mut self, mut var_name: String, mut new_value: String) {
        if var_name.ends_with('?') {
            /* remove '?' from name */
            var_name.pop();

            self.modifiables.push(var_name.clone());
        }

        /* @ref: https://www.gnu.org/software/make/manual/html_node/Flavors.html */
        if var_name.ends_with(':') {
            /* remove ':' or '::' from name */
            var_name = var_name.trim_end_matches(':').to_string();

            self.simple_expanded.push(var_name.clone());
        }

        if var_name.ends_with('+') {
            /* remove '+' from name */
            var_name.pop();

            /* modify new_value by adding it to previous value if any exists */
            if let Some(old_value) = self.get(&var_name) {
                new_value = old_value.clone() + " " + &new_value;
            }
        }

        self.mapping.insert(var_name, new_value);
    }

    /**
     * @note If not present, then this call is simply a no-op
     */
    pub fn unset(&mut self, var_name: &str) {
        self.mapping.remove(var_name);
    }
}
