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
	}
}
