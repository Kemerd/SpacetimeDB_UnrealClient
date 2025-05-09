// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBNetConnection.h"
#include "SpacetimeDBNetDriver.h"

USpacetimeDBNetConnection::USpacetimeDBNetConnection(const FObjectInitializer& ObjectInitializer)
    : Super(ObjectInitializer)
{
    // Set default values
    RemoteAddress = TEXT("spacetimedb://unknown");
    SpacetimeIdentity = TEXT("");
    
    // Initialize as a client connection
    SetConnectionState(EConnectionState::USOCK_Open);
    Driver = nullptr;
    
    // Set initial packet handling values
    MaxPacket = 1024;
    PacketOverhead = 0;
    InternalAck = true;
}

void USpacetimeDBNetConnection::InitBase(UNetDriver* InDriver, FSocket* InSocket, const FURL& InURL, EConnectionState InState, int32 InMaxPacket, int32 InPacketOverhead)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetConnection: InitBase URL=%s"), *InURL.ToString());
    
    // Call parent implementation
    Super::InitBase(InDriver, InSocket, InURL, InState, InMaxPacket, InPacketOverhead);
    
    // Store remote address from URL
    RemoteAddress = FString::Printf(TEXT("spacetimedb://%s/%s"), *InURL.Host, *InURL.Map);
}

void USpacetimeDBNetConnection::InitLocalConnection(UNetDriver* InDriver, FSocket* InSocket, const FURL& InURL, EConnectionState InState, int32 InMaxPacket, int32 InPacketOverhead)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetConnection: InitLocalConnection URL=%s"), *InURL.ToString());
    
    // Call parent implementation
    Super::InitLocalConnection(InDriver, InSocket, InURL, InState, InMaxPacket, InPacketOverhead);
    
    // Set the address and initialize
    RemoteAddress = FString::Printf(TEXT("spacetimedb://%s/%s"), *InURL.Host, *InURL.Map);
    SetConnectionState(EConnectionState::USOCK_Open);
}

void USpacetimeDBNetConnection::InitRemoteConnection(UNetDriver* InDriver, FSocket* InSocket, const FURL& InURL, const FInternetAddr& InRemoteAddr, EConnectionState InState, int32 InMaxPacket, int32 InPacketOverhead)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetConnection: InitRemoteConnection URL=%s"), *InURL.ToString());
    
    // Call parent implementation
    Super::InitRemoteConnection(InDriver, InSocket, InURL, InRemoteAddr, InState, InMaxPacket, InPacketOverhead);
    
    // Set the address and initialize
    RemoteAddress = FString::Printf(TEXT("spacetimedb://%s/%s"), *InURL.Host, *InURL.Map);
    SetConnectionState(EConnectionState::USOCK_Open);
}

void USpacetimeDBNetConnection::LowLevelSend(void* Data, int32 CountBits, FOutPacketTraits& Traits)
{
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBNetConnection: LowLevelSend %d bits"), CountBits);
    
    // In a real implementation, this would transmit data via the SpacetimeDB client, 
    // possibly calling a reducer function with the data as a parameter.
    // For now, we'll just log the call.
    
    // TODO: Map Unreal's packet data to SpacetimeDB reducers for RPC and replication support
}

FString USpacetimeDBNetConnection::LowLevelGetRemoteAddress(bool bAppendPort)
{
    return RemoteAddress;
}

FString USpacetimeDBNetConnection::LowLevelDescribe()
{
    return FString::Printf(TEXT("SpacetimeDBNetConnection to %s [Identity: %s]"), 
        *RemoteAddress, 
        SpacetimeIdentity.IsEmpty() ? TEXT("Unknown") : *SpacetimeIdentity);
}

int32 USpacetimeDBNetConnection::GetAddrAsInt()
{
    // For SpacetimeDB connections, return a hash of the identity if available, 
    // or a hash of the remote address if not
    FString AddrStr = SpacetimeIdentity.IsEmpty() ? RemoteAddress : SpacetimeIdentity;
    return GetTypeHash(AddrStr);
}

int32 USpacetimeDBNetConnection::GetAddrPort()
{
    // SpacetimeDB doesn't use traditional ports, so return a placeholder
    return 42069;
}

FString USpacetimeDBNetConnection::RemoteAddressToString()
{
    return RemoteAddress;
}

void USpacetimeDBNetConnection::SetSpacetimeIdentity(const FString& Identity)
{
    SpacetimeIdentity = Identity;
    
    // In a real-world implementation, this would be a good place to update
    // connection-specific data in SpacetimeDB tables, such as player presence
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBNetConnection: Identity set to %s"), *Identity);
} 