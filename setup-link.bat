@echo off
REM setup-link.bat
REM Script to create a symbolic link from an Unreal Engine project to the SpacetimeDB_UnrealClient plugin
REM This allows for easier development and testing of the plugin

echo ====================================================
echo SpacetimeDB Unreal Client Plugin - Link Setup Script
echo ====================================================
echo.

REM Set hardcoded paths
SET "PLUGIN_PATH=%~dp0SpacetimeDB_UnrealClient"
SET "PROJECT_PATH=%~dp0STDB_UE_Example"

echo Project path: %PROJECT_PATH%
echo Plugin path: %PLUGIN_PATH%
echo.

REM Check if directories exist
if not exist "%PLUGIN_PATH%" (
    echo ERROR: Plugin directory not found at "%PLUGIN_PATH%"
    echo.
    pause
    exit /b 1
)

if not exist "%PROJECT_PATH%" (
    echo ERROR: Project directory not found at "%PROJECT_PATH%"
    echo Please ensure the STDB_UE_Example directory exists at: %PROJECT_PATH%
    echo.
    pause
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
        echo.
        echo ERROR: Failed to remove existing directory.
        echo Please remove it manually: %TARGET_PATH%
        echo.
        pause
        exit /b 1
    )
)

REM Create the symbolic link (requires admin privileges)
mklink /D "%TARGET_PATH%" "%PLUGIN_PATH%"

if %ERRORLEVEL% neq 0 (
    echo.
    echo ERROR: Failed to create symbolic link.
    echo Make sure you're running this script as administrator.
    echo.
    echo Alternatively, you can manually copy the plugin to your project using:
    echo xcopy /E /I /H "%PLUGIN_PATH%" "%TARGET_PATH%"
    echo.
    pause
    exit /b 1
)

echo.
echo ====================================================
echo Success! SpacetimeDB Unreal Client plugin linked.
echo ====================================================
echo.
echo Plugin linked to: %PROJECT_PATH%\Plugins\
echo.
echo IMPORTANT: You need to regenerate your project files 
echo and rebuild your project in your IDE.
echo.
echo ====================================================
echo.

pause
exit /b 0 