// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

/**
 * @file UnrealReplication.h
 * @brief Minimal interface for bridging between Rust and Unreal Engine.
 * 
 * This file serves as a minimal interface for the CXX bridge, without
 * requiring any Unreal Engine specific headers. This allows it to be 
 * compiled outside of the Unreal build environment.
 * 
 * The real integration with Unreal types happens in the C++ side
 * of the plugin, not in this header.
 */

namespace SpacetimeDBReplication
{
    // Any shared types would be defined here
    // These types should be simple C++ types without dependencies on Unreal
} 