// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBClient.h"
#include "HAL/UnrealMemory.h"
#include "Async/Async.h"
#include "SpacetimeDBSubsystem.h"
#include "Engine/GameInstance.h"
#include "ffi.h" // Include the generated FFI header file
#include "SpacetimeDB_ErrorHandler.h"

// Initialize static singleton instance for callbacks
FSpacetimeDBClient* FSpacetimeDBClient::Instance = nullptr;

FSpacetimeDBClient::FSpacetimeDBClient()
{
    // Set the singleton instance for callbacks
    // Note: This approach assumes a single instance of FSpacetimeDBClient
    // For multiple instances, a more sophisticated approach would be needed
    if (Instance == nullptr)
    {
        Instance = this;
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("Multiple FSpacetimeDBClient instances created. Callback behavior may be unpredictable."));
    }
}

FSpacetimeDBClient::~FSpacetimeDBClient()
{
    // Only clean up if this is the active instance
    if (Instance == this)
    {
        Disconnect(); // Ensure we're disconnected
        Instance = nullptr;
    }
}

bool FSpacetimeDBClient::Connect(const FString& Host, const FString& DatabaseName, const FString& AuthToken)
{
    UE_LOG(LogSpacetimeDB, Log, TEXT("Connecting to %s/%s"), *Host, *DatabaseName);
    
    // Validate input parameters
    if (Host.IsEmpty())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Empty host provided for connection"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Connection"),
            1001
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    if (DatabaseName.IsEmpty())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Empty database name provided for connection"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Connection"),
            1002
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Check if already connected
    if (IsConnected())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Already connected to SpacetimeDB. Disconnect first before connecting again."),
            ESpacetimeDBErrorSeverity::Warning,
            TEXT("Connection"),
            1003
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Create FFI connection config
    stdb::ffi::ConnectionConfig config;
    config.host = TCHAR_TO_UTF8(*Host);
    config.db_name = TCHAR_TO_UTF8(*DatabaseName);
    config.auth_token = TCHAR_TO_UTF8(*AuthToken);
    
    // Set up callback function pointers
    stdb::ffi::EventCallbackPointers callbacks;
    callbacks.on_connected = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnConnectedCallback);
    callbacks.on_disconnected = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnDisconnectedCallback);
    callbacks.on_event_received = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnEventReceivedCallback);
    callbacks.on_error_occurred = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnErrorOccurredCallback);
    
    // Add object system callbacks
    callbacks.on_property_updated = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnPropertyUpdatedCallback);
    callbacks.on_object_created = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnObjectCreatedCallback);
    callbacks.on_object_destroyed = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnObjectDestroyedCallback);
    callbacks.on_object_id_remapped = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnObjectIdRemappedCallback);
    
    // Add component system callbacks
    callbacks.on_component_added = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnComponentAddedCallback);
    callbacks.on_component_removed = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnComponentRemovedCallback);
    
    // Call the Rust function through FFI and capture the result
    bool bResult = stdb::ffi::connect_to_server(config, callbacks);
    
    // No need for try/catch in UE4/5 - it doesn't support exceptions
    if (!bResult)
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Failed to initiate connection to SpacetimeDB"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Connection"),
            1004,
            FString::Printf(TEXT("Host: %s, Database: %s"), *Host, *DatabaseName)
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
    }
    
    return bResult;
}

bool FSpacetimeDBClient::Disconnect()
{
    UE_LOG(LogSpacetimeDB, Log, TEXT("Disconnecting from SpacetimeDB"));
    
    // Check if already disconnected
    if (!IsConnected())
    {
        UE_LOG(LogSpacetimeDB, Verbose, TEXT("Already disconnected from SpacetimeDB"));
        return true; // Not an error, already disconnected
    }
    
    // Call the FFI function and capture the result
    bool bResult = stdb::ffi::disconnect_from_server();
    
    if (!bResult)
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Failed to disconnect from SpacetimeDB"),
            ESpacetimeDBErrorSeverity::Warning, // Warning, not Error since we can try again
            TEXT("Connection"),
            1010
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
    }
    
    return bResult;
}

bool FSpacetimeDBClient::IsConnected() const
{
    return stdb::ffi::is_client_connected();
}

