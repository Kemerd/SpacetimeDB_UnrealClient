// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "rust/cxx.h"
#include <string>
#include <memory>

/**
 * @file bridge.h
 * @brief Helper functions for the cxx FFI bridge between Rust and C++.
 * 
 * This file contains various helper functions that facilitate interactions
 * between Rust and C++ through the cxx bridge, particularly for memory 
 * management of objects that need to be passed across the FFI boundary.
 */

namespace SpacetimeDBReplication
{
    /**
     * Creates a UniquePtr-wrapped CxxString (std::string) from a Rust string slice.
     * 
     * @param s The Rust string slice to convert to a C++ owned string
     * @return A std::unique_ptr to a std::string containing a copy of the input
     */
    inline std::unique_ptr<std::string> make_unique_string(rust::Str s) {
        return std::make_unique<std::string>(s.data(), s.size());
    }
} 