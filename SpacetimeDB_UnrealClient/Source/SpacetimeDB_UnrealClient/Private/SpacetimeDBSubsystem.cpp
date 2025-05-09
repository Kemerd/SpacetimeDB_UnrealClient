// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBSubsystem.h"
#include "Engine/Engine.h"
#include "Async/Async.h"

void USpacetimeDBSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
    Super::Initialize(Collection);
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Initializing"));
    
    // Register for client events
    OnConnectedHandle = Client.OnConnected.AddUObject(this, &USpacetimeDBSubsystem::HandleConnected);
    OnDisconnectedHandle = Client.OnDisconnected.AddUObject(this, &USpacetimeDBSubsystem::HandleDisconnected);
    OnIdentityReceivedHandle = Client.OnIdentityReceived.AddUObject(this, &USpacetimeDBSubsystem::HandleIdentityReceived);
    OnEventReceivedHandle = Client.OnEventReceived.AddUObject(this, &USpacetimeDBSubsystem::HandleEventReceived);
    OnErrorOccurredHandle = Client.OnErrorOccurred.AddUObject(this, &USpacetimeDBSubsystem::HandleErrorOccurred);
    
    // Register for object system events
    OnPropertyUpdatedHandle = Client.OnPropertyUpdated.AddUObject(this, &USpacetimeDBSubsystem::HandlePropertyUpdated);
    OnObjectCreatedHandle = Client.OnObjectCreated.AddUObject(this, &USpacetimeDBSubsystem::HandleObjectCreated);
    OnObjectDestroyedHandle = Client.OnObjectDestroyed.AddUObject(this, &USpacetimeDBSubsystem::HandleObjectDestroyed);
    OnObjectIdRemappedHandle = Client.OnObjectIdRemapped.AddUObject(this, &USpacetimeDBSubsystem::HandleObjectIdRemapped);
}

void USpacetimeDBSubsystem::Deinitialize()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Deinitializing"));
    
    // Disconnect if still connected
    if (IsConnected())
    {
        Disconnect();
    }
    
    // Unregister from client events
    if (OnConnectedHandle.IsValid())
    {
        Client.OnConnected.Remove(OnConnectedHandle);
        OnConnectedHandle.Reset();
    }
    
    if (OnDisconnectedHandle.IsValid())
    {
        Client.OnDisconnected.Remove(OnDisconnectedHandle);
        OnDisconnectedHandle.Reset();
    }
    
    if (OnIdentityReceivedHandle.IsValid())
    {
        Client.OnIdentityReceived.Remove(OnIdentityReceivedHandle);
        OnIdentityReceivedHandle.Reset();
    }
    
    if (OnEventReceivedHandle.IsValid())
    {
        Client.OnEventReceived.Remove(OnEventReceivedHandle);
        OnEventReceivedHandle.Reset();
    }
    
    if (OnErrorOccurredHandle.IsValid())
    {
        Client.OnErrorOccurred.Remove(OnErrorOccurredHandle);
        OnErrorOccurredHandle.Reset();
    }
    
    // Unregister from object system events
    if (OnPropertyUpdatedHandle.IsValid())
    {
        Client.OnPropertyUpdated.Remove(OnPropertyUpdatedHandle);
        OnPropertyUpdatedHandle.Reset();
    }
    
    if (OnObjectCreatedHandle.IsValid())
    {
        Client.OnObjectCreated.Remove(OnObjectCreatedHandle);
        OnObjectCreatedHandle.Reset();
    }
    
    if (OnObjectDestroyedHandle.IsValid())
    {
        Client.OnObjectDestroyed.Remove(OnObjectDestroyedHandle);
        OnObjectDestroyedHandle.Reset();
    }
    
    if (OnObjectIdRemappedHandle.IsValid())
    {
        Client.OnObjectIdRemapped.Remove(OnObjectIdRemappedHandle);
        OnObjectIdRemappedHandle.Reset();
    }
    
    Super::Deinitialize();
}

bool USpacetimeDBSubsystem::Connect(const FString& Host, const FString& DatabaseName, const FString& AuthToken)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Connect(%s, %s, %s)"), *Host, *DatabaseName, AuthToken.IsEmpty() ? TEXT("<empty>") : TEXT("<token>"));
    return Client.Connect(Host, DatabaseName, AuthToken);
}

bool USpacetimeDBSubsystem::Disconnect()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Disconnect()"));
    return Client.Disconnect();
}

bool USpacetimeDBSubsystem::IsConnected() const
{
    return Client.IsConnected();
}

