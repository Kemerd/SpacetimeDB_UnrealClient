@echo off
REM setup-spacetimedb.bat
REM Script to automate SpacetimeDB setup for Windows
REM This script installs all the necessary components for SpacetimeDB development

echo ====================================================
echo SpacetimeDB Setup Script for Windows
echo ====================================================
echo.

REM Check if Rust is installed
rustc --version > nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Rust is not installed. Installing Rust...
    echo This will download and run the rustup installer.
    echo.
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | cmd
    
    echo.
    echo Adding Rust to the current PATH...
    set PATH=%PATH%;%USERPROFILE%\.cargo\bin
) else (
    echo Rust is already installed. Updating...
    rustup update stable
)

echo.
echo Installing cxxbridge command...
cargo install cxxbridge-cmd

echo.
echo Checking for SpacetimeDB CLI...
spacetime --version > nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo SpacetimeDB CLI is not installed. Installing...
    echo Please follow the instructions at https://spacetimedb.com/install
    echo.
    echo After installation, run the following commands:
    echo.
    echo     spacetime start       # Start a local SpacetimeDB instance
    echo     spacetime login       # Log in to SpacetimeDB via GitHub
    echo.
) else (
    echo SpacetimeDB CLI is already installed.
    echo Starting local SpacetimeDB instance...
    start cmd /k "spacetime start"
    
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

pause 