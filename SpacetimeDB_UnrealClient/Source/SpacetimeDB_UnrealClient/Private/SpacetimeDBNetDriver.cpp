// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBNetDriver.h"
#include "SpacetimeDBNetConnection.h"
#include "SpacetimeDBClient.h"
#include "Engine/Engine.h"
#include "Engine/World.h"
#include "GameFramework/GameModeBase.h"
#include "Misc/NetworkVersion.h"
#include "Dom/JsonObject.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "SpacetimeDBNetDriverPrivate.h"

// FSpacetimeDBReplicationData structure for mapping between UE objects and SpacetimeDB data
struct FSpacetimeDBReplicationData
{
    // Actor ID in SpacetimeDB
    FString ActorId;
    
    // Class name for spawning
    FString ActorClass;
    
    // Actor properties as JSON
    TSharedPtr<FJsonObject> Properties;
    
    // Constructor
    FSpacetimeDBReplicationData()
        : ActorId("")
        , ActorClass("")
        , Properties(MakeShareable(new FJsonObject()))
    {
    }
};

// Private implementation data for the driver
class FSpacetimeDBNetDriverPrivate
{
public:
    FSpacetimeDBNetDriverPrivate()
        : bInitialized(false)
        , Host("")
        , Database("")
        , AuthToken("")
    {
    }
    
    // Connection state
    bool bInitialized;
    FString Host;
    FString Database;
    FString AuthToken;
    
    // Replication data for actors
    TMap<FString, FSpacetimeDBReplicationData> ActorReplicationData;
    
    // Buffer for outgoing packets to be processed in TickFlush
    TArray<TPair<FString, TArray<uint8>>> OutgoingPackets;
};

// Constructor
USpacetimeDBNetDriver::USpacetimeDBNetDriver(const FObjectInitializer& ObjectInitializer)
    : Super(ObjectInitializer)
{
    // Create private implementation
    NetDriverPrivate = new FSpacetimeDBNetDriverPrivate();
    
    // Initialize with default values
    bIsServer = false;
    ServerConnection = nullptr;
    
    // Register net driver as SpacetimeDB client
    NetConnectionClass = USpacetimeDBNetConnection::StaticClass();
    
    // Register for client events
    OnConnectedHandle = Client.OnConnected.AddUObject(this, &USpacetimeDBNetDriver::HandleConnected);
    OnDisconnectedHandle = Client.OnDisconnected.AddUObject(this, &USpacetimeDBNetDriver::HandleDisconnected);
    OnIdentityReceivedHandle = Client.OnIdentityReceived.AddUObject(this, &USpacetimeDBNetDriver::HandleIdentityReceived);
    OnEventReceivedHandle = Client.OnEventReceived.AddUObject(this, &USpacetimeDBNetDriver::HandleEventReceived);
    OnErrorOccurredHandle = Client.OnErrorOccurred.AddUObject(this, &USpacetimeDBNetDriver::HandleErrorOccurred);
}

bool USpacetimeDBNetDriver::IsAvailable() const
{
    // Driver is always available
    return true;
}

bool USpacetimeDBNetDriver::InitBase(bool bInitAsClient, FNetworkNotify* InNotify, const FURL& URL, bool bReuseAddressAndPort, FString& Error)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: InitBase bInitAsClient=%d URL=%s"), bInitAsClient ? 1 : 0, *URL.ToString());
    
    if (!Super::InitBase(bInitAsClient, InNotify, URL, bReuseAddressAndPort, Error))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: Super::InitBase failed"));
        return false;
    }
    
    // Get private data
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    // Parse URL parameters
    PrivateData->Host = URL.Host;
    PrivateData->Database = URL.Map;
    
    // Get auth token from options if provided
    if (URL.HasOption(TEXT("AuthToken")))
    {
        PrivateData->AuthToken = URL.GetOption(TEXT("AuthToken="), TEXT(""));
    }
    
    // Initialize base driver values
    bIsServer = !bInitAsClient;
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Initialized as %s"), bIsServer ? TEXT("SERVER") : TEXT("CLIENT"));
    
    // Setup relevancy settings - Note: We cannot directly call SetClientConnectionAlwaysRelevant
    // as that method either doesn't exist or has been renamed in this engine version
    // Instead configure the settings directly
    // bClientHasSpawned = true; // Commented out due to C2065, may need alternative for relevancy
    
    return true;
}

