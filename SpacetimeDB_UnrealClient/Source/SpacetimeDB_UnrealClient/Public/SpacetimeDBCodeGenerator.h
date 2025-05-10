// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"

#include "SpacetimeDBCodeGenerator.generated.h"

/**
 * SpacetimeDBCodeGenerator
 * 
 * Editor utility that generates Rust code for SpacetimeDB integration.
 * This generator creates class definitions, component mappings, and property registrations
 * based on the current project's class hierarchy.
 * 
 * Usage:
 * - Call GenerateRustClassRegistry from the editor to create a new class_registry.rs file
 * - This file will be imported by the SpacetimeDB server module to register UE classes
 */
UCLASS(Config=Editor)
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBCodeGenerator : public UEditorSubsystem
{
    GENERATED_BODY()

public:
    USpacetimeDBCodeGenerator();

    /** Initialize the subsystem */
    virtual void Initialize(FSubsystemCollectionBase& Collection) override;

    /** Deinitialize the subsystem */
    virtual void Deinitialize() override;

    /**
     * Generate a Rust file containing registrations for all relevant classes
     * @param OutputPath Path where the generated file should be saved
     * @return True if generation was successful
     */
    UFUNCTION(BlueprintCallable, Category="SpacetimeDB")
    bool GenerateRustClassRegistry(const FString& OutputPath);

    /**
     * Generate a Rust file containing default component mappings for actor classes
     * @param OutputPath Path where the generated file should be saved
     * @return True if generation was successful
     */
    UFUNCTION(BlueprintCallable, Category="SpacetimeDB")
    bool GenerateRustComponentMappings(const FString& OutputPath);

private:
    /** Collects all UClass objects that should be registered with SpacetimeDB */
    void GetAllRelevantClasses(TArray<UClass*>& OutClasses);

    /** Generates Rust code for a single UClass object */
    FString GenerateClassRegistration(UClass* Class, int32 ClassId);

    /** Generates Rust code for registering class properties */
    FString GeneratePropertyRegistrations(UClass* Class, int32 ClassId);

    /** Determines component requirements for a given actor class */
    void GetDefaultComponentsForClass(UClass* Class, TMap<FString, FString>& OutComponents);

    /** Generates a unique class ID for a UClass */
    int32 GenerateClassId(UClass* Class);

    /** Map of class path names to assigned IDs */
    TMap<FString, int32> ClassIdMap;

    /** Next available class ID */
    int32 NextClassId;
}; 