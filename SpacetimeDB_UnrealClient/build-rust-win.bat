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
    exit /b 1
)

REM Set build configuration based on UE_BUILD_CONFIGURATION env var if present
if "%UE_BUILD_CONFIGURATION%"=="Debug" (
    set CARGO_PROFILE=debug
) else if "%UE_BUILD_CONFIGURATION%"=="Development" (
    set CARGO_PROFILE=release
) else if "%UE_BUILD_CONFIGURATION%"=="Shipping" (
    set CARGO_PROFILE=release
) else (
    set CARGO_PROFILE=release
)

echo Building with profile: %CARGO_PROFILE%

REM Change to ClientModule directory
cd "%SCRIPT_DIR%\ClientModule"

REM Build the rust library
echo Running cargo build with profile %CARGO_PROFILE%...
if "%CARGO_PROFILE%"=="debug" (
    cargo build
) else (
    cargo build --release
)

if %ERRORLEVEL% neq 0 (
    echo Failed to build Rust library.
    exit /b %ERRORLEVEL%
)

echo Build completed successfully!
exit /b 0 