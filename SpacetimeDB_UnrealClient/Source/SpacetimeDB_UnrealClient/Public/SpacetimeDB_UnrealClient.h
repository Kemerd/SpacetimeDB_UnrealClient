// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

/**
 * @class FSpacetimeDB_UnrealClientModule
 * @brief The module implementation for the SpacetimeDB Unreal Client plugin.
 * 
 * This module handles registration of the SpacetimeDB NetDriver and Subsystem.
 */
class SPACETIMEDB_UNREALCLIENT_API FSpacetimeDB_UnrealClientModule : public IModuleInterface
{
public:
    /** IModuleInterface implementation */
    virtual void StartupModule() override;
    virtual void ShutdownModule() override;
    
    /**
     * Gets the name of the SpacetimeDB net driver
     * 
     * @return The name of the SpacetimeDB net driver
     */
    FName GetNetDriverName() const;
}; 