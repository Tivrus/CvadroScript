@echo off
echo Сборка компилятора CvadroScript...
cargo build --release

if %errorlevel% neq 0 (
    echo Ошибка сборки!
    pause
    exit /b 1
)

echo Компиляция run.cst...
target\release\cvadroscript.exe run.cst

if %errorlevel% neq 0 (
    echo Ошибка компиляции CvadroScript!
    pause
    exit /b 1
)

echo Компиляция в C++...
g++ -o run.exe run.cpp

if %errorlevel% neq 0 (
    echo Ошибка компиляции C++!
    echo Убедитесь, что у вас установлен g++
    pause
    exit /b 1
)

echo Готово! Запускаем программу:
echo ================================
run.exe
echo ================================
pause
