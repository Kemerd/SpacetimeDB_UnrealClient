// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBOwnershipHelper.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDBSubsystem.h"
#include "Kismet/GameplayStatics.h"
#include "Engine/World.h"

bool USpacetimeDBOwnershipHelper::HasOwnership(int64 ObjectId)
{
    if (ObjectId <= 0)
    {
        return false;
    }
    
    // Get the subsystem
    USpacetimeDBSubsystem* SpacetimeDB = GEngine->GetEngineSubsystem<USpacetimeDBSubsystem>();
    if (!SpacetimeDB)
    {
        return false;
    }
    
    // Get the client ID
    uint64 ClientId = SpacetimeDB->GetClient().GetClientID();
    
    // Get the owner ID for this object
    int64 OwnerId = GetOwnerClientId(ObjectId);
    
    // Compare the client ID with the owner ID
    return (OwnerId > 0) && (OwnerId == ClientId);
}

bool USpacetimeDBOwnershipHelper::HasAuthority(int64 ObjectId)
{
    UGameInstance* GameInstance = UGameplayStatics::GetGameInstance(GEngine->GetWorld());
    if (!GameInstance)
    {
        return false;
    }

    USpacetimeDBSubsystem* SpacetimeDB = GameInstance->GetSubsystem<USpacetimeDBSubsystem>();
    if (!SpacetimeDB || !SpacetimeDB->IsConnected())
    {
        return false;
    }

    // Get the owner ID of the object
    int64 OwnerClientId = GetOwnerClientId(ObjectId);
    
    // SECURITY: Only allow clients to modify objects they explicitly own
    // Server-owned objects (owner_id == 0) should NOT be directly modifiable by clients
    // This prevents cheating by disallowing arbitrary modification of server-owned objects
    // Instead, clients should use validated RPCs to request changes to server-owned objects
    return OwnerClientId == SpacetimeDB->GetClientId();
}

int64 USpacetimeDBOwnershipHelper::GetOwnerClientId(int64 ObjectId)
{
    UGameInstance* GameInstance = UGameplayStatics::GetGameInstance(GEngine->GetWorld());
    if (!GameInstance)
    {
        return 0;
    }

    USpacetimeDBSubsystem* SpacetimeDB = GameInstance->GetSubsystem<USpacetimeDBSubsystem>();
    if (!SpacetimeDB)
    {
        return 0;
    }

    // Get the object from the SpacetimeDB object map
    UObject* Object = SpacetimeDB->FindObjectById(ObjectId);
    if (!Object)
    {
        return 0;
    }

    // Try to get the owner client ID from the object's properties
    FString OwnerIdJson = SpacetimeDB->GetPropertyJsonValue(ObjectId, TEXT("owner_id"));
    if (OwnerIdJson.IsEmpty() || OwnerIdJson == TEXT("null"))
    {
        return 0; // No owner (server-owned)
    }

    // Parse the JSON string to get the owner ID
    int64 OwnerClientId = FCString::Atoi64(*OwnerIdJson);
    return OwnerClientId;
}

bool USpacetimeDBOwnershipHelper::RequestSetOwner(int64 ObjectId, int64 NewOwnerClientId)
{
    // Get the SpacetimeDB subsystem
    USpacetimeDBSubsystem* SpacetimeDB = GEngine->GetEngineSubsystem<USpacetimeDBSubsystem>();
    if (!SpacetimeDB)
    {
        return false;
    }
    
    // Check if the current client has authority
    if (!HasAuthority(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBOwnershipHelper: Cannot set owner - No authority over object %lld"), ObjectId);
        return false;
    }
    
    // We'll use the set_property RPC to set the owner_id
    FString ValueJson = FString::Printf(TEXT("%lld"), NewOwnerClientId);
    
    // Call the server function to set the owner
    TArray<FStdbRpcArg> Args;
    Args.Add(FStdbRpcArg(TEXT("new_owner_id"), (int32)NewOwnerClientId));
    return SpacetimeDB->CallServerFunction(ObjectId, TEXT("set_owner"), Args);
}

bool USpacetimeDBOwnershipHelper::CanCallRPC(int64 ObjectId, const FString& FunctionName)
{
    // By default, we'll allow RPCs if the client has authority over the object
    // More sophisticated implementations could check specific RPC permissions
    // based on the function name or other criteria
    
    // Simple case: Client can call RPCs on objects they have authority over
    return HasAuthority(ObjectId);
}

bool USpacetimeDBOwnershipHelper::CanModifyProperty(int64 ObjectId, const FString& PropertyName)
{
    // By default, we'll allow property modifications if the client has authority over the object
    // More sophisticated implementations could check specific property permissions
    
    // Simple case: Client can modify properties on objects they have authority over
    return HasAuthority(ObjectId);
} 