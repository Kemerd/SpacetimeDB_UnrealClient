// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDB_UnrealClient.h"
#include "Modules/ModuleManager.h"
#include "SpacetimeDBNetDriver.h"
#include "SpacetimeDBNetConnection.h"
#include "SpacetimeDBSubsystem.h"
#include "Net/UnrealNetwork.h"
#include "Misc/CoreDelegates.h"

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
    bool bRegisteredNetDriver = GEngine->RegisterNetDriver_Override.IsBound() ?
        GEngine->RegisterNetDriver_Override.Execute(GetNetDriverName(), TEXT("SpacetimeDBNetDriver")) :
        GEngine->CreateNamedNetDriver(nullptr, GetNetDriverName(), FName(TEXT("SpacetimeDBNetDriver")));
    
    if (bRegisteredNetDriver)
    {
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDB NetDriver registered with engine successfully"));
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("Failed to register SpacetimeDB NetDriver with engine"));
    }
    
    // Register connection factory
    GEngine->NetworkDriverDelegates.Add(FName(TEXT("SpacetimeDBNetDriver")), FNetworkDriverFactory::CreateNetworkDriverDelegate::CreateLambda([](const FName& InNetDriverName) {
        return NewObject<USpacetimeDBNetDriver>();
    }));
    
    // Register client-side subsystem
    if (!IGameInstanceSubsystemRegistrar::Get().UnregisterClass(USpacetimeDBSubsystem::StaticClass()))
    {
        IGameInstanceSubsystemRegistrar::Get().RegisterClass(USpacetimeDBSubsystem::StaticClass());
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem registered"));
    }
}

void FSpacetimeDB_UnrealClientModule::ShutdownModule()
{
    // This function may be called during shutdown to clean up your module.
    // For modules that support dynamic reloading, we call this function before unloading the module.
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDB Unreal Client module shutting down"));
    
    // Unregister NetDriver
    if (GEngine)
    {
        GEngine->NetworkDriverDelegates.Remove(FName(TEXT("SpacetimeDBNetDriver")));
        GEngine->DestroyNamedNetDriver(nullptr, GetNetDriverName());
        
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
    IGameInstanceSubsystemRegistrar::Get().UnregisterClass(USpacetimeDBSubsystem::StaticClass());
}

FName FSpacetimeDB_UnrealClientModule::GetNetDriverName() const
{
    return FName(TEXT("SpacetimeDBNetDriver"));
}

#undef LOCTEXT_NAMESPACE
    
IMPLEMENT_MODULE(FSpacetimeDB_UnrealClientModule, SpacetimeDB_UnrealClient) 