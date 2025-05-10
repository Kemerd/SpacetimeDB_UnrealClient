// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDB_UnrealClient.h"
#include "Modules/ModuleManager.h"
#include "SpacetimeDBNetDriver.h"
#include "SpacetimeDBNetConnection.h"
#include "SpacetimeDBSubsystem.h"
#include "Net/UnrealNetwork.h"
#include "Misc/CoreDelegates.h"
#include "Misc/Paths.h"
#include "HAL/PlatformFileManager.h"
#include "HAL/PlatformProcess.h"

// Define the log category
DEFINE_LOG_CATEGORY(LogSpacetimeDB);

#define LOCTEXT_NAMESPACE "FSpacetimeDB_UnrealClientModule"

void FSpacetimeDB_UnrealClientModule::StartupModule()
{
    // This code will execute after your module is loaded into memory; the exact timing is specified in the .uplugin file per-module
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDB Unreal Client module starting up"));
    
    // Register the SpacetimeDB NetDriver
    if (!GEngine->NetDriverDefinitions.ContainsByPredicate([](const FNetDriverDefinition& Def) {
        return Def.DefName == FName(TEXT("SpacetimeDB"));
    }))
    {
        FNetDriverDefinition SpacetimeDBDriverDef;
        SpacetimeDBDriverDef.DefName = FName(TEXT("SpacetimeDB"));
        SpacetimeDBDriverDef.DriverClassName = TEXT("/Script/SpacetimeDB_UnrealClient.SpacetimeDBNetDriver");
        SpacetimeDBDriverDef.DriverClassNameFallback = TEXT("");
        
        GEngine->NetDriverDefinitions.Add(SpacetimeDBDriverDef);
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDB NetDriver registered"));
    }
    
    // Register NetDriver with the engine
    bool bRegisteredNetDriver = false;
    // In UE 5.5, we need to use a different approach for registering net drivers
    UWorld* World = nullptr;  // We don't have a world yet
    bRegisteredNetDriver = GEngine->CreateNamedNetDriver(World, GetNetDriverName(), FName(TEXT("SpacetimeDBNetDriver")));
    
    if (bRegisteredNetDriver)
    {
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDB NetDriver registered with engine successfully"));
    }
    
    // Register connection factory - modified for UE 5.5
    // NetworkDriverDelegates was removed in UE 5.5, we need to use a different approach
    FNetDriverDefinition NewDriverDef;
    NewDriverDef.DefName = GetNetDriverName();
    NewDriverDef.DriverClassName = TEXT("/Script/SpacetimeDB_UnrealClient.SpacetimeDBNetDriver");
    NewDriverDef.DriverClassNameFallback = TEXT("");
    GEngine->NetDriverDefinitions.Add(NewDriverDef);
    
    // Register client-side subsystem
    // IGameInstanceSubsystemRegistrar has been removed in UE 5.5
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem will be registered automatically"));
}

void FSpacetimeDB_UnrealClientModule::ShutdownModule()
{
    // This function may be called during shutdown to clean up your module.
    // For modules that support dynamic reloading, we call this function before unloading the module.
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDB Unreal Client module shutting down"));
    
    // Unregister NetDriver
    if (GEngine)
    {
        // NetworkDriverDelegates was removed in UE 5.5
        // We'll just destroy the named net driver
        UWorld* World = nullptr;  // We don't have a world at this point
        GEngine->DestroyNamedNetDriver(World, GetNetDriverName());

        // Remove from NetDriverDefinitions
        for (int32 i = GEngine->NetDriverDefinitions.Num() - 1; i >= 0; i--)
        {
            if (GEngine->NetDriverDefinitions[i].DefName == FName(TEXT("SpacetimeDB")))
            {
                GEngine->NetDriverDefinitions.RemoveAt(i);
                UE_LOG(LogTemp, Log, TEXT("SpacetimeDB NetDriver unregistered"));
                break;
            }
        }
    }
    
    // Unregister client-side subsystem
    // In UE 5.5, subsystems are registered automatically by the engine
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem will be unregistered automatically"));
}

FName FSpacetimeDB_UnrealClientModule::GetNetDriverName() const
{
    return FName(TEXT("SpacetimeDBNetDriver"));
}

#undef LOCTEXT_NAMESPACE
    
IMPLEMENT_MODULE(FSpacetimeDB_UnrealClientModule, SpacetimeDB_UnrealClient) 