// Copyright (c) 2023 SpacetimeDB authors. All rights reserved.
// SPDX-License-Identifier: MIT

#include "SpacetimeDBTypeConversions.h"
#include "ffi.h" // Include the FFI header generated from Rust

// Vector3 conversions
stdb::shared::Vector3 FSpacetimeDBTypeConversions::ToStdbVector3(const FVector& Vector)
{
    stdb::shared::Vector3 Result;
    Result.x = Vector.X;
    Result.y = Vector.Y;
    Result.z = Vector.Z;
    return Result;
}

FVector FSpacetimeDBTypeConversions::FromStdbVector3(const stdb::shared::Vector3& Vector)
{
    return FVector(Vector.x, Vector.y, Vector.z);
}

// Rotator conversions
stdb::shared::Rotator FSpacetimeDBTypeConversions::ToStdbRotator(const FRotator& Rotator)
{
    stdb::shared::Rotator Result;
    Result.pitch = Rotator.Pitch;
    Result.yaw = Rotator.Yaw;
    Result.roll = Rotator.Roll;
    return Result;
}

FRotator FSpacetimeDBTypeConversions::FromStdbRotator(const stdb::shared::Rotator& Rotator)
{
    return FRotator(Rotator.pitch, Rotator.yaw, Rotator.roll);
}

// Quat conversions
stdb::shared::Quat FSpacetimeDBTypeConversions::ToStdbQuat(const FQuat& Quat)
{
    stdb::shared::Quat Result;
    Result.x = Quat.X;
    Result.y = Quat.Y;
    Result.z = Quat.Z;
    Result.w = Quat.W;
    return Result;
}

FQuat FSpacetimeDBTypeConversions::FromStdbQuat(const stdb::shared::Quat& Quat)
{
    return FQuat(Quat.x, Quat.y, Quat.z, Quat.w);
}

// Transform conversions
stdb::shared::Transform FSpacetimeDBTypeConversions::ToStdbTransform(const FTransform& Transform)
{
    stdb::shared::Transform Result;
    Result.location = ToStdbVector3(Transform.GetLocation());
    Result.rotation = ToStdbQuat(Transform.GetRotation());
    Result.scale = ToStdbVector3(Transform.GetScale3D());
    return Result;
}

FTransform FSpacetimeDBTypeConversions::FromStdbTransform(const stdb::shared::Transform& Transform)
{
    return FTransform(
        FromStdbQuat(Transform.rotation),
        FromStdbVector3(Transform.location),
        FromStdbVector3(Transform.scale)
    );
}

// Color conversions
stdb::shared::Color FSpacetimeDBTypeConversions::ToStdbColor(const FColor& Color)
{
    stdb::shared::Color Result;
    Result.r = Color.R;
    Result.g = Color.G;
    Result.b = Color.B;
    Result.a = Color.A;
    return Result;
}

FColor FSpacetimeDBTypeConversions::FromStdbColor(const stdb::shared::Color& Color)
{
    return FColor(Color.r, Color.g, Color.b, Color.a);
}

stdb::shared::Color FSpacetimeDBTypeConversions::ToStdbColor(const FLinearColor& Color)
{
    // Convert to FColor first (which handles clamping values)
    FColor IntermediateColor = Color.ToFColor(true);
    return ToStdbColor(IntermediateColor);
}

FLinearColor FSpacetimeDBTypeConversions::ToLinearColor(const stdb::shared::Color& Color)
{
    return FLinearColor(
        static_cast<float>(Color.r) / 255.0f,
        static_cast<float>(Color.g) / 255.0f,
        static_cast<float>(Color.b) / 255.0f,
        static_cast<float>(Color.a) / 255.0f
    );
} 