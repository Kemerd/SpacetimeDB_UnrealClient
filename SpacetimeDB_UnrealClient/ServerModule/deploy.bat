@echo off
REM deploy.bat - Deploy the ServerModule to SpacetimeDB
REM
REM This script builds and deploys the ServerModule to a SpacetimeDB instance.
REM Usage: deploy.bat [database_name] [host]
REM
REM If database_name is not provided, it will use "unreal_game" as default.
REM If host is not provided, it will use "localhost:3000" as default.

setlocal EnableDelayedExpansion

REM Parse command line arguments
set DB_NAME=unreal_game
set HOST=localhost:3000

if not "%~1"=="" set DB_NAME=%~1
if not "%~2"=="" set HOST=%~2

echo ========================================
echo SpacetimeDB UnrealClient Deployment Tool
echo ========================================
echo Deploying ServerModule to:
echo Database: %DB_NAME%
echo Host:     %HOST%
echo ========================================

REM Check if spacetime CLI is installed
where spacetime >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: SpacetimeDB CLI not found!
    echo Please install it with: cargo install spacetime
    exit /b 1
)

REM Build the CustomServerModule first (since ServerModule depends on it)
echo Building CustomServerModule...
cd ..\CustomServerModule
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Error building CustomServerModule!
    exit /b 1
)

REM Build the ServerModule
echo Building ServerModule...
cd ..\ServerModule
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Error building ServerModule!
    exit /b 1
)

REM Check if the database exists, if not create it
echo Checking if database exists...
spacetime db list | findstr /C:"%DB_NAME%" >nul
if %ERRORLEVEL% neq 0 (
    echo Creating database %DB_NAME%...
    spacetime db create "%DB_NAME%"
    if %ERRORLEVEL% neq 0 (
        echo Error creating database!
        exit /b 1
    )
)

REM Deploy the module
echo Deploying module to SpacetimeDB...
spacetime db publish "%DB_NAME%" --host "%HOST%"
if %ERRORLEVEL% neq 0 (
    echo Error publishing module!
    exit /b 1
)

echo ========================================
echo Deployment completed successfully!
echo ========================================
echo You can now connect to your database at:
echo %HOST%/%DB_NAME%
echo ========================================

endlocal 