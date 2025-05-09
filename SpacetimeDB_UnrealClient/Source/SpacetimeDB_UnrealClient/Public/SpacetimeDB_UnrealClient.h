// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

// Create a dedicated log category for SpacetimeDB
SPACETIMEDB_UNREALCLIENT_API DECLARE_LOG_CATEGORY_EXTERN(LogSpacetimeDB, Log, All);

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
	 * Get the module instance
	 * @return Module instance
	 */
	static inline FSpacetimeDB_UnrealClientModule& Get()
	{
		return FModuleManager::LoadModuleChecked<FSpacetimeDB_UnrealClientModule>("SpacetimeDB_UnrealClient");
	}

	/**
	 * Check if the module is available
	 * @return True if the module is loaded
	 */
	static inline bool IsAvailable()
	{
		return FModuleManager::Get().IsModuleLoaded("SpacetimeDB_UnrealClient");
	}

	/**
	 * Gets the name of the SpacetimeDB net driver
	 * 
	 * @return The name of the SpacetimeDB net driver
	 */
	FName GetNetDriverName() const;
}; 