bool USpacetimeDBNetDriver::InitConnect(FNetworkNotify* InNotify, const FURL& ConnectURL, FString& Error)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: InitConnect URL=%s"), *ConnectURL.ToString());
    
    if (!InitBase(true, InNotify, ConnectURL, false, Error))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: InitBase failed for client connection"));
        return false;
    }
    
    // Get private implementation data
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    // Connect to SpacetimeDB
    if (!Client.Connect(PrivateData->Host, PrivateData->Database, PrivateData->AuthToken))
    {
        Error = TEXT("Failed to connect to SpacetimeDB server");
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: %s"), *Error);
        return false;
    }
    
    // Create the client connection
    ServerConnection = NewObject<USpacetimeDBNetConnection>(GetTransientPackage(), NetConnectionClass);
    check(ServerConnection != nullptr);
    
    // Initialize the connection
    ServerConnection->InitLocalConnection(this, nullptr, ConnectURL, USOCK_Open);
    
    // Register the connection
    ClientConnections.Add(ServerConnection);
    
    // Set as successfully initialized
    PrivateData->bInitialized = true;
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Client connection initialized"));
    return true;
}

bool USpacetimeDBNetDriver::InitListen(FNetworkNotify* InNotify, FURL& LocalURL, bool bReuseAddressAndPort, FString& Error)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: InitListen URL=%s"), *LocalURL.ToString());
    
    if (!InitBase(false, InNotify, LocalURL, bReuseAddressAndPort, Error))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: InitBase failed for server"));
        return false;
    }
    
    // Get private implementation data
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    // Connect to SpacetimeDB (servers also connect, they just operate differently)
    if (!Client.Connect(PrivateData->Host, PrivateData->Database, PrivateData->AuthToken))
    {
        Error = TEXT("Failed to connect to SpacetimeDB server");
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: %s"), *Error);
        return false;
    }
    
    // Mark server as initialized
    PrivateData->bInitialized = true;
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Server initialized"));
    return true;
}

void USpacetimeDBNetDriver::TickDispatch(float DeltaTime)
{
    // Process incoming data from SpacetimeDB
    // The SpacetimeDBClient handles this asynchronously via callbacks,
    // so we don't need to do anything special here.
    
    // Call parent implementation
    Super::TickDispatch(DeltaTime);
}

void USpacetimeDBNetDriver::TickFlush(float DeltaTime)
{
    // Call parent implementation first to gather outgoing packets
    Super::TickFlush(DeltaTime);
    
    // Now process any outgoing packets that were queued in LowLevelSend
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    if (PrivateData->OutgoingPackets.Num() > 0)
    {
        UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: TickFlush processing %d outgoing packets"), PrivateData->OutgoingPackets.Num());
        
        // Process each outgoing packet
        for (auto& Packet : PrivateData->OutgoingPackets)
        {
            const FString& ReducerName = Packet.Key;
            const TArray<uint8>& Data = Packet.Value;
            
            // Convert binary data to Base64 for JSON compatibility
            FString Base64Data = FBase64::Encode(Data.GetData(), Data.Num());
            
            // Create JSON for the reducer call
            FString ArgsJson = FString::Printf(TEXT("{\"data\": \"%s\"}"), *Base64Data);
            
            // Call the reducer
            Client.CallReducer(ReducerName, ArgsJson);
        }
        
        // Clear the queue
        PrivateData->OutgoingPackets.Empty();
    }
}

void USpacetimeDBNetDriver::ProcessRemoteFunction(class AActor* Actor, class UFunction* Function, void* Parameters, struct FOutParmRec* OutParms, struct FFrame* Stack, class UObject* SubObject)
{
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: ProcessRemoteFunction %s.%s"), *Actor->GetName(), *Function->GetName());
    
    // Call parent implementation which will route the RPC through LowLevelSend
    Super::ProcessRemoteFunction(Actor, Function, Parameters, OutParms, Stack, SubObject);
}

void USpacetimeDBNetDriver::LowLevelSend(TSharedPtr<const FInternetAddr, ESPMode::ThreadSafe> Address, void* Data, int32 CountBits, FOutPacketTraits& Traits)
{
    // Assuming Address is not directly used in this SpacetimeDB implementation for sending,
    // but logging it might still be useful.
    FString AddressString = Address.IsValid() ? Address->ToString(true) : TEXT("InvalidAddress");
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: LowLevelSend to %s, %d bits"), *AddressString, CountBits);
    
    // Get private implementation data correctly
    FSpacetimeDBNetDriverPrivate* PrivateData = this->NetDriverPrivate;
    
    // Convert bits to bytes (rounding up)
    int32 NumBytes = (CountBits + 7) >> 3;
    
    // Get access to the binary data
    uint8* ByteData = static_cast<uint8*>(Data);
    
    // Copy data to a new buffer
    TArray<uint8> PacketData;
    PacketData.Append(ByteData, NumBytes);
    
    // Queue for sending in TickFlush
    // Here we're using "network_packet" as the reducer name
    // In a real implementation, there might be different reducers for different packet types
    PrivateData->OutgoingPackets.Add(TPair<FString, TArray<uint8>>(TEXT("network_packet"), PacketData));
}

