@echo off
REM setup-spacetimedb.bat
REM Script to automate SpacetimeDB setup for Windows
REM This script installs all the necessary components for SpacetimeDB development

echo ====================================================
echo SpacetimeDB Setup Script for Windows
echo ====================================================
echo.

SETLOCAL EnableDelayedExpansion

REM Check if Rust is installed
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Rust is not installed. Installing Rust...
    echo This will download the rustup-init.exe installer.
    echo.
    
    REM Download rustup-init.exe instead of piping curl to cmd
    echo Downloading rustup-init.exe...
    curl -sSfO https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
    
    echo Running rustup-init.exe...
    rustup-init.exe -y --default-toolchain stable --profile default
    
    echo.
    echo Adding Rust to the current PATH...
    
    REM Set the PATH to include Cargo and Rust (without requiring restart)
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    
    REM Clean up the installer
    del rustup-init.exe
    
    echo Rust installation complete.
) else (
    echo Rust is already installed. Updating...
    rustup update stable
)

REM Verify that cargo is in the PATH and working
echo.
echo Checking for Cargo...
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Cargo not found in PATH despite Rust being installed.
    echo This may indicate an issue with your Rust installation.
    echo.
    echo Please make sure "%USERPROFILE%\.cargo\bin" is in your PATH.
    echo You may need to restart your terminal or computer.
    echo.
    echo After ensuring Cargo is available, run this script again.
    pause
    exit /b 1
) else (
    echo Cargo is available. Proceeding with setup...
)

echo.
echo Installing cxxbridge command...
cargo install cxxbridge-cmd

echo.
echo Checking for SpacetimeDB CLI...
where spacetime >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo SpacetimeDB CLI is not installed. Installing...
    echo Please visit https://spacetimedb.com/install in your browser
    echo to download and install the SpacetimeDB CLI.
    echo.
    echo After installation, run the following commands:
    echo.
    echo     spacetime start       # Start a local SpacetimeDB instance
    echo     spacetime login       # Log in to SpacetimeDB via GitHub
    echo.
    pause
) else (
    echo SpacetimeDB CLI is already installed.
    echo Starting local SpacetimeDB instance...
    start cmd /c "spacetime start"
    
    echo.
    echo In a new terminal window, log in to SpacetimeDB with:
    echo     spacetime login
)

echo.
echo ====================================================
echo SpacetimeDB setup is complete
echo ====================================================
echo.
echo For local development:
echo  - In a terminal window, run: spacetime start
echo  - In another terminal, run: spacetime login
echo.
echo To create a new project:
echo  - Run: spacetime init --lang rust
echo  - or: spacetime init --lang csharp
echo.
echo Docs: https://spacetimedb.com/docs
echo.
echo ====================================================

ENDLOCAL
pause 