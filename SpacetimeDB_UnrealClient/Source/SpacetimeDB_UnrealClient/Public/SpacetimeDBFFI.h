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
inline rust::String ToCxxString(const FString& InString)
{
    return rust::String(TCHAR_TO_UTF8(*InString));
}

/**
 * Helper function to convert CxxString to FString for FFI results
 */
inline FString FromCxxString(const rust::String& InString)
{
    return FString(UTF8_TO_TCHAR(InString.data()));
}

// Forward declarations for FFI types
typedef uint64_t ObjectId;
typedef uint32_t SequenceNumber;

extern "C" {
    // Existing functions...

    // Prediction-related functions
    bool register_prediction_object(ObjectId object_id);
    bool unregister_prediction_object(ObjectId object_id);
    SequenceNumber get_next_prediction_sequence(ObjectId object_id);
    bool send_predicted_transform(
        ObjectId object_id,
        SequenceNumber sequence,
        float location_x,
        float location_y,
        float location_z,
        float rotation_x,
        float rotation_y,
        float rotation_z,
        float rotation_w,
        float scale_x,
        float scale_y,
        float scale_z,
        float velocity_x,
        float velocity_y,
        float velocity_z,
        bool has_velocity
    );
    SequenceNumber get_last_acked_sequence(ObjectId object_id);
} 