// Copyright Epic Games, Inc. All Rights Reserved.

using UnrealBuildTool;
using System.Collections.Generic;

public class STDB_UE_ExampleEditorTarget : TargetRules
{
	public STDB_UE_ExampleEditorTarget( TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.V5;
		IncludeOrderVersion = EngineIncludeOrderVersion.Unreal5_5;
		ExtraModuleNames.Add("STDB_UE_Example");
		// We cannot undo this because then we would have to build the engine from source
		// Do not uncomment, this will not help us we CANNOT UNCOMMENT THIS
		//GlobalDefinitions.Add("_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH=1");
		//BuildEnvironment = TargetBuildEnvironment.Unique;
	}
}
