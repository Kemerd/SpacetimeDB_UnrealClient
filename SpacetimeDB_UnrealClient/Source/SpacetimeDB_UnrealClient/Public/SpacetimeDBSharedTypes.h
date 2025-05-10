// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"

#define SPACETIMEDB_SHARED_TYPES_INCLUDED 1

namespace stdb {
    namespace shared {
        // Vector3 type that matches FVector
        struct Vector3 
        {
            float x;
            float y;
            float z;
        };

        // Rotator type that matches FRotator
        struct Rotator 
        {
            float pitch;
            float yaw;
            float roll;
        };

        // Quaternion type that matches FQuat
        struct Quat 
        {
            float x;
            float y;
            float z;
            float w;
        };

        // Transform type that matches FTransform
        struct Transform 
        {
            Vector3 location;
            Quat rotation;
            Vector3 scale;
        };

        // Color type that matches FColor
        struct Color 
        {
            uint8 r;
            uint8 g;
            uint8 b;
            uint8 a;
        };
    }
} 