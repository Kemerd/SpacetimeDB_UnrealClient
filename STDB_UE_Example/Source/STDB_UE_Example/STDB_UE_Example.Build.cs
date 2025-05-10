// Copyright Epic Games, Inc. All Rights Reserved.

using UnrealBuildTool;

public class STDB_UE_Example : ModuleRules
{
	public STDB_UE_Example(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
	
		PublicDependencyModuleNames.AddRange(new string[] { "Core", "CoreUObject", "Engine", "InputCore", "EnhancedInput" });

		// Add the preprocessor definition here
		PublicDefinitions.Add("_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH=1");

		// Add verbose linker flag for MSVC
		if (Target.Platform == UnrealTargetPlatform.Win64) // Ensure it's for native MSVC compilation
		{
			// This is a common way to pass flags to the MSVC linker
			PublicAdditionalLibraries.Add("/VERBOSE:LIB"); // For verbose library searching
			// Or for general verbosity: PublicAdditionalLibraries.Add("/VERBOSE"); 
		}

		PrivateDependencyModuleNames.AddRange(new string[] {  });

		// Uncomment if you are using Slate UI
		// PrivateDependencyModuleNames.AddRange(new string[] { "Slate", "SlateCore" });
		
		// Uncomment if you are using online features
		// PrivateDependencyModuleNames.Add("OnlineSubsystem");

		// To include OnlineSubsystemSteam, add it to the plugins section in your uproject file with the Enabled attribute set to true
	}
}