bool FSpacetimeDBClient::CallReducer(const FString& ReducerName, const FString& ArgsJson)
{
    UE_LOG(LogSpacetimeDB, Log, TEXT("Calling reducer %s with args: %s"), *ReducerName, *ArgsJson);
    
    // Check connection state
    if (!IsConnected())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Cannot call reducer - Not connected to SpacetimeDB"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Reducer"),
            2001,
            FString::Printf(TEXT("Reducer: %s"), *ReducerName)
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Validate parameters
    if (ReducerName.IsEmpty())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Empty reducer name provided"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Reducer"),
            2002
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Prepare strings for FFI - convert to std::string as required by the FFI function
    std::string stdReducerName = TCHAR_TO_UTF8(*ReducerName);
    std::string stdArgsJson = TCHAR_TO_UTF8(*ArgsJson);
    
    // Call the FFI function and capture the result
    bool bResult = stdb::ffi::call_reducer(stdReducerName, stdArgsJson);
    
    if (!bResult)
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Failed to call reducer"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Reducer"),
            2003,
            FString::Printf(TEXT("Reducer: %s, Args: %s"), *ReducerName, *ArgsJson)
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
    }
    
    return bResult;
}

bool FSpacetimeDBClient::SubscribeToTables(const TArray<FString>& TableNames)
{
    // Create log message with table names for debugging
    FString TablesStr;
    for (const FString& TableName : TableNames)
    {
        if (!TablesStr.IsEmpty())
        {
            TablesStr += TEXT(", ");
        }
        TablesStr += TableName;
    }
    
    UE_LOG(LogSpacetimeDB, Log, TEXT("Subscribing to tables: [%s]"), TableNames.Num() > 0 ? *TablesStr : TEXT("none"));
    
    // Check if connected
    if (!IsConnected())
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Cannot subscribe to tables - Not connected to SpacetimeDB"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Subscription"),
            3001
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Check if we have tables to subscribe to
    if (TableNames.Num() == 0)
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("No tables specified for subscription"),
            ESpacetimeDBErrorSeverity::Warning,
            TEXT("Subscription"),
            3002
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
        
        return false;
    }
    
    // Convert from Unreal's TArray<FString> to a std::vector<std::string>
    std::vector<std::string> stdTableNames;
    for (const FString& TableName : TableNames)
    {
        stdTableNames.push_back(TCHAR_TO_UTF8(*TableName));
    }
    
    // Call the FFI function and capture the result
    bool bResult = stdb::ffi::subscribe_to_tables(stdTableNames);
    
    if (!bResult)
    {
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::LogError(
            TEXT("Failed to subscribe to tables"),
            ESpacetimeDBErrorSeverity::Error,
            TEXT("Subscription"),
            3003,
            FString::Printf(TEXT("Tables: [%s]"), *TablesStr)
        );
        
        // Execute on game thread to ensure thread safety
        AsyncTask(ENamedThreads::GameThread, [this, ErrorInfo]() {
            OnErrorOccurred.Broadcast(ErrorInfo);
        });
    }
    
    return bResult;
}

FString FSpacetimeDBClient::GetClientIdentity() const
{
    rust::String identityStr = stdb::ffi::get_client_identity();
    return UTF8_TO_TCHAR(identityStr.c_str());
}

uint64 FSpacetimeDBClient::GetClientID() const
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBClient: GetClientID() called while not connected"));
        return 0;
    }
    
    return stdb::ffi::get_client_id();
}

// ---- Static callback implementations ----

void FSpacetimeDBClient::OnConnectedCallback()
{
    // Execute on game thread
    if (Instance)
    {
        AsyncTask(ENamedThreads::GameThread, []() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Connected successfully to SpacetimeDB"));
            Instance->OnConnected.Broadcast();
        });
    }
}

void FSpacetimeDBClient::OnDisconnectedCallback(const char* Reason)
{
    if (Instance)
    {
        FString ReasonStr = UTF8_TO_TCHAR(Reason);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ReasonStr]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Disconnected from SpacetimeDB - Reason: %s"), *ReasonStr);
            Instance->OnDisconnected.Broadcast(ReasonStr);
        });
    }
}

void FSpacetimeDBClient::OnEventReceivedCallback(const char* EventData, const char* TableName)
{
    if (Instance)
    {
        FString EventDataStr = UTF8_TO_TCHAR(EventData);
        FString TableNameStr = UTF8_TO_TCHAR(TableName);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [EventDataStr, TableNameStr]() {
            UE_LOG(LogSpacetimeDB, Verbose, TEXT("Event received for table '%s'"), *TableNameStr);
            Instance->OnEventReceived.Broadcast(TableNameStr, EventDataStr);
        });
    }
}

void FSpacetimeDBClient::OnErrorOccurredCallback(const char* ErrorMessage)
{
    if (Instance)
    {
        FString ErrorMessageStr = UTF8_TO_TCHAR(ErrorMessage);
        
        // Process the error and create a structured error info object
        FSpacetimeDBErrorInfo ErrorInfo = FSpacetimeDBErrorHandler::HandleFFIError(
            TEXT("FFI_Callback"),  // Generic function name since this is a callback
            ErrorMessageStr,
            false  // Don't log stack trace here as we're in a callback
        );
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ErrorInfo]() {
            // Use our dedicated log category instead of LogTemp
            UE_LOG(LogSpacetimeDB, Error, TEXT("Error: %s"), *ErrorInfo.Message);
            
            // Broadcast with rich error info
            Instance->OnErrorOccurred.Broadcast(ErrorInfo);
        });
    }
}

