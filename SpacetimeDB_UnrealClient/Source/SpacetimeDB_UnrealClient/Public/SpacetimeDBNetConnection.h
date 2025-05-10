// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Engine/NetConnection.h"
#include "SpacetimeDBNetConnection.generated.h"

/**
 * @class USpacetimeDBNetConnection
 * @brief Custom NetConnection for integration with SpacetimeDB.
 * 
 * This NetConnection represents a connection to the SpacetimeDB server.
 */
UCLASS(transient, config=Engine)
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBNetConnection : public UNetConnection
{
    GENERATED_BODY()
    
public:
    USpacetimeDBNetConnection(const FObjectInitializer& ObjectInitializer);
    
    //~ Begin UNetConnection Interface
    virtual void InitBase(UNetDriver* InDriver, class FSocket* InSocket, const FURL& InURL, EConnectionState InState, int32 InMaxPacket = 0, int32 InPacketOverhead = 0) override;
    virtual void InitLocalConnection(UNetDriver* InDriver, class FSocket* InSocket, const FURL& InURL, EConnectionState InState, int32 InMaxPacket = 0, int32 InPacketOverhead = 0) override;
    virtual void InitRemoteConnection(UNetDriver* InDriver, class FSocket* InSocket, const FURL& InURL, const FInternetAddr& InRemoteAddr, EConnectionState InState, int32 InMaxPacket = 0, int32 InPacketOverhead = 0) override;
    virtual void LowLevelSend(void* Data, int32 CountBits, FOutPacketTraits& Traits) override;
    virtual FString LowLevelGetRemoteAddress(bool bAppendPort = false) override;
    virtual FString LowLevelDescribe() override;
    virtual int32 GetAddrAsInt();
    virtual int32 GetAddrPort() override;
    virtual FString RemoteAddressToString() override;
    //~ End UNetConnection Interface
    
    /** Sets the SpacetimeDB identity for this connection */
    void SetSpacetimeIdentity(const FString& Identity);
    
    /** Gets the SpacetimeDB identity for this connection */
    const FString& GetSpacetimeIdentity() const { return SpacetimeIdentity; }
    
private:
    /** The SpacetimeDB identity for this connection */
    FString SpacetimeIdentity;
    
    /** The remote address for this connection */
    FString RemoteAddress;
}; 