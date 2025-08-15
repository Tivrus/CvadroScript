mod lexer;
mod ast;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use codegen::CodeGen;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Использование: cvadroscript <файл.cst>");
        process::exit(1);
    }
    
    let filename = &args[1];
    
    // Проверяем расширение файла
    if !filename.ends_with(".cst") {
        eprintln!("Ошибка: файл должен иметь расширение .cst");
        process::exit(1);
    }
    
    // Читаем исходный код
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Ошибка чтения файла {}: {}", filename, err);
            process::exit(1);
        }
    };
    
    println!("Компилируем файл: {}", filename);
    
    // Этап 1: Лексический анализ
    println!("Этап 1: Лексический анализ...");
    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();
    
    loop {
        match lexer.next_token() {
            Some(token) => {
                if token == lexer::Token::EOF {
                    tokens.push(token);
                    break;
                }
                tokens.push(token);
            },
            None => break,
        }
    }
    
    println!("Найдено {} токенов", tokens.len());
    
    // Этап 2: Синтаксический анализ
    println!("Этап 2: Синтаксический анализ...");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    println!("Создано AST с {} выражениями", ast.len());
    
    // Этап 3: Генерация кода
    println!("Этап 3: Генерация C++ кода...");
    let cpp_code = CodeGen::generate(ast);
    
    // Создаем выходной файл
    let output_filename = filename.replace(".cst", ".cpp");
    match fs::write(&output_filename, cpp_code) {
        Ok(_) => println!("C++ код сохранен в: {}", output_filename),
        Err(err) => {
            eprintln!("Ошибка записи в файл {}: {}", output_filename, err);
            process::exit(1);
        }
    }
    
    println!("Компиляция завершена успешно!");
    println!("Для компиляции C++ файла используйте:");
    println!("g++ -o {} {}", filename.replace(".cst", ""), output_filename);
}
