use crate::ast::{Expr, Program, Statement};
use std::collections::HashMap;

pub struct CodeGenerator {
    output: String,
    var_counter: usize,
    block_counter: usize,
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
            var_ptrs: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        // First pass: collect all variables used in the program
        let used_vars = self.collect_used_variables(program);

        // Determine max line number to create labels for ALL lines
        let max_line = program.statements.iter()
            .map(|(_, line)| *line)
            .max()
            .unwrap_or(1);

        // Create a map of line_number -> statement_index
        let mut line_to_stmt: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for (idx, (_, line_num)) in program.statements.iter().enumerate() {
            line_to_stmt.insert(*line_num, idx);
        }

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

        // Add jump to first line if we have statements
        if !program.statements.is_empty() {
            let first_line = program.statements[0].1;
            self.output.push_str(&format!("    jmp line_{}\n", first_line));
        }

        let mut last_needs_terminator = true;
        let mut current_line = 1;

        for (idx, (stmt, line_num)) in program.statements.iter().enumerate() {
            // Add labels for all lines from current_line to line_num
            while current_line <= *line_num {
                self.output.push_str(&format!("\n  line_{}:\n", current_line));

                // If this line has a statement, generate it
                if current_line == *line_num {
                    let needs_jump = self.generate_statement(stmt)?;
                    last_needs_terminator = needs_jump;

                    // Add fall-through jump to next statement's line if needed
                    if needs_jump && idx + 1 < program.statements.len() {
                        let next_line = program.statements[idx + 1].1;
                        self.output.push_str(&format!("    jmp line_{}\n", next_line));
                    }
                } else {
                    // Empty line - just jump to next line
                    if current_line < max_line {
                        self.output.push_str(&format!("    jmp line_{}\n", current_line + 1));
                    }
                }

                current_line += 1;
            }
        }

        // Add default return only if last statement needs it
        if last_needs_terminator {
            self.output.push_str("    ret.i64 0\n");
        }
        self.output.push_str("}\n");

