// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Engine/NetDriver.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDBNetDriver.generated.h"

/**
 * @class USpacetimeDBNetDriver
 * @brief Custom NetDriver for integration with SpacetimeDB.
 * 
 * This NetDriver implements Unreal Engine's networking APIs to use SpacetimeDB
 * as the backend for replicating actors and RPCs.
 */
UCLASS(transient, config=Engine)
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBNetDriver : public UNetDriver
{
    GENERATED_BODY()
    
public:
    USpacetimeDBNetDriver(const FObjectInitializer& ObjectInitializer);
    
    //~ Begin UNetDriver Interface
    virtual bool IsAvailable() const override;
    virtual bool InitBase(bool bInitAsClient, FNetworkNotify* InNotify, const FURL& URL, bool bReuseAddressAndPort, FString& Error) override;
    virtual bool InitConnect(FNetworkNotify* InNotify, const FURL& ConnectURL, FString& Error) override;
    virtual bool InitListen(FNetworkNotify* InNotify, FURL& LocalURL, bool bReuseAddressAndPort, FString& Error) override;
    virtual void TickDispatch(float DeltaTime) override;
    virtual void TickFlush(float DeltaTime) override;
    virtual void ProcessRemoteFunction(class AActor* Actor, class UFunction* Function, void* Parameters, struct FOutParmRec* OutParms, struct FFrame* Stack, class UObject* SubObject = NULL) override;
    virtual void LowLevelSend(FString Address, void* Data, int32 CountBits, FOutPacketTraits& Traits) override;
    virtual void Shutdown() override;
    virtual bool IsNetResourceValid() override;
    //~ End UNetDriver Interface

private:
    /** The SpacetimeDB client instance used for network communication */
    FSpacetimeDBClient Client;
    
    /** Flag to indicate if this driver is acting as a server (listen) or client (connect) */
    bool bIsServer;
    
    /** Names of SpacetimeDB tables we're subscribed to */
    TArray<FString> SubscribedTables;
    
    /** Handles for client events */
    FDelegateHandle OnConnectedHandle;
    FDelegateHandle OnDisconnectedHandle;
    FDelegateHandle OnIdentityReceivedHandle;
    FDelegateHandle OnEventReceivedHandle;
    FDelegateHandle OnErrorOccurredHandle;
    
    // Network connection to the SpacetimeDB server
    TObjectPtr<class USpacetimeDBNetConnection> ServerConnection;
    
    // Internal methods
    void HandleConnected();
    void HandleDisconnected(const FString& Reason);
    void HandleIdentityReceived(const FString& Identity);
    void HandleEventReceived(const FString& TableName, const FString& EventData);
    void HandleErrorOccurred(const FString& ErrorMessage);
    
    // Process table events from SpacetimeDB and update replication state
    void ProcessTableEvent(const FString& TableName, const FString& EventData);
}; 