@echo off
echo Установка CvadroScript...

:: Проверяем, установлен ли Rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Ошибка: Rust не установлен!
    echo Пожалуйста, установите Rust с https://rustup.rs/
    pause
    exit /b 1
)

:: Проверяем, установлен ли g++
where g++ >nul 2>nul
if %errorlevel% neq 0 (
    echo Ошибка: g++ не установлен!
    echo Пожалуйста, установите MinGW или Visual Studio Build Tools
    pause
    exit /b 1
)

:: Собираем проект
echo Сборка CvadroScript компилятора...
cargo build --release
if %errorlevel% neq 0 (
    echo Ошибка сборки!
    pause
    exit /b 1
)

:: Создаем директорию для установки
set INSTALL_DIR=%USERPROFILE%\.cvadroscript
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

:: Копируем исполняемый файл
echo Установка в %INSTALL_DIR%...
copy target\release\cvadroscript.exe "%INSTALL_DIR%\cvadros.exe"

:: Создаем bat файл для удобного запуска
echo @echo off > "%INSTALL_DIR%\cvadros.bat"
echo "%INSTALL_DIR%\cvadros.exe" %%* >> "%INSTALL_DIR%\cvadros.bat"

echo.
echo ========================================
echo CvadroScript успешно установлен!
echo ========================================
echo.
echo Для использования добавьте в PATH: %INSTALL_DIR%
echo.
echo Или выполните следующую команду в PowerShell (от администратора):
echo [Environment]::SetEnvironmentVariable("Path", $env:Path + ";%INSTALL_DIR%", "User")
echo.
echo После добавления в PATH вы сможете использовать:
echo cvadros your_file.cst
echo.
echo Для ручного добавления в PATH:
echo 1. Откройте "Системные переменные среды"
echo 2. Нажмите "Переменные среды"
echo 3. В разделе "Переменные пользователя" найдите "Path"
echo 4. Добавьте: %INSTALL_DIR%
echo.
pause
