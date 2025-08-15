mod lexer;
mod ast;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::process::{self, Command};

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
    
    // Создаем временный файл
    let temp_cpp = "temp_cvadroscript.cpp";
    let temp_exe = "temp_cvadroscript.exe";
    
    match fs::write(temp_cpp, cpp_code) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("Ошибка создания временного файла: {}", err);
            process::exit(1);
        }
    }
    
    // Этап 4: Компиляция C++
    println!("Этап 4: Компиляция C++ кода...");
    let compile_result = Command::new("g++")
        .args(&["-o", temp_exe, temp_cpp])
        .output();
    
    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Ошибка компиляции C++:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                cleanup_files(temp_cpp, temp_exe);
                process::exit(1);
            }
        },
        Err(err) => {
            eprintln!("Ошибка запуска g++: {}", err);
            eprintln!("Убедитесь, что g++ установлен и доступен в PATH");
            cleanup_files(temp_cpp, temp_exe);
            process::exit(1);
        }
    }
    
    // Этап 5: Выполнение программы
    println!("Этап 5: Выполнение программы...");
    println!("================================");
    
    let run_result = if cfg!(target_os = "windows") {
        Command::new(format!(".\\{}", temp_exe)).output()
    } else {
        Command::new(format!("./{}", temp_exe)).output()
    };
    
    match run_result {
        Ok(output) => {
            // Выводим результат выполнения
            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            
            if !output.status.success() {
                eprintln!("Программа завершилась с ошибкой (код: {:?})", output.status.code());
            }
        },
        Err(err) => {
            eprintln!("Ошибка выполнения программы: {}", err);
        }
    }
    
    println!("================================");
    
    // Очистка временных файлов
    cleanup_files(temp_cpp, temp_exe);
    println!("Выполнение завершено!");
}

fn cleanup_files(cpp_file: &str, exe_file: &str) {
    let _ = fs::remove_file(cpp_file);
    let _ = fs::remove_file(exe_file);
}
