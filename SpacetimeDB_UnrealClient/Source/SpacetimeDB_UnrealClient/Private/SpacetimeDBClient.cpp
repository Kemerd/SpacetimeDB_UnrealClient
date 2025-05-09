// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBClient.h"
#include "HAL/UnrealMemory.h"
#include "rust/stdb.hpp" // Include the generated FFI header file

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
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Connecting to %s/%s"), *Host, *DatabaseName);
    
    // Create FFI connection config
    stdb::ffi::ConnectionConfig config;
    config.host = TCHAR_TO_UTF8(*Host);
    config.db_name = TCHAR_TO_UTF8(*DatabaseName);
    config.auth_token = TCHAR_TO_UTF8(*AuthToken);
    
    // Set up callback function pointers
    stdb::ffi::EventCallbackPointers callbacks;
    callbacks.on_connected = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnConnectedCallback);
    callbacks.on_disconnected = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnDisconnectedCallback);
    callbacks.on_identity_received = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnIdentityReceivedCallback);
    callbacks.on_event_received = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnEventReceivedCallback);
    callbacks.on_error_occurred = reinterpret_cast<uintptr_t>(&FSpacetimeDBClient::OnErrorOccurredCallback);
    
    // Call the Rust function through FFI
    return stdb::ffi::connect_to_server(config, callbacks);
}

bool FSpacetimeDBClient::Disconnect()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Disconnecting"));
    return stdb::ffi::disconnect_from_server();
}

bool FSpacetimeDBClient::IsConnected() const
{
    return stdb::ffi::is_client_connected();
}

bool FSpacetimeDBClient::CallReducer(const FString& ReducerName, const FString& ArgsJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Calling reducer %s with args: %s"), *ReducerName, *ArgsJson);
    
    // Rusty strings for FFI
    rust::String rustReducerName = rust::String(TCHAR_TO_UTF8(*ReducerName));
    rust::String rustArgsJson = rust::String(TCHAR_TO_UTF8(*ArgsJson));
    
    return stdb::ffi::call_reducer(rustReducerName, rustArgsJson);
}

bool FSpacetimeDBClient::SubscribeToTables(const TArray<FString>& TableNames)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Subscribing to %d tables"), TableNames.Num());
    
    // Convert from Unreal's TArray<FString> to a rust::Vec<rust::String>
    std::vector<rust::String> rustTableNames;
    for (const FString& TableName : TableNames)
    {
        rustTableNames.push_back(rust::String(TCHAR_TO_UTF8(*TableName)));
    }
    
    return stdb::ffi::subscribe_to_tables(rustTableNames);
}

FString FSpacetimeDBClient::GetClientIdentity() const
{
    rust::String identityStr = stdb::ffi::get_client_identity();
    return UTF8_TO_TCHAR(identityStr.c_str());
}

// ---- Static callback implementations ----

void FSpacetimeDBClient::OnConnectedCallback()
{
    // Execute on game thread
    if (Instance)
    {
        AsyncTask(ENamedThreads::GameThread, [=]() {
            UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Connected successfully"));
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
        AsyncTask(ENamedThreads::GameThread, [=]() {
            UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Disconnected - %s"), *ReasonStr);
            Instance->OnDisconnected.Broadcast(ReasonStr);
        });
    }
}

void FSpacetimeDBClient::OnIdentityReceivedCallback(const char* Identity)
{
    if (Instance)
    {
        FString IdentityStr = UTF8_TO_TCHAR(Identity);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [=]() {
            UE_LOG(LogTemp, Log, TEXT("SpacetimeDBClient: Identity received - %s"), *IdentityStr);
            Instance->OnIdentityReceived.Broadcast(IdentityStr);
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
        AsyncTask(ENamedThreads::GameThread, [=]() {
            UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBClient: Event received for table %s"), *TableNameStr);
            Instance->OnEventReceived.Broadcast(TableNameStr, EventDataStr);
        });
    }
}

void FSpacetimeDBClient::OnErrorOccurredCallback(const char* ErrorMessage)
{
    if (Instance)
    {
        FString ErrorMessageStr = UTF8_TO_TCHAR(ErrorMessage);
        
        // Execute on game thread
        AsyncTask(ENamedThreads::GameThread, [=]() {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBClient: Error - %s"), *ErrorMessageStr);
            Instance->OnErrorOccurred.Broadcast(ErrorMessageStr);
        });
    }
} 