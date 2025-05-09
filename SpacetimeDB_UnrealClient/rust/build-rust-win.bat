@echo off
setlocal

echo Building SpacetimeDB Rust client for Windows...

:: Check if Rust is installed
where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Rust is not installed. Please install Rust from https://rustup.rs/
    exit /b 1
)

:: Check if cxxbridge is installed
where cxxbridge >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo cxxbridge is not installed. Installing...
    cargo install cxxbridge-cmd
    if %ERRORLEVEL% neq 0 (
        echo Failed to install cxxbridge. Please install manually: cargo install cxxbridge-cmd
        exit /b 1
    )
)

:: Navigate to the Rust directory
cd %~dp0

:: Build the Rust library
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Failed to build Rust library
    exit /b 1
)

:: Generate C++ headers
cxxbridge --header > stdb.hpp
if %ERRORLEVEL% neq 0 (
    echo Failed to generate C++ headers
    exit /b 1
)

echo SpacetimeDB Rust client built successfully!
exit /b 0 