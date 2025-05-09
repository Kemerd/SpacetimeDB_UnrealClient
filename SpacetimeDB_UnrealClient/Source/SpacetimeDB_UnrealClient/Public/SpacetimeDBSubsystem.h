// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Subsystems/GameInstanceSubsystem.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDBSubsystem.generated.h"

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
    
private:
    // The client instance
    FSpacetimeDBClient Client;
    
    // Delegate handles for event forwarding
    FDelegateHandle OnConnectedHandle;
    FDelegateHandle OnDisconnectedHandle;
    FDelegateHandle OnIdentityReceivedHandle;
    FDelegateHandle OnEventReceivedHandle;
    FDelegateHandle OnErrorOccurredHandle;
    
    // Dynamic multicast delegate declarations for Blueprint exposed events
    DECLARE_DYNAMIC_MULTICAST_DELEGATE(FOnConnectedDynamic);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnDisconnectedDynamic, const FString&, Reason);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnIdentityReceivedDynamic, const FString&, Identity);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnEventReceivedDynamic, const FString&, TableName, const FString&, EventData);
    DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnErrorOccurredDynamic, const FString&, ErrorMessage);
    
    // Event handlers to forward client events to Blueprint events
    void HandleConnected();
    void HandleDisconnected(const FString& Reason);
    void HandleIdentityReceived(const FString& Identity);
    void HandleEventReceived(const FString& TableName, const FString& EventData);
    void HandleErrorOccurred(const FString& ErrorMessage);
}; 