// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBSettings.h"

USpacetimeDBSettings::USpacetimeDBSettings()
{
    // Default connection settings
    DefaultHostname = TEXT("localhost:3000");
    DefaultDatabaseName = TEXT("spacetimedb-example");
    bAutoConnect = false;
    ReconnectionDelay = 2.0f;
    MaxReconnectionAttempts = 3;
    
    // Default debugging settings
    bEnableDebugLogging = false;
    
    // Default networking settings
    bEnablePrediction = true;
    
    // Default table subscriptions
    bAutoSubscribeDefaultTables = true;
    DefaultTableSubscriptions.Add(TEXT("object_class"));
    DefaultTableSubscriptions.Add(TEXT("property_definition"));
    DefaultTableSubscriptions.Add(TEXT("object_instance"));
}

const USpacetimeDBSettings* USpacetimeDBSettings::Get()
{
    return GetDefault<USpacetimeDBSettings>();
}

#if WITH_EDITOR
FText USpacetimeDBSettings::GetSectionText() const
{
    return FText::FromString(TEXT("SpacetimeDB"));
}

FText USpacetimeDBSettings::GetSectionDescription() const
{
    return FText::FromString(TEXT("Configure settings for the SpacetimeDB integration."));
}
#endif 