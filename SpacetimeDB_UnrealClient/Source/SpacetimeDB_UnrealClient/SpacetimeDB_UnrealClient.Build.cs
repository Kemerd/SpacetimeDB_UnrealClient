// Copyright SpacetimeDB. All Rights Reserved.

using System;
using System.IO;
using System.Diagnostics;
using UnrealBuildTool;

public class SpacetimeDB_UnrealClient : ModuleRules
{
    public SpacetimeDB_UnrealClient(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;
        
        PublicIncludePaths.AddRange(
            new string[] {
                // ... add public include paths required here ...
            }
        );
        
        PrivateIncludePaths.AddRange(
            new string[] {
                // ... add private include paths required here ...
            }
        );
        
        PublicDependencyModuleNames.AddRange(
            new string[]
            {
                "Core",
                "CoreUObject",
                "Engine",
                "InputCore",
                "Networking",
                "Sockets",
                "Json",
                "JsonUtilities"
                // ... add other public dependencies that you statically link with here ...
            }
        );
        
        PrivateDependencyModuleNames.AddRange(
            new string[]
            {
                // ... add private dependencies that you statically link with here ...
            }
        );
        
        DynamicallyLoadedModuleNames.AddRange(
            new string[]
            {
                // ... add any modules that your module loads dynamically here ...
            }
        );
        
        // Only build Rust code if not in a CI environment that has pre-built the libraries
        // This can be controlled by setting an environment variable like SPACETIMEDB_SKIP_RUST_BUILD
        bool skipRustBuild = Environment.GetEnvironmentVariable("SPACETIMEDB_SKIP_RUST_BUILD") == "1";
        
        if (!skipRustBuild)
        {
            // Determine build script based on platform
            string rustBuildScript = "";
            string libExtension = "";
            
            if (Target.Platform == UnrealTargetPlatform.Win64)
            {
                rustBuildScript = "build-rust-win.bat";
                libExtension = "lib";
            }
            else if (Target.Platform == UnrealTargetPlatform.Mac)
            {
                rustBuildScript = "build-rust.sh";
                libExtension = "a";
            }
            else if (Target.Platform == UnrealTargetPlatform.Linux)
            {
                rustBuildScript = "build-rust.sh";
                libExtension = "a";
            }
            else
            {
                throw new BuildException("Unsupported platform for SpacetimeDB Unreal Client");
            }
            
            // Full path to the build script
            string rustBuildScriptPath = Path.Combine(ModuleDirectory, "../../", rustBuildScript);
            
            // Make sure the script is executable on Unix platforms
            if (Target.Platform != UnrealTargetPlatform.Win64)
            {
                try
                {
                    Process chmod = new Process();
                    chmod.StartInfo.FileName = "/bin/chmod";
                    chmod.StartInfo.Arguments = $"+x \"{rustBuildScriptPath}\"";
                    chmod.StartInfo.UseShellExecute = false;
                    chmod.Start();
                    chmod.WaitForExit();
                }
                catch (Exception e)
                {
                    Console.WriteLine($"Warning: Failed to set executable permission on build script: {e.Message}");
                }
            }
            
            // Add pre-build step to build Rust library
            PreBuildSteps.Add(rustBuildScriptPath);
            
            // Log information about the build
            Console.WriteLine($"SpacetimeDB: Running Rust build script: {rustBuildScriptPath}");
        }
        else
        {
            Console.WriteLine("SpacetimeDB: Skipping Rust build as SPACETIMEDB_SKIP_RUST_BUILD is set");
        }
        
        // Add Rust library path and header include path
        string rustTargetPath = Path.Combine(ModuleDirectory, "../../ClientModule/target");
        string configFolder = (Target.Configuration == UnrealTargetConfiguration.Debug ||
                              Target.Configuration == UnrealTargetConfiguration.DebugGame) ? "debug" : "release";
        string rustLibPath = Path.Combine(rustTargetPath, configFolder);
        
        // Determine the correct library name based on platform
        string libName = "";
        if (Target.Platform == UnrealTargetPlatform.Win64)
        {
            libName = "stdb_client.lib";
        }
        else if (Target.Platform == UnrealTargetPlatform.Mac || Target.Platform == UnrealTargetPlatform.Linux)
        {
            libName = "libstdb_client.a";
        }
        
        // Add the Rust library to link against
        string fullLibPath = Path.Combine(rustLibPath, libName);
        if (File.Exists(fullLibPath))
        {
            PublicAdditionalLibraries.Add(fullLibPath);
            Console.WriteLine($"SpacetimeDB: Adding Rust library: {fullLibPath}");
        }
        else
        {
            // In CI/CD environments, we might have pre-built libs in a different location
            string ciLibPath = Environment.GetEnvironmentVariable("SPACETIMEDB_RUST_LIB_PATH");
            if (!string.IsNullOrEmpty(ciLibPath) && File.Exists(Path.Combine(ciLibPath, libName)))
            {
                PublicAdditionalLibraries.Add(Path.Combine(ciLibPath, libName));
                Console.WriteLine($"SpacetimeDB: Adding CI/CD provided Rust library from: {ciLibPath}");
            }
            else
            {
                // Still add the path even if the file doesn't exist yet, as it will be created during build
                PublicAdditionalLibraries.Add(fullLibPath);
                Console.WriteLine($"SpacetimeDB: Expected Rust library will be built at: {fullLibPath}");
            }
        }
        
        // On macOS, we need to add these frameworks
        if (Target.Platform == UnrealTargetPlatform.Mac)
        {
            PublicFrameworks.AddRange(new string[] { "Security", "CoreFoundation" });
        }
        
        // On Linux, we might need additional libraries
        if (Target.Platform == UnrealTargetPlatform.Linux)
        {
            PublicSystemLibraries.AddRange(new string[] { "ssl", "crypto" });
        }
    }
} 