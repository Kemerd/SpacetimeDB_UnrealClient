// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"

// Forward declare the FSpacetimeDBNetDriverPrivate class
// This is used as a PIMPL (Pointer to IMPLementation) pattern for the USpacetimeDBNetDriver
// to hide implementation details and maintain ABI compatibility.
//
// The FSpacetimeDBNetDriverPrivate class is fully defined in SpacetimeDBNetDriver.cpp.
class FSpacetimeDBNetDriverPrivate; 