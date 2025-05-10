@echo off
REM Build script for SpacetimeDB Unreal Client Rust components
REM This builds the ClientModule for use with Unreal Engine on Windows

setlocal enabledelayedexpansion

echo Building SpacetimeDB Unreal Client Rust components...

REM Get the current directory (where this script is located)
set SCRIPT_DIR=%~dp0
echo Script directory: %SCRIPT_DIR%

REM Verify cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Rust and Cargo must be installed to build the SpacetimeDB Unreal Client.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Verify cxxbridge is installed
where cxxbridge >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo cxxbridge command not found. Installing cxxbridge-cmd...
    cargo install cxxbridge-cmd
    if %ERRORLEVEL% neq 0 (
        echo Failed to install cxxbridge-cmd.
        pause
        exit /b %ERRORLEVEL%
    )
)

REM Set build configuration based on UE_BUILD_CONFIGURATION env var if present
set CARGO_PROFILE=debug

echo Building with profile: %CARGO_PROFILE%

REM Change to ClientModule directory
cd "%SCRIPT_DIR%\ClientModule"

REM Create directory for CXX bridge headers if it doesn't exist
if not exist "target\cxxbridge" (
    mkdir "target\cxxbridge"
)

REM Generate CXX bridge headers
echo Generating CXX bridge headers...
cxxbridge src\ffi.rs --header > target\cxxbridge\ffi.h
if %ERRORLEVEL% neq 0 (
    echo Failed to generate CXX bridge headers.
    pause
    exit /b %ERRORLEVEL%
)

REM Build the rust library
echo Running cargo build with profile %CARGO_PROFILE%...
if "%CARGO_PROFILE%"=="debug" (
    cargo build
) else (
    cargo build --release
)

if %ERRORLEVEL% neq 0 (
    echo Failed to build Rust library.
    pause
    exit /b %ERRORLEVEL%
)

echo Build completed successfully!
pause
exit /b 0 