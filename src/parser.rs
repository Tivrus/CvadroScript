use crate::lexer::{Token};
use crate::ast::{Stmt, Expr, FunctionDef, Block, Type, Literal, BinOp, UnaryOp};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }
    
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut program = vec![];
        while !self.is_at_end() {
            program.push(self.declaration());
        }
        program
    }
    
    fn declaration(&mut self) -> Stmt {
        match self.peek() {
            Token::Def => self.function(),
            Token::Let => self.let_stmt(),
            _ => self.statement(),
        }
    }
    
    fn function(&mut self) -> Stmt {
        self.consume(&Token::Def);
        let name = self.consume_ident();
        self.consume(&Token::LParen);
        
        let mut params = vec![];
        while self.peek() != &Token::RParen {
            let name = self.consume_ident();
            self.consume(&Token::Colon);
            let type_ = self.parse_type();
            params.push((name, type_));
            
            if self.peek() == &Token::Comma {
                self.advance();
            }
        }
        
        self.consume(&Token::RParen);
        self.consume(&Token::Arrow);
        let return_type = self.parse_type();
        
        self.consume(&Token::Colon);
        self.consume(&Token::Newline);
        self.consume(&Token::Indent);
        
        let body = self.block();
        
        Stmt::Function(FunctionDef {
            name,
            params,
            return_type,
            body,
        })
    }
    
    fn let_stmt(&mut self) -> Stmt {
        self.consume(&Token::Let);
        let name = self.consume_ident();
        
        let type_annotation = if self.peek() == &Token::Colon {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        
        self.consume(&Token::Eq);
        let expr = self.expression();
        Stmt::Let(name, type_annotation, expr)
    }
    
    fn statement(&mut self) -> Stmt {
        match self.peek() {
            Token::Return => {
                self.advance();
                let expr = if self.peek() != &Token::Newline {
                    Some(self.expression())
                } else {
                    None
                };
                Stmt::Return(expr)
            },
            Token::Pass => {
                self.advance();
                Stmt::Pass
            },
            Token::If => self.if_stmt(),
            Token::While => self.while_stmt(),
            Token::For => self.for_stmt(),
            _ => {
                // Проверяем, не является ли это присваиванием
                if self.is_assignment() {
                    let name = self.consume_ident();
                    self.consume(&Token::Eq);
                    let expr = self.expression();
                    Stmt::Assign(name, expr)
                } else {
                    Stmt::Expr(self.expression())
                }
            }
        }
    }
    
    fn if_stmt(&mut self) -> Stmt {
        self.consume(&Token::If);
        let condition = self.expression();
        self.consume(&Token::Colon);
        self.consume(&Token::Newline);
        self.consume(&Token::Indent);
        let then_block = self.block();
        
        let else_block = if self.peek() == &Token::Else {
            self.advance();
            self.consume(&Token::Colon);
            self.consume(&Token::Newline);
            self.consume(&Token::Indent);
            Some(self.block())
        } else {
            None
        };
        
        Stmt::Expr(Expr::If(Box::new(condition), then_block, else_block))
    }
    
    fn while_stmt(&mut self) -> Stmt {
        self.consume(&Token::While);
        let condition = self.expression();
        self.consume(&Token::Colon);
        self.consume(&Token::Newline);
        self.consume(&Token::Indent);
        let body = self.block();
        
        Stmt::Expr(Expr::While(Box::new(condition), body))
    }
    
    fn for_stmt(&mut self) -> Stmt {
        self.consume(&Token::For);
        let var = self.consume_ident();
        self.consume(&Token::In);
        let iterable = self.expression();
        self.consume(&Token::Colon);
        self.consume(&Token::Newline);
        self.consume(&Token::Indent);
        let body = self.block();
        
        Stmt::Expr(Expr::For(var, Box::new(iterable), body))
    }
    
    fn block(&mut self) -> Block {
        let mut stmts = vec![];
        
        while self.peek() != &Token::Dedent && !self.is_at_end() {
            stmts.push(self.declaration());
            if self.peek() == &Token::Newline {
                self.advance();
            }
        }
        
        if self.peek() == &Token::Dedent {
            self.advance();
        }
        
        Block(stmts)
    }
    
    fn expression(&mut self) -> Expr {
        self.logical_or()
    }
    
    fn logical_or(&mut self) -> Expr {
        let mut expr = self.logical_and();
        
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.logical_and();
            expr = Expr::BinaryOp(Box::new(expr), BinOp::Or, Box::new(right));
        }
        
        expr
    }
    
    fn logical_and(&mut self) -> Expr {
        let mut expr = self.equality();
        
        while self.peek() == &Token::And {
            self.advance();
            let right = self.equality();
            expr = Expr::BinaryOp(Box::new(expr), BinOp::And, Box::new(right));
        }
        
        expr
    }
    
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        
        while matches!(self.peek(), Token::EqEq | Token::Ne) {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.comparison();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        expr
    }
    
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        
        while matches!(self.peek(), Token::Gt | Token::Ge | Token::Lt | Token::Le) {
            let op = match self.peek() {
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.term();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        expr
    }
    
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.factor();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        expr
    }
    
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.unary();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        expr
    }
    
    fn unary(&mut self) -> Expr {
        match self.peek() {
            Token::Not => {
                self.advance();
                Expr::UnaryOp(UnaryOp::Not, Box::new(self.unary()))
            },
            Token::Minus => {
                self.advance();
                Expr::UnaryOp(UnaryOp::Neg, Box::new(self.unary()))
            },
            _ => self.call(),
        }
    }
    
    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        
        while self.peek() == &Token::LParen {
            self.advance();
            let mut args = vec![];
            
            while self.peek() != &Token::RParen {
                args.push(self.expression());
                if self.peek() == &Token::Comma {
                    self.advance();
                }
            }
            
            self.consume(&Token::RParen);
            
            if let Expr::Variable(name) = expr {
                expr = Expr::Call(name, args);
            }
        }
        
        expr
    }
    
    fn primary(&mut self) -> Expr {
        match self.peek() {
            Token::True => {
                self.advance();
                Expr::Literal(Literal::Bool(true))
            },
            Token::False => {
                self.advance();
                Expr::Literal(Literal::Bool(false))
            },
            Token::Int(n) => {
                let n = *n;
                self.advance();
                Expr::Literal(Literal::Int(n))
            },
            Token::Float(f) => {
                let f = *f;
                self.advance();
                Expr::Literal(Literal::Float(f))
            },
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Expr::Literal(Literal::String(s))
            },
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Expr::Variable(name)
            },
            Token::LParen => {
                self.advance();
                let expr = self.expression();
                self.consume(&Token::RParen);
                expr
            },
            _ => panic!("Unexpected token: {:?}", self.peek()),
        }
    }
    
    fn parse_type(&mut self) -> Type {
        match self.peek() {
            Token::Ident(type_name) => {
                let type_name = type_name.clone();
                self.advance();
                match type_name.as_str() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "string" => Type::String,
                    "bool" => Type::Bool,
                    "void" => Type::Void,
                    _ => Type::Custom(type_name),
                }
            },
            _ => panic!("Expected type name"),
        }
    }
    
    // Вспомогательные методы
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek() == &Token::EOF
    }
    
    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::EOF)
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn consume(&mut self, expected: &Token) {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            self.advance();
        } else {
            panic!("Expected {:?}, found {:?}", expected, self.peek());
        }
    }
    
    fn consume_ident(&mut self) -> String {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                name
            },
            _ => panic!("Expected identifier"),
        }
    }
    
    fn is_assignment(&self) -> bool {
        if let Token::Ident(_) = self.peek() {
            if let Some(Token::Eq) = self.tokens.get(self.current + 1) {
                return true;
            }
        }
        false
    }
}