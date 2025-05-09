// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"

// This header acts as a wrapper for the CXX-generated FFI headers from Rust
// It deals with platform-specific includes and any required wrapper functions

// Silence some warnings from cxx.h
PRAGMA_DISABLE_SHADOW_VARIABLE_WARNINGS
THIRD_PARTY_INCLUDES_START
#include "ffi.h"  // CXX-generated header from Rust
THIRD_PARTY_INCLUDES_END
PRAGMA_ENABLE_SHADOW_VARIABLE_WARNINGS

// You can add additional helper functions or type conversions here if needed
// to make working with the Rust FFI more convenient

/**
 * Helper function to convert FString to CxxString for FFI calls
 */
inline cxx::String ToCxxString(const FString& InString)
{
    return cxx::String(TCHAR_TO_UTF8(*InString));
}

/**
 * Helper function to convert CxxString to FString for FFI results
 */
inline FString FromCxxString(const cxx::String& InString)
{
    return FString(UTF8_TO_TCHAR(InString.c_str()));
} 