void USpacetimeDBNetDriver::Shutdown()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Shutdown"));
    
    // Disconnect from SpacetimeDB
    Client.Disconnect();
    
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
    
    // Clean up connections
    if (ServerConnection)
    {
        ServerConnection->Close();
        ServerConnection = nullptr;
    }
    
    // Clean up client connections
    for (int32 i = ClientConnections.Num() - 1; i >= 0; i--)
    {
        if (ClientConnections[i])
        {
            ClientConnections[i]->Close();
        }
    }
    ClientConnections.Empty();
    
    // Clean up private implementation
    if (NetDriverPrivate)
    {
        delete static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
        NetDriverPrivate = nullptr;
    }
    
    // Call parent implementation
    Super::Shutdown();
}

bool USpacetimeDBNetDriver::IsNetResourceValid()
{
    // A SpacetimeDB connection is valid if it's been initialized and is connected
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    return PrivateData && PrivateData->bInitialized && Client.IsConnected();
}

// --- Event handlers ---

void USpacetimeDBNetDriver::HandleConnected()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Connected to SpacetimeDB"));
    
    // Subscribe to relevant tables
    // For a real implementation, you'd subscribe to all tables needed for replication
    SubscribedTables.Add(TEXT("actors"));
    SubscribedTables.Add(TEXT("network_packets"));
    
    if (SubscribedTables.Num() > 0)
    {
        Client.SubscribeToTables(SubscribedTables);
    }
}

void USpacetimeDBNetDriver::HandleDisconnected(const FString& Reason)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Disconnected from SpacetimeDB: %s"), *Reason);
    
    // Clear subscriptions
    SubscribedTables.Empty();
    
    // Notify the game code that we've been disconnected
    if (ServerConnection)
    {
        ServerConnection->SetConnectionState(EConnectionState::USOCK_Closed);
    }
    
    // TODO: Handle reconnection logic if desired
}

void USpacetimeDBNetDriver::HandleIdentityReceived(const FString& Identity)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetDriver: Identity received: %s"), *Identity);
    
    // Set identity on connection
    if (ServerConnection)
    {
        static_cast<USpacetimeDBNetConnection*>(ServerConnection)->SetSpacetimeIdentity(Identity);
    }
}

void USpacetimeDBNetDriver::HandleEventReceived(const FString& TableName, const FString& EventData)
{
    // This method is called when data is received from a SpacetimeDB subscription
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: Event received for table %s"), *TableName);
    
    // Process based on the table type
    if (TableName == TEXT("actors"))
    {
        ProcessTableEvent(TableName, EventData);
    }
    else if (TableName == TEXT("network_packets"))
    {
        // Process network packets - these would be actual replication data or RPCs
        // In a real implementation, you'd parse the data and deliver it to the appropriate actor
        UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: Network packet received with data: %s"), *EventData);
        
        // TODO: Implement packet delivery to actors
    }
}

void USpacetimeDBNetDriver::HandleErrorOccurred(const FSpacetimeDBErrorInfo& ErrorInfo)
{
    UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: Error - %s"), *ErrorInfo.Message);
    
    // Add additional handling as needed
    // For example, notify game code, disconnect on fatal errors, etc.
}

void USpacetimeDBNetDriver::ProcessTableEvent(const FString& TableName, const FString& EventData)
{
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetDriver: Processing event for table %s: %s"), *TableName, *EventData);
    
    // Parse the JSON data
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> JsonReader = TJsonReaderFactory<>::Create(EventData);
    
    if (!FJsonSerializer::Deserialize(JsonReader, JsonObject) || !JsonObject.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBNetDriver: Failed to parse event data as JSON: %s"), *EventData);
        return;
    }
    
    // In a real implementation, this is where you'd update actor replication state
    // based on the data from SpacetimeDB
    
    // Get private implementation data
    FSpacetimeDBNetDriverPrivate* PrivateData = static_cast<FSpacetimeDBNetDriverPrivate*>(NetDriverPrivate);
    
    // Example for handling actor table events (not a complete implementation)
    if (TableName == TEXT("actors"))
    {
        // Try to get actor ID from the event data
        FString ActorId;
        if (JsonObject->TryGetStringField(TEXT("id"), ActorId))
        {
            // Check if we're already tracking this actor
            if (!PrivateData->ActorReplicationData.Contains(ActorId))
            {
                // New actor, store its data
                FSpacetimeDBReplicationData ReplicationData;
                ReplicationData.ActorId = ActorId;
                
                // Try to get actor class from the event data
                JsonObject->TryGetStringField(TEXT("class"), ReplicationData.ActorClass);
                
                // Store properties
                ReplicationData.Properties = JsonObject;
                
                // Add to our map
                PrivateData->ActorReplicationData.Add(ActorId, ReplicationData);
                
                // TODO: Spawn actor if appropriate
            }
            else
            {
                // Existing actor, update properties
                FSpacetimeDBReplicationData& ReplicationData = PrivateData->ActorReplicationData[ActorId];
                ReplicationData.Properties = JsonObject;
                
                // TODO: Update actor properties if appropriate
            }
        }
    }
} 