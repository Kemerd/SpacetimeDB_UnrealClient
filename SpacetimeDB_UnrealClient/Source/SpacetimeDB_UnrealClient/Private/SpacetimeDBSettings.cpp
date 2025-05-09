// Copyright Epic Games, Inc. All Rights Reserved.

#include "SpacetimeDBSettings.h"

USpacetimeDBSettings::USpacetimeDBSettings()
{
    // Set default values from README.md
    SpacetimeHost = TEXT("localhost:3000");
    SpacetimeDBName = TEXT("");
    SpacetimeAuthToken = TEXT("");
    bAutoConnect = false;
    
    // Default performance settings
    MaxObjects = 100000;
    ReplicationInterval = 0.1f;
    
    // Default relevancy settings
    DefaultRelevancy = TEXT("AlwaysRelevant");
    MaxRelevancyDistance = 10000.0f;
    ZoneLimit = 1000;
    
    // Default debug settings
    bVerboseLogging = false;
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