@echo off
setlocal enabledelayedexpansion

REM Set paths - modify these as needed
SET "PLUGIN_PATH=%~dp0SpacetimeDB_UnrealClient"
SET "PROJECT_PATH=%~1"

if "%PROJECT_PATH%"=="" (
    echo Usage: setup-link.bat [path\to\UE_Example]
    echo Example: setup-link.bat C:\Projects\UE_Example
    exit /b 1
)

REM Check if directories exist
if not exist "%PLUGIN_PATH%" (
    echo Error: Plugin directory not found at "%PLUGIN_PATH%"
    exit /b 1
)

if not exist "%PROJECT_PATH%" (
    echo Error: Project directory not found at "%PROJECT_PATH%"
    exit /b 1
)

REM Create Plugins directory if it doesn't exist
if not exist "%PROJECT_PATH%\Plugins" (
    echo Creating Plugins directory...
    mkdir "%PROJECT_PATH%\Plugins"
)

REM Create symbolic link
echo Creating symbolic link...
SET "TARGET_PATH=%PROJECT_PATH%\Plugins\SpacetimeDB_UnrealClient"

REM Remove existing link/directory if it exists
if exist "%TARGET_PATH%" (
    echo Removing existing link or directory...
    rmdir "%TARGET_PATH%" 2>nul
    if exist "%TARGET_PATH%" (
        echo Failed to remove existing directory. Please remove it manually.
        exit /b 1
    )
)

REM Create the symbolic link (requires admin privileges)
mklink /D "%TARGET_PATH%" "%PLUGIN_PATH%"

if %ERRORLEVEL% neq 0 (
    echo Failed to create symbolic link. Make sure you're running as administrator.
    echo.
    echo You can manually copy the plugin to your project using:
    echo xcopy /E /I /H "%PLUGIN_PATH%" "%TARGET_PATH%"
    exit /b 1
)

echo.
echo Successfully linked SpacetimeDB_UnrealClient plugin to %PROJECT_PATH%\Plugins\
echo.
echo IMPORTANT: You will need to regenerate project files and rebuild your project.
echo.

exit /b 0 