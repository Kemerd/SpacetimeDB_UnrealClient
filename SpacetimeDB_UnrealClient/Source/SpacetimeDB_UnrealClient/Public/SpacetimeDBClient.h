// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Containers/UnrealString.h"
#include "Containers/Array.h"
#include "SpacetimeDB_ErrorHandler.h"

// Forward declarations for SpacetimeDB FFI types
namespace stdb {
    namespace ffi {
        struct ConnectionConfig;
        struct EventCallbackPointers;
    }
}

/**
 * @class FSpacetimeDBClient
 * @brief C++ wrapper for SpacetimeDB client functionality.
 * 
 * This class provides a clean C++ interface for interacting with the SpacetimeDB
 * Rust client library via FFI. It handles connection management, reducer calls,
 * subscriptions, and event handling.
 */
class SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBClient
{
public:
    /** Delegate for handling SpacetimeDB connection established events */
    DECLARE_MULTICAST_DELEGATE(FOnConnected);
    
    /** Delegate for handling SpacetimeDB disconnection events */
    DECLARE_MULTICAST_DELEGATE_OneParam(FOnDisconnected, const FString& /* Reason */);
    
    /** Delegate for handling SpacetimeDB identity received events */
    DECLARE_MULTICAST_DELEGATE_OneParam(FOnIdentityReceived, const FString& /* Identity */);
    
    /** Delegate for handling SpacetimeDB table events */
    DECLARE_MULTICAST_DELEGATE_TwoParams(FOnEventReceived, const FString& /* TableName */, const FString& /* EventData */);
    
    /** Delegate for handling SpacetimeDB error events with detailed error info */
    DECLARE_MULTICAST_DELEGATE_OneParam(FOnErrorOccurred, const FSpacetimeDBErrorInfo& /* ErrorInfo */);
    
    /** Delegate for when a property is updated on an object */
    DECLARE_MULTICAST_DELEGATE_ThreeParams(FOnPropertyUpdated, uint64 /* ObjectId */, const FString& /* PropertyName */, const FString& /* ValueJson */);
    
    /** Delegate for when an object is created */
    DECLARE_MULTICAST_DELEGATE_ThreeParams(FOnObjectCreated, uint64 /* ObjectId */, const FString& /* ClassName */, const FString& /* DataJson */);
    
    /** Delegate for when an object is destroyed */
    DECLARE_MULTICAST_DELEGATE_OneParam(FOnObjectDestroyed, uint64 /* ObjectId */);
    
    /** Delegate for when an object ID is remapped (temporary to server-assigned) */
    DECLARE_MULTICAST_DELEGATE_TwoParams(FOnObjectIdRemapped, uint64 /* TempId */, uint64 /* ServerId */);
    
    /** Delegate that is broadcast when a component is added to an actor */
    DECLARE_MULTICAST_DELEGATE_ThreeParams(FOnComponentAdded, uint64 /* ActorId */, uint64 /* ComponentId */, const FString& /* ComponentClassName */);
    FOnComponentAdded OnComponentAdded;
    
    /** Delegate that is broadcast when a component is removed from an actor */
    DECLARE_MULTICAST_DELEGATE_TwoParams(FOnComponentRemoved, uint64 /* ActorId */, uint64 /* ComponentId */);
    FOnComponentRemoved OnComponentRemoved;
    
public:
    /** Default constructor */
    FSpacetimeDBClient();
    
    /** Destructor - ensures client is disconnected */
    ~FSpacetimeDBClient();
    
    /**
     * Connects to a SpacetimeDB instance.
     * 
     * @param Host The host address (e.g., "localhost:3000" or "api.spacetimedb.com")
     * @param DatabaseName The name of the database to connect to
     * @param AuthToken Optional authentication token
     * @return True if connection initiated successfully, false otherwise
     */
    bool Connect(const FString& Host, const FString& DatabaseName, const FString& AuthToken = TEXT(""));
    
    /**
     * Disconnects from the SpacetimeDB instance.
     * 
     * @return True if disconnection was successful, false if not connected
     */
    bool Disconnect();
    
    /**
     * Checks if the client is currently connected.
     * 
     * @return True if connected, false otherwise
     */
    bool IsConnected() const;
    
    /**
     * Calls a reducer function on the SpacetimeDB instance.
     * 
     * @param ReducerName The name of the reducer to call
     * @param ArgsJson A JSON string with the arguments for the reducer
     * @return True if the call was initiated successfully, false otherwise
     */
    bool CallReducer(const FString& ReducerName, const FString& ArgsJson);
    
    /**
     * Subscribes to one or more tables in the SpacetimeDB instance.
     * 
     * @param TableNames Array of table names to subscribe to
     * @return True if the subscription was initiated successfully, false otherwise
     */
    bool SubscribeToTables(const TArray<FString>& TableNames);
    
    /**
     * Gets the client's identity as a hex string.
     * 
     * @return The identity hex string, or empty string if not available
     */
    FString GetClientIdentity() const;
    
    /**
     * Gets the client's numerical ID.
     * 
     * @return The client ID as a 64-bit integer, or 0 if not connected
     */
    uint64 GetClientID() const;
    
    /** Delegate that is broadcast when the connection is established */
    FOnConnected OnConnected;
    
    /** Delegate that is broadcast when the connection is closed */
    FOnDisconnected OnDisconnected;
    
    /** Delegate that is broadcast when the client identity is received */
    FOnIdentityReceived OnIdentityReceived;
    
    /** Delegate that is broadcast when a table event is received */
    FOnEventReceived OnEventReceived;
    
    /** Delegate that is broadcast when an error occurs */
    FOnErrorOccurred OnErrorOccurred;
    
    /** Delegate that is broadcast when a property is updated on an object */
    FOnPropertyUpdated OnPropertyUpdated;
    
    /** Delegate that is broadcast when an object is created */
    FOnObjectCreated OnObjectCreated;
    
    /** Delegate that is broadcast when an object is destroyed */
    FOnObjectDestroyed OnObjectDestroyed;
    
    /** Delegate that is broadcast when an object ID is remapped */
    FOnObjectIdRemapped OnObjectIdRemapped;
    
private:
    // FFI callback functions
    static void OnConnectedCallback();
    static void OnDisconnectedCallback(const char* Reason);
    static void OnIdentityReceivedCallback(const char* Identity);
    static void OnEventReceivedCallback(const char* EventData, const char* TableName);
    static void OnErrorOccurredCallback(const char* ErrorMessage);
    
    // New FFI callback functions for object management
    static void OnPropertyUpdatedCallback(uint64 ObjectId, const char* PropertyName, const char* ValueJson);
    static void OnObjectCreatedCallback(uint64 ObjectId, const char* ClassName, const char* DataJson);
    static void OnObjectDestroyedCallback(uint64 ObjectId);
    static void OnObjectIdRemappedCallback(uint64 TempId, uint64 ServerId);
    
    // FFI callback functions for component management
    static void OnComponentAddedCallback(uint64 ActorId, uint64 ComponentId, const char* ComponentClassName, const char* DataJson);
    static void OnComponentRemovedCallback(uint64 ActorId, uint64 ComponentId);
    
    // Singleton instance pointer for callbacks
    static FSpacetimeDBClient* Instance;
}; 