// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDB_UnrealClient.h"
#include "Modules/ModuleManager.h"

#define LOCTEXT_NAMESPACE "FSpacetimeDB_UnrealClientModule"

void FSpacetimeDB_UnrealClientModule::StartupModule()
{
    // This code will execute after your module is loaded into memory; the exact timing is specified in the .uplugin file per-module
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDB Unreal Client module starting up"));
}

void FSpacetimeDB_UnrealClientModule::ShutdownModule()
{
    // This function may be called during shutdown to clean up your module.
    // For modules that support dynamic reloading, we call this function before unloading the module.
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDB Unreal Client module shutting down"));
}

#undef LOCTEXT_NAMESPACE
	
IMPLEMENT_MODULE(FSpacetimeDB_UnrealClientModule, SpacetimeDB_UnrealClient) 