// --- New callback implementations ---

void FSpacetimeDBClient::OnPropertyUpdatedCallback(uint64 ObjectId, const char* PropertyName, const char* ValueJson)
{
    if (Instance)
    {
        FString PropertyNameStr = UTF8_TO_TCHAR(PropertyName);
        FString ValueJsonStr = UTF8_TO_TCHAR(ValueJson);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ObjectId, PropertyNameStr, ValueJsonStr]() {
            UE_LOG(LogSpacetimeDB, Verbose, TEXT("Property updated - Object %llu, Property '%s'"), ObjectId, *PropertyNameStr);
            Instance->OnPropertyUpdated.Broadcast(ObjectId, PropertyNameStr, ValueJsonStr);
        });
    }
}

void FSpacetimeDBClient::OnObjectCreatedCallback(uint64 ObjectId, const char* ClassName, const char* DataJson)
{
    if (Instance)
    {
        FString ClassNameStr = UTF8_TO_TCHAR(ClassName);
        FString DataJsonStr = UTF8_TO_TCHAR(DataJson);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ObjectId, ClassNameStr, DataJsonStr]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Object created - ID: %llu, Class: '%s'"), ObjectId, *ClassNameStr);
            Instance->OnObjectCreated.Broadcast(ObjectId, ClassNameStr, DataJsonStr);
        });
    }
}

void FSpacetimeDBClient::OnObjectDestroyedCallback(uint64 ObjectId)
{
    if (Instance)
    {
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ObjectId]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Object destroyed - ID: %llu"), ObjectId);
            Instance->OnObjectDestroyed.Broadcast(ObjectId);
        });
    }
}

void FSpacetimeDBClient::OnObjectIdRemappedCallback(uint64 TempId, uint64 ServerId)
{
    if (Instance)
    {
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [TempId, ServerId]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Object ID remapped - Temp ID: %llu -> Server ID: %llu"), TempId, ServerId);
            Instance->OnObjectIdRemapped.Broadcast(TempId, ServerId);
        });
    }
}

void FSpacetimeDBClient::OnComponentAddedCallback(uint64 ActorId, uint64 ComponentId, const char* ComponentClassName, const char* DataJson)
{
    if (Instance)
    {
        FString ComponentClassNameStr = UTF8_TO_TCHAR(ComponentClassName);
        FString DataJsonStr = UTF8_TO_TCHAR(DataJson);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ActorId, ComponentId, ComponentClassNameStr, DataJsonStr]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Component added - Actor: %llu, Component: %llu, Class: '%s'"), 
                ActorId, ComponentId, *ComponentClassNameStr);
            Instance->OnComponentAdded.Broadcast(ActorId, ComponentId, ComponentClassNameStr);
            
            // Find the subsystem to handle component creation
            for (TObjectIterator<UGameInstance> It; It; ++It)
            {
                if (UGameInstance* GameInstance = *It)
                {
                    if (IsValid(GameInstance) && GameInstance->GetWorld() && GameInstance->GetWorld()->IsGameWorld())
                    {
                        if (USpacetimeDBSubsystem* Subsystem = GameInstance->GetSubsystem<USpacetimeDBSubsystem>())
                        {
                            Subsystem->HandleComponentAdded(ActorId, ComponentId, ComponentClassNameStr, DataJsonStr);
                            break;
                        }
                    }
                }
            }
        });
    }
}

void FSpacetimeDBClient::OnComponentRemovedCallback(uint64 ActorId, uint64 ComponentId)
{
    if (Instance)
    {
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [ActorId, ComponentId]() {
            UE_LOG(LogSpacetimeDB, Log, TEXT("Component removed - Actor: %llu, Component: %llu"), ActorId, ComponentId);
            Instance->OnComponentRemoved.Broadcast(ActorId, ComponentId);
            
            // Find the subsystem to handle component removal
            for (TObjectIterator<UGameInstance> It; It; ++It)
            {
                if (UGameInstance* GameInstance = *It)
                {
                    if (IsValid(GameInstance) && GameInstance->GetWorld() && GameInstance->GetWorld()->IsGameWorld())
                    {
                        if (USpacetimeDBSubsystem* Subsystem = GameInstance->GetSubsystem<USpacetimeDBSubsystem>())
                        {
                            Subsystem->HandleComponentRemoved(ActorId, ComponentId);
                            break;
                        }
                    }
                }
            }
        });
    }
} 