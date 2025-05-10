// STLCompat.cpp - Provides compatibility for missing STL symbols
// This file is compiled by Unreal's build system, ensuring proper linkage with MSVC STL

#include "SpacetimeDB_UnrealClient.h"

// Define the missing STL symbol that's causing linker errors
// This is a fallback implementation that will be used if the linker can't find it in MSVC STL
extern "C" {
    // The mangled name for this is what the Rust code is looking for
    __declspec(dllexport) unsigned long long __cdecl __std_mismatch_1(const void* first, const void* last, unsigned long long count) {
        // Simple implementation that searches for the first mismatch between two memory regions
        const unsigned char* p1 = static_cast<const unsigned char*>(first);
        const unsigned char* p2 = static_cast<const unsigned char*>(last);
        
        unsigned long long i = 0;
        while (i < count && p1[i] == p2[i]) {
            ++i;
        }
        
        return i; // Return position of first mismatch
    }
} 