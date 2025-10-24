use crate::ast::{Expr, Program, Statement};
use std::collections::HashMap;

pub struct CodeGenerator {
    output: String,
    var_counter: usize,
    block_counter: usize,
    line_labels: Vec<String>,         // For goto statements
    var_ptrs: HashMap<usize, String>, // Track variable pointers (var_index -> ptr_name)
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            var_counter: 0,
            block_counter: 0,
            line_labels: Vec::new(),
            var_ptrs: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        // Create labels for each statement (for goto)
        self.create_line_labels(&program.statements);

        // First pass: collect all variables used in the program
        let used_vars = self.collect_used_variables(program);

        // Generate main function
        self.output.push_str("fn @main() -> i64 {\n");
        self.output.push_str("  entry:\n");

        // Allocate only the variables that are actually used
        for var_idx in used_vars {
            let ptr = format!("%var_ptr_{}", var_idx);
            self.output.push_str(&format!("    {} = alloc.ptr.stack i64\n", ptr));
            self.output.push_str(&format!("    store.i64 {}, 0\n", ptr));
            self.var_ptrs.insert(var_idx, ptr);
        }

        for (idx, stmt) in program.statements.iter().enumerate() {
            // Add label for this statement (for goto)
            if idx < self.line_labels.len() {
                let label = &self.line_labels[idx];
                if !label.is_empty() {
                    self.output.push_str(&format!("\n  {}:\n", label));
                }
            }

            self.generate_statement(stmt)?;
        }

        // Default return
        self.output.push_str("    ret.i64 0\n");
        self.output.push_str("}\n");

