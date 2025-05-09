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
                Path.Combine(ModuleDirectory, "../../rust")
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
                "Sockets"
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
        
        // Build Rust code on Windows and non-Windows platforms
        string rustBuildScript = IsWindows(Target) ? "build-rust-win.bat" : "build-rust.sh";
        string rustBuildScriptPath = Path.Combine(ModuleDirectory, "../../rust", rustBuildScript);
        
        // Add Rust library path to link against
        string rustLibPath = Path.Combine(ModuleDirectory, "../../rust/target/release");
        PublicAdditionalLibraries.Add(Path.Combine(rustLibPath, "spacetimedb_client.lib"));
        
        // Add pre-build step to build Rust library
        PreBuildSteps.Add(rustBuildScriptPath);
    }
    
    private bool IsWindows(ReadOnlyTargetRules Target)
    {
        return Target.Platform == UnrealTargetPlatform.Win64;
    }
} 