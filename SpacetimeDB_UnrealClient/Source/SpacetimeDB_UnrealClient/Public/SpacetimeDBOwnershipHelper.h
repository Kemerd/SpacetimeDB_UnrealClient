// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "UObject/NoExportTypes.h"
#include "SpacetimeDBOwnershipHelper.generated.h"

/**
 * Helper class for SpacetimeDB ownership and authority management.
 * Provides functions to check and handle object ownership and authority.
 */
UCLASS()
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBOwnershipHelper : public UObject
{
    GENERATED_BODY()

public:
    /**
     * Checks if the local client has ownership of the specified object.
     * @param ObjectId - The SpacetimeDB object ID
     * @return True if the local client owns the object
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static bool HasOwnership(int64 ObjectId);
    
    /**
     * Checks if the local client has authority (can modify) over the specified object.
     * Objects can be modified if:
     * 1. The client owns the object (owner_id matches client ID)
     * 2. The object has no owner (server-owned)
     * 3. The client has explicit authority granted by the server
     * 
     * @param ObjectId - The SpacetimeDB object ID
     * @return True if the local client has authority over the object
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static bool HasAuthority(int64 ObjectId);
    
    /**
     * Gets the owner client ID of an object.
     * @param ObjectId - The SpacetimeDB object ID
     * @return The owner client ID, or 0 if server-owned or not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static int64 GetOwnerClientId(int64 ObjectId);
    
    /**
     * Sets the owner of an object (server-side function)
     * @param ObjectId - The SpacetimeDB object ID
     * @param NewOwnerClientId - The new owner client ID (0 for server)
     * @return True if the request was sent successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static bool RequestSetOwner(int64 ObjectId, int64 NewOwnerClientId);
    
    /**
     * Checks if an RPC can be called on an object by the local client.
     * @param ObjectId - The target object ID
     * @param FunctionName - The RPC function name
     * @return True if the RPC can be called
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static bool CanCallRPC(int64 ObjectId, const FString& FunctionName);

    /**
     * Checks if a property can be modified on an object by the local client.
     * @param ObjectId - The target object ID
     * @param PropertyName - The property name
     * @return True if the property can be modified
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Ownership")
    static bool CanModifyProperty(int64 ObjectId, const FString& PropertyName);
}; 