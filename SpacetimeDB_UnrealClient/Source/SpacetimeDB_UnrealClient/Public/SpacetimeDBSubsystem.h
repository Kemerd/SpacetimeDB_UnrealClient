// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDB_Types.h"
#include "SpacetimeDB_PropertyValue.h"
#include "SpacetimeDBFFI.h"
#include "SpacetimeDB_ErrorHandler.h"
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
 * @struct FStdbRpcArg
 * @brief Structure representing a single RPC argument
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FStdbRpcArg
{
    GENERATED_BODY()

    /** The name of the argument */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|RPC")
    FString Name;
    
    /** The type of the argument */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|RPC")
    ESpacetimeDBValueType Type = ESpacetimeDBValueType::Null;
    
    /** The value of the argument as a variant */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|RPC")
    FSpacetimeDBPropertyValue Value;

    /** Default constructor */
    FStdbRpcArg() {}
    
    /** Constructor with name and boolean value */
    FStdbRpcArg(const FString& InName, bool InValue)
        : Name(InName), Type(ESpacetimeDBValueType::Bool)
    {
        Value.SetBool(InValue);
    }
    
    /** Constructor with name and integer value */
    FStdbRpcArg(const FString& InName, int32 InValue)
        : Name(InName), Type(ESpacetimeDBValueType::Int)
    {
        Value.SetInt(InValue);
    }
    
    /** Constructor with name and float value */
    FStdbRpcArg(const FString& InName, float InValue)
        : Name(InName), Type(ESpacetimeDBValueType::Float)
    {
        Value.SetFloat(InValue);
    }
    
    /** Constructor with name and string value */
    FStdbRpcArg(const FString& InName, const FString& InValue)
        : Name(InName), Type(ESpacetimeDBValueType::String)
    {
        Value.SetString(InValue);
    }
};

/**
 * Handles transformation data with prediction sequence information
 */
USTRUCT(BlueprintType)
struct FPredictedTransformData
{
	GENERATED_BODY()

	/** The object ID */
	UPROPERTY(BlueprintReadWrite, Category = "SpacetimeDB|Prediction")
	FObjectID ObjectID;

	/** The sequence number for prediction */
	UPROPERTY(BlueprintReadWrite, Category = "SpacetimeDB|Prediction")
	int32 SequenceNumber = 0;

	/** The transform data */
	UPROPERTY(BlueprintReadWrite, Category = "SpacetimeDB|Prediction")
	FTransform Transform;

	/** The velocity data */
	UPROPERTY(BlueprintReadWrite, Category = "SpacetimeDB|Prediction")
	FVector Velocity = FVector::ZeroVector;

	/** Whether this update includes velocity */
	UPROPERTY(BlueprintReadWrite, Category = "SpacetimeDB|Prediction")
	bool bHasVelocity = false;
};

/** Delegate for client RPC events */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnClientRpcReceived, int64, ObjectId, const TArray<FStdbRpcArg>&, Args);

/** Delegate for server RPC events */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnServerRpcReceived, int64, ObjectId, const TArray<FStdbRpcArg>&, Args);

/** Delegate for component added events */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(FOnComponentAddedDynamic, int64, ActorId, int64, ComponentId, const FString&, ComponentClassName);

/** Delegate for component removed events */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnComponentRemovedDynamic, int64, ActorId, int64, ComponentId);