        Ok(self.output.clone())
    }

    fn collect_used_variables(&self, program: &Program) -> Vec<usize> {
        use std::collections::BTreeSet;
        let mut vars = BTreeSet::new();

        for stmt in &program.statements {
            self.collect_vars_from_statement(stmt, &mut vars);
        }

        vars.into_iter().collect()
    }

    fn collect_vars_from_statement(&self, stmt: &Statement, vars: &mut std::collections::BTreeSet<usize>) {
        match stmt {
            Statement::Assign { var_index, value } => {
                vars.insert(*var_index);
                self.collect_vars_from_expr(value, vars);
            },
            Statement::Input { var_index } => {
                vars.insert(*var_index);
            },
            Statement::PrintNum(expr) | Statement::PrintChar(expr) => {
                self.collect_vars_from_expr(expr, vars);
            },
            Statement::PrintNewline => {},
            Statement::Conditional { condition, body } => {
                self.collect_vars_from_expr(condition, vars);
                for s in body {
                    self.collect_vars_from_statement(s, vars);
                }
            },
            Statement::Goto(_) => {},
            Statement::Return(expr) => {
                self.collect_vars_from_expr(expr, vars);
            },
        }
    }

    fn collect_vars_from_expr(&self, expr: &Expr, vars: &mut std::collections::BTreeSet<usize>) {
        match expr {
            Expr::Number(_) => {},
            Expr::Var(index) => {
                vars.insert(*index);
            },
            Expr::Add(left, right) | Expr::Sub(left, right) | Expr::Mul(left, right) => {
                self.collect_vars_from_expr(left, vars);
                self.collect_vars_from_expr(right, vars);
            },
        }
    }

    fn create_line_labels(&mut self, statements: &[Statement]) {
        for i in 0..statements.len() {
            // Create label for statements that might be goto targets
            self.line_labels.push(format!("line_{}", i + 1));
        }
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Assign { var_index, value } => {
                let expr_var = self.generate_expr(value)?;
                // Store to memory location
                if let Some(ptr) = self.var_ptrs.get(var_index).cloned() {
                    self.output.push_str(&format!("    store.i64 {}, {}\n", ptr, expr_var));
                } else {
                    return Err(format!("Variable index {} out of range", var_index));
                }
                Ok(())
            },
            Statement::Input { var_index } => {
                // Read integer from stdin using scanf-like function
                // We'll create an external function declaration
                self.output.push_str("    ; TODO: call scanf to read input\n");
                self.output.push_str(&format!("    ; input to var[{}]\n", var_index));

                // For now, just store 0 as placeholder
                if let Some(ptr) = self.var_ptrs.get(var_index).cloned() {
                    self.output
                        .push_str(&format!("    ; (placeholder) store.i64 {}, 0\n", ptr));
                }
                Ok(())
            },
            Statement::PrintNum(expr) => {
                let expr_var = self.generate_expr(expr)?;
                self.output.push_str(&format!("    print {}\n", expr_var));
                Ok(())
            },
            Statement::PrintChar(expr) => {
                let expr_var = self.generate_expr(expr)?;
                // Print character - we use printchar which prints the ASCII/Unicode character
                self.output.push_str(&format!("    printchar {}\n", expr_var));
                Ok(())
            },
            Statement::PrintNewline => {
                // Print newline character (ASCII 10)
                let newline = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 10, 0\n", newline));
                self.output.push_str(&format!("    printchar {}\n", newline));
                Ok(())
            },
            Statement::Conditional { condition, body } => {
                let cond_var = self.generate_expr(condition)?;

                let then_block = format!("then_{}", self.block_counter);
                let else_block = format!("else_{}", self.block_counter);
                self.block_counter += 1;

                // Check if condition is zero (false)
                let is_zero = self.new_var();
                self.output
                    .push_str(&format!("    {} = eq.i64 {}, 0\n", is_zero, cond_var));

                self.output
                    .push_str(&format!("    br {}, {}, {}\n", is_zero, else_block, then_block));

                // Then block (when condition is NOT zero)
                self.output.push_str(&format!("\n  {}:\n", then_block));
                for s in body {
                    self.generate_statement(s)?;
                }
                self.output.push_str(&format!("    jmp {}\n", else_block));

                // Else block (continue)
                self.output.push_str(&format!("\n  {}:\n", else_block));
                Ok(())
            },
            Statement::Goto(line) => {
                if *line > 0 && *line <= self.line_labels.len() {
                    let label = &self.line_labels[*line - 1];
                    self.output.push_str(&format!("    jmp {}\n", label));
                    Ok(())
                } else {
                    Err(format!("Invalid goto line: {}", line))
                }
            },
            Statement::Return(expr) => {
                let expr_var = self.generate_expr(expr)?;
                self.output.push_str(&format!("    ret.i64 {}\n", expr_var));
                Ok(())
            },
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n) => {
                let var = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 {}, 0\n", var, n));
                Ok(var)
            },
            Expr::Var(index) => {
                // Load from memory
                if let Some(ptr) = self.var_ptrs.get(index).cloned() {
                    let loaded = self.new_var();
                    self.output.push_str(&format!("    {} = load.i64 {}\n", loaded, ptr));
                    Ok(loaded)
                } else {
                    Err(format!("Variable index {} out of range", index))
                }
            },
            Expr::Add(left, right) => {
                let left_var = self.generate_expr(left)?;
                let right_var = self.generate_expr(right)?;
                let result = self.new_var();
                self.output
                    .push_str(&format!("    {} = add.i64 {}, {}\n", result, left_var, right_var));
                Ok(result)
            },
            Expr::Sub(left, right) => {
                let left_var = self.generate_expr(left)?;
                let right_var = self.generate_expr(right)?;
                let result = self.new_var();
                self.output
                    .push_str(&format!("    {} = sub.i64 {}, {}\n", result, left_var, right_var));
                Ok(result)
            },
            Expr::Mul(left, right) => {
                let left_var = self.generate_expr(left)?;
                let right_var = self.generate_expr(right)?;
                let result = self.new_var();
                self.output
                    .push_str(&format!("    {} = mul.i64 {}, {}\n", result, left_var, right_var));
                Ok(result)
            },
        }
    }

    fn new_var(&mut self) -> String {
        let var = format!("%t{}", self.var_counter);
        self.var_counter += 1;
        var
    }
}
