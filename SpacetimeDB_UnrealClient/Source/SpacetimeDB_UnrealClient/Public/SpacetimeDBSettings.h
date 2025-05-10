// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/Object.h"
#include "SpacetimeDBSettings.generated.h"

/**
 * Settings class for the SpacetimeDB Unreal Client plugin.
 * Provides configuration options for the plugin behavior.
 */
UCLASS(config=Game, defaultconfig)
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBSettings : public UObject
{
    GENERATED_BODY()
    
public:
    USpacetimeDBSettings();
    
    /** Default server hostname to connect to */
    UPROPERTY(config, EditAnywhere, Category = "Connection")
    FString DefaultHostname;
    
    /** Default database name to connect to */
    UPROPERTY(config, EditAnywhere, Category = "Connection")
    FString DefaultDatabaseName;
    
    /** Whether to enable debug logging */
    UPROPERTY(config, EditAnywhere, Category = "Debugging")
    bool bEnableDebugLogging;
    
    /** Whether to auto-connect on game start */
    UPROPERTY(config, EditAnywhere, Category = "Connection")
    bool bAutoConnect;
    
    /** Time in seconds to wait before attempting reconnection */
    UPROPERTY(config, EditAnywhere, Category = "Connection", meta = (ClampMin = "0.5", ClampMax = "60.0"))
    float ReconnectionDelay;
    
    /** Maximum number of reconnection attempts before giving up */
    UPROPERTY(config, EditAnywhere, Category = "Connection", meta = (ClampMin = "0", ClampMax = "10"))
    int32 MaxReconnectionAttempts;
    
    /** Whether to enable network prediction */
    UPROPERTY(config, EditAnywhere, Category = "Networking")
    bool bEnablePrediction;
    
    /** Whether to automatically subscribe to default tables on connect */
    UPROPERTY(config, EditAnywhere, Category = "Tables")
    bool bAutoSubscribeDefaultTables;
    
    /** List of table names to auto-subscribe to if bAutoSubscribeDefaultTables is true */
    UPROPERTY(config, EditAnywhere, Category = "Tables")
    TArray<FString> DefaultTableSubscriptions;
    
    /** Get the settings object. */
    static const USpacetimeDBSettings* Get();
    
#if WITH_EDITOR
    /** Get the section text for the settings panel. */
    virtual FText GetSectionText() const;
    
    /** Get the section description for the settings panel. */
    virtual FText GetSectionDescription() const;
#endif
}; 