/** Delegate for handling SpacetimeDB error events with detailed error info */
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnSpacetimeDBErrorOccurred, const FSpacetimeDBErrorInfo&, ErrorInfo);

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
    FOnSpacetimeDBErrorOccurred OnErrorOccurred;
    
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
    
    /** Delegate that fires when a server RPC is received */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|RPC")
    FOnServerRpcReceived OnServerRpcReceived;

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
     * Gets the SpacetimeDB object ID for a UObject.
     * 
     * @param Object The UObject to find the ID for
     * @return The object ID, or 0 if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Objects")
    int64 GetObjectId(UObject* Object) const;
    
    //============================
    // Property Management
    //============================
    
    /**
     * Gets a property value as a JSON string.
     * 
     * @param ObjectId The SpacetimeDB object ID
     * @param PropertyName The name of the property to get
     * @return The property value as a JSON string, or empty string if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    FString GetPropertyJsonValue(int64 ObjectId, const FString& PropertyName) const;
    
    /**
     * Gets a property value.
     * 
     * @param ObjectId The SpacetimeDB object ID
     * @param PropertyName The name of the property to get
     * @return The property value, or null value if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    FSpacetimeDBPropertyValue GetPropertyValue(int64 ObjectId, const FString& PropertyName) const;
    
    /**
     * Sets a property value from a JSON string.
     * 
     * @param ObjectId The SpacetimeDB object ID
     * @param PropertyName The name of the property to set
     * @param ValueJson The new property value as a JSON string
     * @param bReplicateToServer Whether to replicate the change to the server
     * @return True if the property was set successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    bool SetPropertyValueFromJson(int64 ObjectId, const FString& PropertyName, const FString& ValueJson, bool bReplicateToServer = true);
    
    /**
     * Sets a property value from a UObject.
     * 
     * @param ObjectId The SpacetimeDB object ID
     * @param PropertyName The name of the property to set
     * @param Object The UObject containing the property to set
     * @param bReplicateToServer Whether to replicate the change to the server
     * @return True if the property was set successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    bool SetPropertyValue(int64 ObjectId, const FString& PropertyName, UObject* Object, bool bReplicateToServer = true);
    
    //============================
    // RPC Management
    //============================
    
    /**
     * Calls a server function on a specific object.
     * 
     * @param TargetObject The object on which to call the function
     * @param FunctionName The name of the function to call
     * @param Args The arguments to pass to the function
     * @return True if the call was initiated successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|RPC")
    bool CallServerFunctionOnObject(UObject* TargetObject, const FString& FunctionName, const TArray<FStdbRpcArg>& Args);
    
    /**
     * Calls a server function on an object identified by its ID.
     * 
     * @param ObjectId The ID of the object on which to call the function
     * @param FunctionName The name of the function to call
     * @param Args The arguments to pass to the function
     * @return True if the call was initiated successfully
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|RPC")
    bool CallServerFunction(int64 ObjectId, const FString& FunctionName, const TArray<FStdbRpcArg>& Args);
    
    /**
     * Registers a client function that can be called by the server.
     * 
     * @param FunctionName The name of the function to register
     * @param Object The object that will handle the function
     * @param FunctionPtr The function pointer to call when the server calls this function
     * @return True if registration was successful
     */
    template<class UserClass>
    bool RegisterRPCHandler(const FString& FunctionName, UserClass* Object, void(UserClass::*FunctionPtr)(int64, const TArray<FStdbRpcArg>&))
    {
        if (!Object || FunctionName.IsEmpty())
        {
            return false;
        }
        
        // Create a lambda that calls the member function
        auto Handler = [Object, FunctionPtr](int64 ObjectId, const TArray<FStdbRpcArg>& Args) {
            (Object->*FunctionPtr)(ObjectId, Args);
        };
        
        // Register with internal map
        ClientRpcHandlers.Add(FunctionName, Handler);
        
        // Register with Rust FFI
        return RegisterClientFunctionWithFFI(FunctionName);
    }
    
    /** Register an object for client-side prediction */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    bool RegisterPredictionObject(const FObjectID& ObjectID);

    /** Unregister an object from client-side prediction */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    bool UnregisterPredictionObject(const FObjectID& ObjectID);

    /** Get the next sequence number for an object */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    int32 GetNextPredictionSequence(const FObjectID& ObjectID);

    /** Send a predicted transform update to the server */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    bool SendPredictedTransform(const FPredictedTransformData& TransformData);

    /** Get the last acknowledged sequence number for an object */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    int32 GetLastAckedSequence(const FObjectID& ObjectID);

    /** Process a server transform update with sequence number */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
    void ProcessServerTransformUpdate(const FObjectID& ObjectID, const FTransform& Transform, const FVector& Velocity, int32 AckedSequence);

    //============================
    // Component Replication
    //============================
    
    /**
     * Handles a component being added to an actor from the server.
     * 
     * @param ActorId The SpacetimeDB object ID of the actor
     * @param ComponentId The SpacetimeDB object ID of the component
     * @param ComponentClassName The class name of the component
     * @param DataJson JSON string containing the component's initial data
     * @return The created component UObject, or nullptr if creation failed
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    UActorComponent* HandleComponentAdded(int64 ActorId, int64 ComponentId, const FString& ComponentClassName, const FString& DataJson);
    
    /**
     * Handles a component being removed from an actor from the server.
     * 
     * @param ActorId The SpacetimeDB object ID of the actor
     * @param ComponentId The SpacetimeDB object ID of the component
     * @return True if successfully removed, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    bool HandleComponentRemoved(int64 ActorId, int64 ComponentId);
    
    /**
     * Gets a component by its SpacetimeDB object ID.
     * 
     * @param ComponentId The SpacetimeDB object ID of the component
     * @return The UActorComponent pointer, or nullptr if not found
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    UActorComponent* GetComponentById(int64 ComponentId) const;
    
    /**
     * Gets all components for an actor by its SpacetimeDB object ID.
     * 
     * @param ActorId The SpacetimeDB object ID of the actor
     * @return Array of component IDs attached to the actor
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    TArray<int64> GetComponentIdsForActor(int64 ActorId) const;
    
    /**
     * Gets all components for an actor.
     * 
     * @param Actor The actor to get components for
     * @return Array of component IDs attached to the actor
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    TArray<int64> GetComponentIdsForActor(AActor* Actor) const;
    
    /**
     * Adds a component to an actor (client-side request).
     * 
     * @param ActorId The SpacetimeDB object ID of the actor
     * @param ComponentClassName The class name of the component to add
     * @return The temporary component ID, or 0 if failed
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    int64 RequestAddComponent(int64 ActorId, const FString& ComponentClassName);
    
    /**
     * Removes a component from an actor (client-side request).
     * 
     * @param ActorId The SpacetimeDB object ID of the actor
     * @param ComponentId The SpacetimeDB object ID of the component
     * @return True if the request was sent successfully, false otherwise
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Components")
    bool RequestRemoveComponent(int64 ActorId, int64 ComponentId);

    /** Event that fires when a component is added to an actor */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Components")
    FOnComponentAddedDynamic OnComponentAdded;
    
    /** Event that fires when a component is removed from an actor */
    UPROPERTY(BlueprintAssignable, Category = "SpacetimeDB|Components")
    FOnComponentRemovedDynamic OnComponentRemoved;

