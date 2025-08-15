#[derive(Debug, PartialEq)]
pub enum Token {
    // Ключевые слова
    Def, If, Else, While, For, In, Return, Let, Extern, Struct, Pass, And, Or, Not, True, False, 
    
    // Идентификаторы и литералы
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    
    // Операторы
    Plus, Minus, Star, Slash, Percent,
    Eq, EqEq, Ne, Lt, Gt, Le, Ge,
    
    // Разделители
    LParen, RParen, Colon, Comma, Newline,
    Indent, Dedent, EOF,
    
    // Специальные
    Arrow, // ->
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            source: input.chars().collect(),
            pos: 0,
            indent_stack: vec![0],
        }
    }
    
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        match self.current() {
            None => Some(Token::EOF),
            Some('\n') => self.handle_indent(),
            Some('#') => {
                self.skip_comment();
                self.next_token()
            },
            Some(c) if c.is_alphabetic() || *c == '_' => self.read_identifier(),
            Some(c) if c.is_digit(10) => self.read_number(),
            Some('"') | Some('\'') => self.read_string(),
            Some('+') => { self.advance(); Some(Token::Plus) },
            Some('-') => self.read_minus_or_arrow(),
            Some('*') => { self.advance(); Some(Token::Star) },
            Some('/') => { self.advance(); Some(Token::Slash) },
            Some('%') => { self.advance(); Some(Token::Percent) },
            Some('=') => self.read_equals(),
            Some('!') => self.read_not_equals(),
            Some('<') => self.read_less(),
            Some('>') => self.read_greater(),
            Some('(') => { self.advance(); Some(Token::LParen) },
            Some(')') => { self.advance(); Some(Token::RParen) },
            Some(':') => { self.advance(); Some(Token::Colon) },
            Some(',') => { self.advance(); Some(Token::Comma) },
            _ => {
                self.advance();
                self.next_token()
            }
        }
    }
    
    fn handle_indent(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '\n'
        let mut indent_level = 0;
        
        // Считаем пробелы/табы
        while let Some(' ') | Some('\t') = self.current() {
            indent_level += 1;
            self.advance();
        }
        
        let current_indent = *self.indent_stack.last().unwrap();
        
        if indent_level > current_indent {
            self.indent_stack.push(indent_level);
            Some(Token::Indent)
        } else if indent_level < current_indent {
            self.indent_stack.pop();
            Some(Token::Dedent)
        } else {
            Some(Token::Newline)
        }
    }
    
    fn current(&self) -> Option<&char> {
        self.source.get(self.pos)
    }
    
    fn advance(&mut self) {
        self.pos += 1;
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() && *c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        while let Some(c) = self.current() {
            if *c == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn read_identifier(&mut self) -> Option<Token> {
        let start = self.pos;
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || *c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let ident: String = self.source[start..self.pos].iter().collect();
        Some(match ident.as_str() {
            "def" => Token::Def,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "let" => Token::Let,
            "extern" => Token::Extern,
            "struct" => Token::Struct,
            "pass" => Token::Pass,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "True" => Token::True,
            "False" => Token::False,
            _ => Token::Ident(ident),
        })
    }
    
    fn read_number(&mut self) -> Option<Token> {
        let start = self.pos;
        let mut is_float = false;
        
        while let Some(c) = self.current() {
            if c.is_digit(10) {
                self.advance();
            } else if *c == '.' && !is_float {
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let num_str: String = self.source[start..self.pos].iter().collect();
        Some(if is_float {
            Token::Float(num_str.parse().unwrap_or(0.0))
        } else {
            Token::Int(num_str.parse().unwrap_or(0))
        })
    }
    
    fn read_string(&mut self) -> Option<Token> {
        let quote = *self.current()?;
        self.advance(); // Пропускаем открывающую кавычку
        
        let mut string = String::new();
        while let Some(c) = self.current() {
            if *c == quote {
                self.advance(); // Пропускаем закрывающую кавычку
                break;
            }
            if *c == '\\' {
                self.advance();
                match self.current() {
                    Some('n') => string.push('\n'),
                    Some('t') => string.push('\t'),
                    Some('r') => string.push('\r'),
                    Some('\\') => string.push('\\'),
                    Some('"') => string.push('"'),
                    Some('\'') => string.push('\''),
                    Some(c) => string.push(*c),
                    None => break,
                }
            } else {
                string.push(*c);
            }
            self.advance();
        }
        Some(Token::String(string))
    }
    
    fn read_minus_or_arrow(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '-'
        if let Some('>') = self.current() {
            self.advance();
            Some(Token::Arrow)
        } else {
            Some(Token::Minus)
        }
    }
    
    fn read_equals(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '='
        if let Some('=') = self.current() {
            self.advance();
            Some(Token::EqEq)
        } else {
            Some(Token::Eq)
        }
    }
    
    fn read_not_equals(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '!'
        if let Some('=') = self.current() {
            self.advance();
            Some(Token::Ne)
        } else {
            // Неизвестный символ, пропускаем
            self.next_token()
        }
    }
    
    fn read_less(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '<'
        if let Some('=') = self.current() {
            self.advance();
            Some(Token::Le)
        } else {
            Some(Token::Lt)
        }
    }
    
    fn read_greater(&mut self) -> Option<Token> {
        self.advance(); // Пропускаем '>'
        if let Some('=') = self.current() {
            self.advance();
            Some(Token::Ge)
        } else {
            Some(Token::Gt)
        }
    }
}
}