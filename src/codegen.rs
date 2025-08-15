use crate::ast::{Stmt, Expr, FunctionDef, Block, Type, Literal, BinOp, UnaryOp};

pub struct CodeGen {
    output: String,
    indent_level: usize,
}

impl CodeGen {
    pub fn generate(ast: Vec<Stmt>) -> String {
        let mut gen = CodeGen {
            output: String::new(),
            indent_level: 0,
        };
        gen.gen_program(ast);
        gen.output
    }
    
    fn gen_program(&mut self, stmts: Vec<Stmt>) {
        self.output.push_str("#include <iostream>\n");
        self.output.push_str("#include <string>\n\n");
        self.output.push_str("// Вспомогательная функция print\n");
        self.output.push_str("template<typename T>\n");
        self.output.push_str("void print(T value) {\n");
        self.output.push_str("    std::cout << value << std::endl;\n");
        self.output.push_str("}\n\n");
        
        let mut has_main = false;
        for stmt in &stmts {
            if let Stmt::Function(func) = stmt {
                if func.name == "main" {
                    has_main = true;
                }
            }
        }
        
        for stmt in stmts {
            self.gen_stmt(&stmt);
        }
        
        // Если нет функции main, создаем её и добавляем туда все глобальные выражения
        if !has_main {
            self.output.push_str("int main() {\n");
            self.output.push_str("    return 0;\n");
            self.output.push_str("}\n");
        }
    }
    
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.push_line(&format!("{};", self.gen_expr(expr)));
            },
            Stmt::Let(name, _type, expr) => {
                let expr_code = self.gen_expr(expr);
                self.push_line(&format!("auto {} = {};", name, expr_code));
            },
            Stmt::Assign(name, expr) => {
                let expr_code = self.gen_expr(expr);
                self.push_line(&format!("{} = {};", name, expr_code));
            },
            Stmt::Function(func) => {
                self.gen_function(func);
            },
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.push_line(&format!("return {};", self.gen_expr(expr)));
                } else {
                    self.push_line("return;");
                }
            },
            Stmt::Pass => {
                self.push_line("// pass");
            }
        }
    }
    
    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => self.gen_literal(lit),
            Expr::Variable(name) => name.clone(),
            Expr::BinaryOp(left, op, right) => {
                let left_code = self.gen_expr(left);
                let right_code = self.gen_expr(right);
                let op_str = self.gen_binop(op);
                format!("({} {} {})", left_code, op_str, right_code)
            },
            Expr::UnaryOp(op, expr) => {
                let expr_code = self.gen_expr(expr);
                let op_str = self.gen_unaryop(op);
                format!("({}{})", op_str, expr_code)
            },
            Expr::Call(name, args) => {
                let args_code: Vec<String> = args.iter()
                    .map(|arg| self.gen_expr(arg))
                    .collect();
                format!("{}({})", name, args_code.join(", "))
            },
            Expr::If(condition, then_block, else_block) => {
                let cond_code = self.gen_expr(condition);
                self.push_line(&format!("if ({}) {{", cond_code));
                self.indent_level += 1;
                self.gen_block(then_block);
                self.indent_level -= 1;
                
                if let Some(else_block) = else_block {
                    self.push_line("} else {");
                    self.indent_level += 1;
                    self.gen_block(else_block);
                    self.indent_level -= 1;
                }
                self.push_line("}");
                "".to_string() // If-выражения не возвращают значение в этой версии
            },
            Expr::While(condition, body) => {
                let cond_code = self.gen_expr(condition);
                self.push_line(&format!("while ({}) {{", cond_code));
                self.indent_level += 1;
                self.gen_block(body);
                self.indent_level -= 1;
                self.push_line("}");
                "".to_string()
            },
            Expr::For(var, iterable, body) => {
                // Простая реализация for - предполагаем, что iterable - это range
                let iter_code = self.gen_expr(iterable);
                self.push_line(&format!("for (auto {} : {}) {{", var, iter_code));
                self.indent_level += 1;
                self.gen_block(body);
                self.indent_level -= 1;
                self.push_line("}");
                "".to_string()
            }
        }
    }
    
    fn gen_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Bool(b) => b.to_string(),
        }
    }
    
    fn gen_binop(&self, op: &BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
    
    fn gen_unaryop(&self, op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        }
    }
    
    fn gen_block(&mut self, block: &Block) {
        for stmt in &block.0 {
            self.gen_stmt(stmt);
        }
    }
    
    fn gen_function(&mut self, f: &FunctionDef) {
        // Конвертация типов cvadroscript → C++
        let return_type = self.type_to_cpp(&f.return_type);
        
        let params = f.params.iter()
            .map(|(name, ty)| format!("{} {}", self.type_to_cpp(ty), name))
            .collect::<Vec<_>>()
            .join(", ");
        
        self.push_line(&format!("{} {}({}) {{", return_type, f.name, params));
        self.indent_level += 1;
        
        // Генерация тела функции
        self.gen_block(&f.body);
        
        self.indent_level -= 1;
        self.push_line("}");
        self.push_line(""); // Пустая строка после функции
    }
    
    fn type_to_cpp(&self, ty: &Type) -> &'static str {
        match ty {
            Type::Int => "int",
            Type::Float => "double",
            Type::String => "std::string",
            Type::Bool => "bool", 
            Type::Void => "void",
            Type::Custom(_) => "auto", // Для пользовательских типов используем auto
        }
    }
    
    fn push_line(&mut self, line: &str) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }
}