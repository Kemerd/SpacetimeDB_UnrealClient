// Copyright (c) 2023 SpacetimeDB authors. All rights reserved.
// SPDX-License-Identifier: MIT

#pragma once

#include "CoreMinimal.h"
#include "SpacetimeDBTypeConversions.generated.h"

// Forward declarations for the FFI types
namespace stdb {
namespace shared {
    struct Vector3;
    struct Rotator;
    struct Quat;
    struct Transform;
    struct Color;
}
}

/**
 * Utility functions for converting between SpacetimeDB and Unreal Engine types.
 * Since the SpacetimeDB SharedModule types are designed to match Unreal's types,
 * we can use direct conversion rather than creating intermediate types.
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBTypeConversions
{
    GENERATED_BODY()

public:
    /**
     * Converts an Unreal FVector to a SpacetimeDB Vector3
     */
    static stdb::shared::Vector3 ToStdbVector3(const FVector& Vector);

    /**
     * Converts a SpacetimeDB Vector3 to an Unreal FVector
     */
    static FVector FromStdbVector3(const stdb::shared::Vector3& Vector);

    /**
     * Converts an Unreal FRotator to a SpacetimeDB Rotator
     */
    static stdb::shared::Rotator ToStdbRotator(const FRotator& Rotator);

    /**
     * Converts a SpacetimeDB Rotator to an Unreal FRotator
     */
    static FRotator FromStdbRotator(const stdb::shared::Rotator& Rotator);

    /**
     * Converts an Unreal FQuat to a SpacetimeDB Quat
     */
    static stdb::shared::Quat ToStdbQuat(const FQuat& Quat);

    /**
     * Converts a SpacetimeDB Quat to an Unreal FQuat
     */
    static FQuat FromStdbQuat(const stdb::shared::Quat& Quat);

    /**
     * Converts an Unreal FTransform to a SpacetimeDB Transform
     */
    static stdb::shared::Transform ToStdbTransform(const FTransform& Transform);

    /**
     * Converts a SpacetimeDB Transform to an Unreal FTransform
     */
    static FTransform FromStdbTransform(const stdb::shared::Transform& Transform);

    /**
     * Converts an Unreal FColor to a SpacetimeDB Color
     */
    static stdb::shared::Color ToStdbColor(const FColor& Color);

    /**
     * Converts a SpacetimeDB Color to an Unreal FColor
     */
    static FColor FromStdbColor(const stdb::shared::Color& Color);

    /**
     * Converts an Unreal FLinearColor to a SpacetimeDB Color
     */
    static stdb::shared::Color ToStdbColor(const FLinearColor& Color);

    /**
     * Converts a SpacetimeDB Color to an Unreal FLinearColor
     */
    static FLinearColor ToLinearColor(const stdb::shared::Color& Color);
}; 