        Ok(self.output.clone())
    }

    fn collect_used_variables(&self, program: &Program) -> Vec<usize> {
        use std::collections::BTreeSet;
        let mut vars = BTreeSet::new();

        for (stmt, _) in &program.statements {
            Self::collect_vars_from_statement(stmt, &mut vars);
        }

        vars.into_iter().collect()
    }

    fn collect_vars_from_statement(stmt: &Statement, vars: &mut std::collections::BTreeSet<usize>) {
        match stmt {
            Statement::Assign { var_index, value } => {
                vars.insert(*var_index);
                Self::collect_vars_from_expr(value, vars);
            },
            Statement::Input { var_index } => {
                vars.insert(*var_index);
            },
            Statement::PrintNum(expr) | Statement::PrintChar(expr) => {
                Self::collect_vars_from_expr(expr, vars);
            },
            Statement::PrintNewline => {},
            Statement::Conditional { condition, body } => {
                Self::collect_vars_from_expr(condition, vars);
                for s in body {
                    Self::collect_vars_from_statement(s, vars);
                }
            },
            Statement::Goto(_) => {},
            Statement::Return(expr) => {
                Self::collect_vars_from_expr(expr, vars);
            },
        }
    }

    fn collect_vars_from_expr(expr: &Expr, vars: &mut std::collections::BTreeSet<usize>) {
        match expr {
            Expr::Number(_) => {},
            Expr::Var(index) => {
                vars.insert(*index);
            },
            Expr::Add(left, right) | Expr::Sub(left, right) | Expr::Mul(left, right) => {
                Self::collect_vars_from_expr(left, vars);
                Self::collect_vars_from_expr(right, vars);
            },
        }
    }


    fn generate_statement(&mut self, stmt: &Statement) -> Result<bool, String> {
        match stmt {
            Statement::Assign { var_index, value } => {
                let expr_var = self.generate_expr(value)?;
                // Store to memory location
                if let Some(ptr) = self.var_ptrs.get(var_index).cloned() {
                    self.output.push_str(&format!("    store.i64 {}, {}\n", ptr, expr_var));
                } else {
                    return Err(format!("Variable index {} out of range", var_index));
                }
                Ok(true) // Needs fall-through jump
            },
            Statement::Input { var_index } => {
                // Read an integer from stdin (multiple digits until newline/space)
                // Use readint instruction if available, otherwise implement digit parsing

                // For now, use a simple implementation:
                // Call a runtime helper function that reads an integer
                // Since Lamina doesn't have readint, we'll generate inline code

                let skip_ws = format!("input_skip_ws_{}", self.block_counter);
                let read_start = format!("input_start_{}", self.block_counter);
                let read_loop = format!("input_loop_{}", self.block_counter);
                let read_done = format!("input_done_{}", self.block_counter);
                self.block_counter += 1;

                // Allocate accumulator
                let acc_ptr = self.new_var();
                self.output.push_str(&format!("    {} = alloc.ptr.stack i64\n", acc_ptr));
                self.output.push_str(&format!("    store.i64 {}, 0\n", acc_ptr));

                // Allocate byte storage
                let byte_ptr = self.new_var();
                self.output.push_str(&format!("    {} = alloc.ptr.stack i64\n", byte_ptr));
                self.output.push_str(&format!("    store.i64 {}, 0\n", byte_ptr));

                self.output.push_str(&format!("    jmp {}\n", skip_ws));

                // Skip whitespace
                self.output.push_str(&format!("\n  {}:\n", skip_ws));
                let ws_byte = self.new_var();
                self.output.push_str(&format!("    {} = readbyte\n", ws_byte));
                self.output.push_str(&format!("    store.i64 {}, {}\n", byte_ptr, ws_byte));

                let space_val = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 32, 0\n", space_val));
                let is_space = self.new_var();
                self.output.push_str(&format!("    {} = eq.i64 {}, {}\n", is_space, ws_byte, space_val));

                let newline_val = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 10, 0\n", newline_val));
                let is_newline = self.new_var();
                self.output.push_str(&format!("    {} = eq.i64 {}, {}\n", is_newline, ws_byte, newline_val));

                // Check if is_space OR is_newline (a + b > 0 since they're booleans)
                let is_ws = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 {}, {}\n", is_ws, is_space, is_newline));

                self.output.push_str(&format!("    br {}, {}, {}\n", is_ws, skip_ws, read_start));

                // Start reading number
                self.output.push_str(&format!("\n  {}:\n", read_start));
                self.output.push_str(&format!("    jmp {}\n", read_loop));

                // Read loop
                self.output.push_str(&format!("\n  {}:\n", read_loop));
                let curr_byte = self.new_var();
                self.output.push_str(&format!("    {} = load.i64 {}\n", curr_byte, byte_ptr));

                // Check if digit (48-57)
                // digit = curr_byte - 48, is_digit = (digit >= 0 && digit <= 9)
                let ascii_zero = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 48, 0\n", ascii_zero));
                let digit_val = self.new_var();
                self.output.push_str(&format!("    {} = sub.i64 {}, {}\n", digit_val, curr_byte, ascii_zero));

                // Check if digit_val < 0 (digit_val + 100 < 100, since we work with unsigned comparison)
                // Actually, let's check if digit_val >= 0 && digit_val <= 9
                // is_digit = digit_val in [0,9]
                // We can check: is_zero = (digit_val == 0), ..., is_nine = (digit_val == 9)
                // OR: check digit_val < 10 and digit_val >= 0

                // Simple approach: check each value 0-9
                let zero = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 0, 0\n", zero));
                let is_zero = self.new_var();
                self.output.push_str(&format!("    {} = eq.i64 {}, {}\n", is_zero, digit_val, zero));

                let mut is_digit_acc = is_zero.clone();
                for i in 1..=9 {
                    let val = self.new_var();
                    self.output.push_str(&format!("    {} = add.i64 {}, 0\n", val, i));
                    let is_val = self.new_var();
                    self.output.push_str(&format!("    {} = eq.i64 {}, {}\n", is_val, digit_val, val));
                    let new_acc = self.new_var();
                    self.output.push_str(&format!("    {} = add.i64 {}, {}\n", new_acc, is_digit_acc, is_val));
                    is_digit_acc = new_acc;
                }

                let is_digit = is_digit_acc;

                self.output.push_str(&format!("    br {}, {}_proc, {}\n", is_digit, read_loop, read_done));

                // Process digit (use digit_val already computed)
                self.output.push_str(&format!("\n  {}_proc:\n", read_loop));
                let old_acc = self.new_var();
                self.output.push_str(&format!("    {} = load.i64 {}\n", old_acc, acc_ptr));
                let ten = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 10, 0\n", ten));
                let acc_times_10 = self.new_var();
                self.output.push_str(&format!("    {} = mul.i64 {}, {}\n", acc_times_10, old_acc, ten));

                let new_acc = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 {}, {}\n", new_acc, acc_times_10, digit_val));
                self.output.push_str(&format!("    store.i64 {}, {}\n", acc_ptr, new_acc));

                // Read next byte
                let next_byte = self.new_var();
                self.output.push_str(&format!("    {} = readbyte\n", next_byte));
                self.output.push_str(&format!("    store.i64 {}, {}\n", byte_ptr, next_byte));
                self.output.push_str(&format!("    jmp {}\n", read_loop));

                // Done
                self.output.push_str(&format!("\n  {}:\n", read_done));
                let final_val = self.new_var();
                self.output.push_str(&format!("    {} = load.i64 {}\n", final_val, acc_ptr));

                if let Some(ptr) = self.var_ptrs.get(var_index).cloned() {
                    self.output.push_str(&format!("    store.i64 {}, {}\n", ptr, final_val));
                } else {
                    return Err(format!("Variable index {} out of range", var_index));
                }
                Ok(true)
            },
            Statement::PrintNum(expr) => {
                let expr_var = self.generate_expr(expr)?;
                self.output.push_str(&format!("    print {}\n", expr_var));
                Ok(true) // Needs fall-through jump
            },
            Statement::PrintChar(expr) => {
                let expr_var = self.generate_expr(expr)?;
                // Print character using writebyte instruction
                let result = self.new_var();
                self.output
                    .push_str(&format!("    {} = writebyte {}\n", result, expr_var));
                Ok(true) // Needs fall-through jump
            },
            Statement::PrintNewline => {
                // Print newline character (ASCII 10)
                let newline = self.new_var();
                self.output.push_str(&format!("    {} = add.i64 10, 0\n", newline));
                let result = self.new_var();
                self.output
                    .push_str(&format!("    {} = writebyte {}\n", result, newline));
                Ok(true) // Needs fall-through jump
            },
            Statement::Conditional { condition, body } => {
                let cond_var = self.generate_expr(condition)?;

                let then_block = format!("then_{}", self.block_counter);
                let else_block = format!("else_{}", self.block_counter);
                self.block_counter += 1;

                // Check if condition is zero (execute when zero)
                let is_zero = self.new_var();
                self.output
                    .push_str(&format!("    {} = eq.i64 {}, 0\n", is_zero, cond_var));

                // Branch: if zero go to then_block, else go to else_block
                self.output
                    .push_str(&format!("    br {}, {}, {}\n", is_zero, then_block, else_block));

                // Then block (when condition IS zero)
                self.output.push_str(&format!("\n  {}:\n", then_block));
                let mut last_needs_jump = true;
                for s in body {
                    last_needs_jump = self.generate_statement(s)?;
                    // Statements in conditional body are in the same block, no fall-through needed
                }
                // Only add jump to else if the last statement needs it (not a goto/return)
                if last_needs_jump {
                    self.output.push_str(&format!("    jmp {}\n", else_block));
                }

                // Else block (continue)
                self.output.push_str(&format!("\n  {}:\n", else_block));
                Ok(true) // Needs fall-through jump
            },
            Statement::Goto(line) => {
                if *line > 0 {
                    self.output.push_str(&format!("    jmp line_{}\n", line));
                    Ok(false) // Already has terminator, no fall-through needed
                } else {
                    Err(format!("Invalid goto line: {}", line))
                }
            },
            Statement::Return(expr) => {
                let expr_var = self.generate_expr(expr)?;
                self.output.push_str(&format!("    ret.i64 {}\n", expr_var));
                Ok(false) // Already has terminator, no fall-through needed
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