private:
    // RPC delegate types
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(FOnServerRpcReceived, int64, ObjectId, const FString&, FunctionName, const TArray<FStdbRpcArg>&, Arguments);
    
    // Dynamic multicast delegates
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnConnectedDynamic);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnDisconnectedDynamic, const FString&, Reason);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnIdentityReceivedDynamic, const FString&, Identity);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnEventReceivedDynamic, const FString&, TableName, const FString&, EventData);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnErrorOccurredDynamic, const FString&, ErrorMessage);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_ThreeParams(FOnObjectCreatedDynamic, int64, ObjectId, const FString&, ClassName, const FString&, InitialDataJson);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnObjectDestroyedDynamic, int64, ObjectId);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnObjectIdRemappedDynamic, int64, TempId, int64, ServerId);
    
    // Callback handlers for FFI events
    void HandleConnected();
    void HandleDisconnected(const FString& Reason);
    void HandleIdentityReceived(const FString& Identity);
    void HandleEventReceived(const FString& TableName, const FString& EventData);
    void HandleErrorOccurred(const FString& ErrorMessage);
    void HandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson);
    void HandleObjectDestroyed(uint64 ObjectId);
    void HandleObjectIdRemapped(uint64 TempId, uint64 ServerId);
    
    // Client instance
    FSpacetimeDBClient Client;
    
    // Property update info structure
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
    
    // Type definition for client RPC handlers
    typedef TFunction<void(int64, const TArray<FStdbRpcArg>&)> FClientRpcHandler;
    
    // Map of registered client RPC handlers
    TMap<FString, FClientRpcHandler> ClientRpcHandlers;
    
    // Register a client function with FFI
    bool RegisterClientFunctionWithFFI(const FString& FunctionName);
    
    // Static callback function for client RPCs (called from FFI)
    static bool HandleClientRpcFromFFI(uint64 ObjectId, const char* ArgsJson);
    
    // Handle client RPC on the game thread
    void HandleClientRpc(uint64 ObjectId, const FString& FunctionName, TSharedPtr<FJsonObject> ArgsObj);
    
    // Parse JSON arguments into an array of FStdbRpcArg
    TArray<FStdbRpcArg> ParseRpcArguments(const FString& ArgsJson);
    
    // Convert array of FStdbRpcArg to JSON string
    FString SerializeRpcArguments(const TArray<FStdbRpcArg>& Args);
}; 