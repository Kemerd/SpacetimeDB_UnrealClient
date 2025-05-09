// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDB_Types.h"
#include "SpacetimeDB_PropertyValue.h"
#include "SpacetimeDBSubsystem.generated.h"

/**
 * @struct FSpacetimeDBSpawnParams
 * @brief Parameters for spawning an actor or object in SpacetimeDB
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBSpawnParams
{
    GENERATED_BODY()

    /** The class name of the object to spawn (must match a class registered with SpacetimeDB) */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
    FString ClassName;

    /** Optional initial transform for actors */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
    FTransform Transform = FTransform::Identity;

    /** Initial properties as a JSON string */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
    FString PropertiesJson;

    /** Whether this object should be replicated to other clients */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
    bool bReplicate = true;
};

/**
 * @class USpacetimeDBSubsystem
 * @brief Game Instance Subsystem for managing SpacetimeDB connections.
 * 
 * This subsystem manages the lifecycle of a SpacetimeDB client at the game instance level.
 * It provides easy access to SpacetimeDB functionality and handles connection management.
 */
UCLASS()
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBSubsystem : public UGameInstanceSubsystem
{
    GENERATED_BODY()

public:
    // Begin USubsystem
    virtual void Initialize(FSubsystemCollectionBase& Collection) override;
    virtual void Deinitialize() override;
    // End USubsystem

    /** 
     * Connects to a SpacetimeDB instance.
     * 
     * @param Host The host address (e.g., "localhost:3000" or "api.spacetimedb.com")
     * @param DatabaseName The name of the database to connect to
     * @param AuthToken Optional authentication token
     * @return True if connection initiated successfully, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    bool Connect(const FString& Host, const FString& DatabaseName, const FString& AuthToken = TEXT(""));
    
    /**
     * Disconnects from the SpacetimeDB instance.
     * 
     * @return True if disconnection was successful, false if not connected
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    bool Disconnect();
    
    /**
     * Checks if the client is currently connected.
     * 
     * @return True if connected, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    bool IsConnected() const;
    
    /**
     * Gets the SpacetimeDB client ID.
     * 
     * @return The client ID as a 64-bit integer, or 0 if not connected
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    int64 GetSpacetimeDBClientID() const;
    
    /**
     * Calls a reducer function on the SpacetimeDB instance with JSON arguments.
     * 
     * @param ReducerName The name of the reducer to call
     * @param ArgsJson A JSON string with the arguments for the reducer
     * @return True if the call was initiated successfully, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    bool CallReducer(const FString& ReducerName, const FString& ArgsJson);
    
    /**
     * Subscribes to one or more tables in the SpacetimeDB instance.
     * 
     * @param TableNames Array of table names to subscribe to
     * @return True if the subscription was initiated successfully, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    bool SubscribeToTables(const TArray<FString>& TableNames);
    
    /**
     * Gets the client's identity as a hex string.
     * 
     * @return The identity hex string, or empty string if not available
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB")
    FString GetClientIdentity() const;
    
    /**
     * Get the underlying client instance for direct access.
     * 
     * @return Reference to the SpacetimeDB client
     */
    FORCEINLINE FSpacetimeDBClient& GetClient() { return Client; }
    
    /**
     * Get the underlying client instance for direct access (const version).
     * 
     * @return Const reference to the SpacetimeDB client
     */
    FORCEINLINE const FSpacetimeDBClient& GetClient() const { return Client; }
    
    // Blueprint assignable event callbacks
    
    /** Event that fires when the connection is established */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Events")
    FOnConnectedDynamic OnConnected;
    
    /** Event that fires when the connection is closed */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Events")
    FOnDisconnectedDynamic OnDisconnected;
    
    /** Event that fires when the client identity is received */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Events")
    FOnIdentityReceivedDynamic OnIdentityReceived;
    
    /** Event that fires when a table event is received */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Events")
    FOnEventReceivedDynamic OnEventReceived;
    
    /** Event that fires when an error occurs */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Events")
    FOnErrorOccurredDynamic OnErrorOccurred;
    
    /** Event that fires when a property is updated on an object */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Objects")
    FOnObjectCreatedDynamic OnObjectCreated;
    
    /** Event that fires when an object is destroyed */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Objects")
    FOnObjectDestroyedDynamic OnObjectDestroyed;
    
    /** Event that fires when an object ID is remapped from temporary to server ID */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Objects")
    FOnObjectIdRemappedDynamic OnObjectIdRemapped;

    /** Delegate that fires when a property is updated */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB")
    FOnSpacetimeDBPropertyUpdated OnPropertyUpdated;

    //============================
    // Object Lifecycle Management
    //============================
    
    /**
     * Requests the server to spawn an object or actor.
     * The object won't be spawned until the server confirms via the OnObjectCreated callback.
     * 
     * @param Params The spawn parameters
     * @return A temporary object ID that will be remapped when the server confirms creation
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    int64 RequestSpawnObject(const FSpacetimeDBSpawnParams& Params);
    
    /**
     * Requests the server to destroy an object or actor.
     * The object won't be destroyed until the server confirms via the OnObjectDestroyed callback.
     * 
     * @param ObjectId The ID of the object to destroy
     * @return True if the request was sent successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    bool RequestDestroyObject(int64 ObjectId);
    
    /**
     * Finds an object by its SpacetimeDB object ID.
     * 
     * @param ObjectId The SpacetimeDB object ID
     * @return The UObject pointer, or nullptr if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    UObject* FindObjectById(int64 ObjectId) const;
    
    /**
     * Finds the SpacetimeDB object ID for a UObject.
     * 
     * @param Object The UObject to find the ID for
     * @return The SpacetimeDB object ID, or 0 if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    int64 FindObjectId(UObject* Object) const;
    
    /**
     * Gets all SpacetimeDB objects tracked by this subsystem.
     * 
     * @return An array of all objects
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    TArray<UObject*> GetAllObjects() const;

    //////////////////////////
    // Property Replication //
    //////////////////////////
    
    /**
     * Set a property value directly from JSON and optionally replicate it to the server
     * @param ObjectId The ID of the object to update
     * @param PropertyName The name of the property to update
     * @param ValueJson The JSON representation of the new property value
     * @param bReplicateToServer Whether to send the update to the server
     * @return True if the property was successfully updated
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    bool SetPropertyValueFromJson(int64 ObjectId, const FString& PropertyName, const FString& ValueJson, bool bReplicateToServer = true);
    
    /**
     * Set a property value using an object as the source and optionally replicate it to the server
     * @param ObjectId The ID of the object to update
     * @param PropertyName The name of the property to update
     * @param Object The object containing the property to use as the source value
     * @param bReplicateToServer Whether to send the update to the server
     * @return True if the property was successfully updated
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    bool SetPropertyValue(int64 ObjectId, const FString& PropertyName, UObject* Object, bool bReplicateToServer = true);

private:
    // The client instance
    FSpacetimeDBClient Client;
    
    // Delegate handles for event forwarding
    FDelegateHandle OnConnectedHandle;
    FDelegateHandle OnDisconnectedHandle;
    FDelegateHandle OnIdentityReceivedHandle;
    FDelegateHandle OnEventReceivedHandle;
    FDelegateHandle OnErrorOccurredHandle;
    FDelegateHandle OnObjectCreatedHandle;
    FDelegateHandle OnObjectDestroyedHandle;
    FDelegateHandle OnObjectIdRemappedHandle;
    
    // Dynamic multicast delegate declarations for Blueprint exposed events
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnConnectedDynamic);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnDisconnectedDynamic, const FString&, Reason);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnIdentityReceivedDynamic, const FString&, Identity);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnEventReceivedDynamic, const FString&, TableName, const FString&, EventData);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnErrorOccurredDynamic, const FString&, ErrorMessage);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(FOnObjectCreatedDynamic, int64, ObjectId, const FString&, ClassName, const FString&, InitialDataJson);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnObjectDestroyedDynamic, int64, ObjectId);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnObjectIdRemappedDynamic, int64, TempId, int64, ServerId);
    
    // Event handlers to forward client events to Blueprint events
    void HandleConnected();
    void HandleDisconnected(const FString& Reason);
    void HandleIdentityReceived(const FString& Identity);
    void HandleEventReceived(const FString& TableName, const FString& EventData);
    void HandleErrorOccurred(const FString& ErrorMessage);
    void HandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson);
    void HandleObjectDestroyed(uint64 ObjectId);
    void HandleObjectIdRemapped(uint64 TempId, uint64 ServerId);
    
    // Event handler for actor destruction
    UFUNCTION()
    void OnActorDestroyed(AActor* DestroyedActor);

    /**
     * Information about a property update
     */
    USTRUCT(BlueprintType)
    struct FSpacetimeDBPropertyUpdateInfo
    {
        GENERATED_BODY()

        /** The ID of the object that was updated */
        UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB")
        int64 ObjectId;

        /** The object that was updated (may be null if object not found) */
        UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB")
        UObject* Object = nullptr;

        /** The name of the property that was updated */
        UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB")
        FString PropertyName;

        /** The raw JSON value of the property */
        UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB")
        FString RawJsonValue;

        /** The parsed property value */
        UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB")
        FSpacetimeDBPropertyValue PropertyValue;
    };

    /**
     * Delegate for property updates
     */
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnSpacetimeDBPropertyUpdated, const FSpacetimeDBPropertyUpdateInfo&, UpdateInfo);

    // Object Registry - Maps SpacetimeDB object IDs to Unreal UObjects
    TMap<int64, UObject*> ObjectRegistry;
    
    // Reverse lookup - Maps UObjects to their SpacetimeDB IDs
    TMap<UObject*, int64> ObjectToIdMap;
    
    // Internal method to spawn an object based on a server notification
    UObject* SpawnObjectFromServer(int64 ObjectId, const FString& ClassName, const FString& DataJson);
    
    // Internal method to destroy an object based on a server notification
    void DestroyObjectFromServer(int64 ObjectId);
    
    // Property update handling
    void HandlePropertyUpdated(uint64 ObjectId, const FString& PropertyName, const FString& ValueJson);

    /**
     * Send a property update to the server
     * @param ObjectId The ID of the object to update
     * @param PropertyName The name of the property to update
     * @param ValueJson The JSON representation of the new property value
     * @return True if the property update was successfully sent
     */
    bool SendPropertyUpdateToServer(int64 ObjectId, const FString& PropertyName, const FString& ValueJson);
}; 