int64 USpacetimeDBSubsystem::GetSpacetimeDBClientID() const
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: GetSpacetimeDBClientID() called while not connected"));
        return 0;
    }

    // Get client ID from the client wrapper
    uint64 ClientID = Client.GetClientID();
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: GetSpacetimeDBClientID() returning %llu"), ClientID);
    return static_cast<int64>(ClientID);
}

bool USpacetimeDBSubsystem::CallReducer(const FString& ReducerName, const FString& ArgsJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: CallReducer(%s, %s)"), *ReducerName, *ArgsJson);
    return Client.CallReducer(ReducerName, ArgsJson);
}

bool USpacetimeDBSubsystem::SubscribeToTables(const TArray<FString>& TableNames)
{
    if (TableNames.Num() > 0)
    {
        FString TablesStr;
        for (const FString& TableName : TableNames)
        {
            if (!TablesStr.IsEmpty())
            {
                TablesStr += TEXT(", ");
            }
            TablesStr += TableName;
        }
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: SubscribeToTables(%s)"), *TablesStr);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SubscribeToTables() called with empty list"));
    }
    
    return Client.SubscribeToTables(TableNames);
}

FString USpacetimeDBSubsystem::GetClientIdentity() const
{
    return Client.GetClientIdentity();
}

// Event handlers to forward client events to Blueprint events

void USpacetimeDBSubsystem::HandleConnected()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Connected event received"));
    OnConnected.Broadcast();
    
    // Optional: Display a notification in game if desired
    if (GEngine)
    {
        GEngine->AddOnScreenDebugMessage(-1, 5.0f, FColor::Green, TEXT("Connected to SpacetimeDB"));
    }
}

void USpacetimeDBSubsystem::HandleDisconnected(const FString& Reason)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Disconnected event received: %s"), *Reason);
    OnDisconnected.Broadcast(Reason);
    
    // Optional: Display a notification in game if desired
    if (GEngine)
    {
        FString Message = FString::Printf(TEXT("Disconnected from SpacetimeDB: %s"), *Reason);
        GEngine->AddOnScreenDebugMessage(-1, 5.0f, FColor::Yellow, Message);
    }
}

void USpacetimeDBSubsystem::HandleIdentityReceived(const FString& Identity)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Identity received: %s"), *Identity);
    OnIdentityReceived.Broadcast(Identity);
}

void USpacetimeDBSubsystem::HandleEventReceived(const FString& TableName, const FString& EventData)
{
    // Use Verbose log level since this could be high frequency
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Event received for table %s"), *TableName);
    OnEventReceived.Broadcast(TableName, EventData);
}

void USpacetimeDBSubsystem::HandleErrorOccurred(const FString& ErrorMessage)
{
    UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Error occurred: %s"), *ErrorMessage);
    OnErrorOccurred.Broadcast(ErrorMessage);
    
    // Optional: Display a notification in game if desired
    if (GEngine)
    {
        FString Message = FString::Printf(TEXT("SpacetimeDB Error: %s"), *ErrorMessage);
        GEngine->AddOnScreenDebugMessage(-1, 5.0f, FColor::Red, Message);
    }
}

// Object system event handlers

void USpacetimeDBSubsystem::HandlePropertyUpdated(uint64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Use Verbose log level since this could be high frequency
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Property updated - Object %llu, Property %s"), ObjectId, *PropertyName);
    OnPropertyUpdated.Broadcast(static_cast<int64>(ObjectId), PropertyName, ValueJson);
    
    // TODO: Once object tracking system is implemented, apply property to local object
}

void USpacetimeDBSubsystem::HandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object created - ID %llu, Class %s"), ObjectId, *ClassName);
    OnObjectCreated.Broadcast(static_cast<int64>(ObjectId), ClassName, DataJson);
    
    // TODO: Once object tracking system is implemented, create local object
}

void USpacetimeDBSubsystem::HandleObjectDestroyed(uint64 ObjectId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object destroyed - ID %llu"), ObjectId);
    OnObjectDestroyed.Broadcast(static_cast<int64>(ObjectId));
    
    // TODO: Once object tracking system is implemented, destroy local object
}

void USpacetimeDBSubsystem::HandleObjectIdRemapped(uint64 TempId, uint64 ServerId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object ID remapped - Temp ID %llu -> Server ID %llu"), TempId, ServerId);
    OnObjectIdRemapped.Broadcast(static_cast<int64>(TempId), static_cast<int64>(ServerId));
    
    // TODO: Once object tracking system is implemented, update object ID mappings
} 