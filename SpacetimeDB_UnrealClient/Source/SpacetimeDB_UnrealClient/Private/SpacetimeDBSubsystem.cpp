// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDBSubsystem.h"
#include "Engine/Engine.h"
#include "Async/Async.h"
#include "SpacetimeDBFFI.h"
#include "SpacetimeDBClient.h"
#include "SpacetimeDB_PropertyValue.h"
#include "Kismet/GameplayStatics.h"
#include "Engine/World.h"
#include "Dom/JsonObject.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"

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

// Property update callback
void USpacetimeDBSubsystem::OnPropertyUpdated(int64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Queue this for processing on the game thread
    AsyncTask(ENamedThreads::GameThread, [this, ObjectId, PropertyName, ValueJson]()
    {
        // Process the property update
        if (USpacetimeDBPropertyHandler::HandlePropertyUpdate(ObjectId, PropertyName, ValueJson))
        {
            // Broadcast the property update event
            FSpacetimeDBPropertyUpdateInfo UpdateInfo;
            UpdateInfo.ObjectId = FSpacetimeDBObjectID(ObjectId);
            UpdateInfo.PropertyName = PropertyName;
            UpdateInfo.RawJsonValue = ValueJson;
            
            // Parse the property value for the delegate
            UpdateInfo.PropertyValue = FSpacetimeDBPropertyValue::FromJsonString(ValueJson);

            // Find the target object
            USpacetimeDBClient* Client = USpacetimeDBClient::Get();
            if (Client)
            {
                UpdateInfo.Object = Client->GetObjectById(UpdateInfo.ObjectId);
            }
            
            // Broadcast the property update event
            OnPropertyUpdated.Broadcast(UpdateInfo);
        }
        else
        {
            UE_LOG(LogTemp, Error, TEXT("Failed to handle property update for object %lld, property %s"),
                ObjectId, *PropertyName);
        }
    });
}

// FFI callback handlers for property updates
void OnPropertyUpdatedCallback(uint64 object_id, cxx::String property_name, cxx::String value_json)
{
    USpacetimeDBSubsystem* Subsystem = GEngine->GetEngineSubsystem<USpacetimeDBSubsystem>();
    if (Subsystem)
    {
        Subsystem->OnPropertyUpdated(object_id, FromCxxString(property_name), FromCxxString(value_json));
    }
}

// Object Lifecycle Management

int64 USpacetimeDBSubsystem::RequestSpawnObject(const FSpacetimeDBSpawnParams& Params)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: RequestSpawnObject called while not connected"));
        return 0;
    }

    // Create JSON for initial properties
    TSharedPtr<FJsonObject> SpawnParamsJson = MakeShared<FJsonObject>();
    
    // Add standard properties
    SpawnParamsJson->SetStringField("class_name", Params.ClassName);
    SpawnParamsJson->SetBoolField("replicate", Params.bReplicate);
    
    // Add transform for actors
    TSharedPtr<FJsonObject> TransformJson = MakeShared<FJsonObject>();
    
    // Location
    TSharedPtr<FJsonObject> LocationJson = MakeShared<FJsonObject>();
    LocationJson->SetNumberField("x", Params.Transform.GetLocation().X);
    LocationJson->SetNumberField("y", Params.Transform.GetLocation().Y);
    LocationJson->SetNumberField("z", Params.Transform.GetLocation().Z);
    TransformJson->SetObjectField("location", LocationJson);
    
    // Rotation
    FRotator Rotator = Params.Transform.Rotator();
    TSharedPtr<FJsonObject> RotationJson = MakeShared<FJsonObject>();
    RotationJson->SetNumberField("pitch", Rotator.Pitch);
    RotationJson->SetNumberField("yaw", Rotator.Yaw);
    RotationJson->SetNumberField("roll", Rotator.Roll);
    TransformJson->SetObjectField("rotation", RotationJson);
    
    // Scale
    TSharedPtr<FJsonObject> ScaleJson = MakeShared<FJsonObject>();
    ScaleJson->SetNumberField("x", Params.Transform.GetScale3D().X);
    ScaleJson->SetNumberField("y", Params.Transform.GetScale3D().Y);
    ScaleJson->SetNumberField("z", Params.Transform.GetScale3D().Z);
    TransformJson->SetObjectField("scale", ScaleJson);
    
    SpawnParamsJson->SetObjectField("transform", TransformJson);
    
    // Add custom properties JSON if provided
    if (!Params.PropertiesJson.IsEmpty())
    {
        // Parse the provided JSON string into an object
        TSharedPtr<FJsonObject> CustomPropertiesJson;
        TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Params.PropertiesJson);
        if (FJsonSerializer::Deserialize(Reader, CustomPropertiesJson))
        {
            // Add all fields from the custom properties to the main JSON
            SpawnParamsJson->SetObjectField("properties", CustomPropertiesJson);
        }
        else
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to parse PropertiesJson: %s"), *Params.PropertiesJson);
        }
    }
    
    // Serialize to JSON string
    FString ParamsJsonStr;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ParamsJsonStr);
    FJsonSerializer::Serialize(SpawnParamsJson.ToSharedRef(), Writer);
    
    // Call the reducer
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Requesting spawn of %s with params: %s"), *Params.ClassName, *ParamsJsonStr);
    if (Client.CallReducer("create_object", ParamsJsonStr))
    {
        // This will return a temporary ID that gets updated when the server confirms
        // For now, return 0 as a placeholder - the object will be created when the server confirms
        return 0;
    }
    
    return 0;
}

bool USpacetimeDBSubsystem::RequestDestroyObject(int64 ObjectId)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: RequestDestroyObject called while not connected"));
        return false;
    }

    // Check if the object exists locally
    if (!ObjectRegistry.Contains(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestDestroyObject - Object with ID %lld not found locally"), ObjectId);
        // We'll still try to send the request to the server
    }
    
    // Create JSON for the destroy request
    TSharedPtr<FJsonObject> DestroyParamsJson = MakeShared<FJsonObject>();
    DestroyParamsJson->SetNumberField("object_id", static_cast<double>(ObjectId));
    
    // Serialize to JSON string
    FString ParamsJsonStr;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ParamsJsonStr);
    FJsonSerializer::Serialize(DestroyParamsJson.ToSharedRef(), Writer);
    
    // Call the reducer
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Requesting destruction of object with ID %lld"), ObjectId);
    return Client.CallReducer("destroy_object", ParamsJsonStr);
}

UObject* USpacetimeDBSubsystem::FindObjectById(int64 ObjectId) const
{
    const UObject* const* FoundObject = ObjectRegistry.Find(ObjectId);
    return FoundObject ? const_cast<UObject*>(*FoundObject) : nullptr;
}

int64 USpacetimeDBSubsystem::FindObjectId(UObject* Object) const
{
    const int64* FoundId = ObjectToIdMap.Find(Object);
    return FoundId ? *FoundId : 0;
}

TArray<UObject*> USpacetimeDBSubsystem::GetAllObjects() const
{
    TArray<UObject*> AllObjects;
    ObjectRegistry.GenerateValueArray(AllObjects);
    return AllObjects;
}

UObject* USpacetimeDBSubsystem::SpawnObjectFromServer(int64 ObjectId, const FString& ClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: SpawnObjectFromServer - ID: %lld, Class: %s"), ObjectId, *ClassName);
    
    // First check if this object is already registered (could be a remap)
    if (UObject* ExistingObject = FindObjectById(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Object with ID %lld already exists"), ObjectId);
        return ExistingObject;
    }
    
    // Parse JSON data
    TSharedPtr<FJsonObject> ObjectDataJson;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(DataJson);
    if (!FJsonSerializer::Deserialize(Reader, ObjectDataJson))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to parse object data JSON: %s"), *DataJson);
        return nullptr;
    }
    
    // Determine if this is an actor or a plain UObject
    bool bIsActor = false;
    FTransform SpawnTransform = FTransform::Identity;
    
    // Extract transform if available (for actors)
    TSharedPtr<FJsonObject> TransformObj = ObjectDataJson->GetObjectField("transform");
    if (TransformObj.IsValid())
    {
        bIsActor = true;
        
        // Extract location
        TSharedPtr<FJsonObject> LocationObj = TransformObj->GetObjectField("location");
        if (LocationObj.IsValid())
        {
            FVector Location;
            Location.X = LocationObj->GetNumberField("x");
            Location.Y = LocationObj->GetNumberField("y");
            Location.Z = LocationObj->GetNumberField("z");
            SpawnTransform.SetLocation(Location);
        }
        
        // Extract rotation
        TSharedPtr<FJsonObject> RotationObj = TransformObj->GetObjectField("rotation");
        if (RotationObj.IsValid())
        {
            FRotator Rotation;
            Rotation.Pitch = RotationObj->GetNumberField("pitch");
            Rotation.Yaw = RotationObj->GetNumberField("yaw");
            Rotation.Roll = RotationObj->GetNumberField("roll");
            SpawnTransform.SetRotation(Rotation.Quaternion());
        }
        
        // Extract scale
        TSharedPtr<FJsonObject> ScaleObj = TransformObj->GetObjectField("scale");
        if (ScaleObj.IsValid())
        {
            FVector Scale;
            Scale.X = ScaleObj->GetNumberField("x");
            Scale.Y = ScaleObj->GetNumberField("y");
            Scale.Z = ScaleObj->GetNumberField("z");
            SpawnTransform.SetScale3D(Scale);
        }
    }
    
    UObject* SpawnedObject = nullptr;
    
    // Find the class by name
    UClass* ObjectClass = FindObject<UClass>(ANY_PACKAGE, *ClassName);
    if (!ObjectClass)
    {
        // Try with a U prefix if not found
        if (!ClassName.StartsWith(TEXT("U")) && !ClassName.StartsWith(TEXT("A")))
        {
            FString PrefixedClassName = FString::Printf(TEXT("U%s"), *ClassName);
            ObjectClass = FindObject<UClass>(ANY_PACKAGE, *PrefixedClassName);
            
            // Also try with A prefix for actors
            if (!ObjectClass)
            {
                PrefixedClassName = FString::Printf(TEXT("A%s"), *ClassName);
                ObjectClass = FindObject<UClass>(ANY_PACKAGE, *PrefixedClassName);
            }
        }
        
        if (!ObjectClass)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Could not find class '%s'"), *ClassName);
            return nullptr;
        }
    }
    
    // Check if this is an actor class
    bIsActor = ObjectClass->IsChildOf(AActor::StaticClass());
    
    if (bIsActor)
    {
        // Spawn the actor
        UWorld* World = GetWorld();
        if (!World)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: No world available for spawning actor"));
            return nullptr;
        }
        
        // Use deferred spawning to allow setting properties before the actor initializes
        FActorSpawnParameters SpawnParams;
        SpawnParams.SpawnCollisionHandlingOverride = ESpawnActorCollisionHandlingMethod::AlwaysSpawn;
        SpawnParams.bDeferConstruction = true;  // Important for setting properties before BeginPlay
        
        AActor* SpawnedActor = World->SpawnActorDeferred<AActor>(ObjectClass, SpawnTransform, nullptr, nullptr, ESpawnActorCollisionHandlingMethod::AlwaysSpawn);
        if (!SpawnedActor)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to spawn actor of class '%s'"), *ClassName);
            return nullptr;
        }
        
        // Apply initial properties from JSON if provided
        TSharedPtr<FJsonObject> PropertiesObj = ObjectDataJson->GetObjectField("properties");
        if (PropertiesObj.IsValid())
        {
            // Apply each property from JSON to the actor
            for (const auto& Pair : PropertiesObj->Values)
            {
                const FString& PropertyName = Pair.Key;
                const TSharedPtr<FJsonValue>& JsonValue = Pair.Value;
                
                // TODO: Implement a more complete property application system
                // This is a basic implementation and should be expanded
                
                // Find the property on the actor
                UProperty* Property = FindField<UProperty>(SpawnedActor->GetClass(), *PropertyName);
                if (!Property)
                {
                    UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Property '%s' not found on actor of class '%s'"), *PropertyName, *ClassName);
                    continue;
                }
                
                // Set the property value based on its type
                if (UBoolProperty* BoolProperty = Cast<UBoolProperty>(Property))
                {
                    BoolProperty->SetPropertyValue_InContainer(SpawnedActor, JsonValue->AsBool());
                }
                else if (UIntProperty* IntProperty = Cast<UIntProperty>(Property))
                {
                    IntProperty->SetPropertyValue_InContainer(SpawnedActor, (int32)JsonValue->AsNumber());
                }
                else if (UFloatProperty* FloatProperty = Cast<UFloatProperty>(Property))
                {
                    FloatProperty->SetPropertyValue_InContainer(SpawnedActor, JsonValue->AsNumber());
                }
                else if (UStrProperty* StrProperty = Cast<UStrProperty>(Property))
                {
                    StrProperty->SetPropertyValue_InContainer(SpawnedActor, JsonValue->AsString());
                }
                // Additional property types can be handled here
            }
        }
        
        // Finalize actor spawning
        UGameplayStatics::FinishSpawningActor(SpawnedActor, SpawnTransform);
        
        SpawnedObject = SpawnedActor;
    }
    else
    {
        // Create a regular UObject
        SpawnedObject = NewObject<UObject>(GetTransientPackage(), ObjectClass);
        if (!SpawnedObject)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to create UObject of class '%s'"), *ClassName);
            return nullptr;
        }
        
        // Apply initial properties from JSON if provided
        TSharedPtr<FJsonObject> PropertiesObj = ObjectDataJson->GetObjectField("properties");
        if (PropertiesObj.IsValid())
        {
            // TODO: Apply properties to the UObject
            // Similar to the actor code above, but for UObject
        }
    }
    
    // Register the object in our registry
    if (SpawnedObject)
    {
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Successfully spawned/created object of class '%s' with ID %lld"), *ClassName, ObjectId);
        
        // Add to registry
        ObjectRegistry.Add(ObjectId, SpawnedObject);
        ObjectToIdMap.Add(SpawnedObject, ObjectId);
        
        // Add a destroy delegate to clean up registry when the object is destroyed
        if (AActor* Actor = Cast<AActor>(SpawnedObject))
        {
            Actor->OnDestroyed.AddDynamic(this, &USpacetimeDBSubsystem::OnActorDestroyed);
        }
    }
    
    return SpawnedObject;
}

void USpacetimeDBSubsystem::DestroyObjectFromServer(int64 ObjectId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: DestroyObjectFromServer - ID: %lld"), ObjectId);
    
    // Find the object in our registry
    UObject* Object = FindObjectById(ObjectId);
    if (!Object)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Object with ID %lld not found in registry, can't destroy"), ObjectId);
        return;
    }
    
    // Remove from registry first
    ObjectRegistry.Remove(ObjectId);
    ObjectToIdMap.Remove(Object);
    
    // Destroy the object
    if (AActor* Actor = Cast<AActor>(Object))
    {
        // It's an actor, use the proper destroy method
        Actor->Destroy();
    }
    else
    {
        // It's a regular UObject, mark it as pending kill
        Object->MarkPendingKill();
    }
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Successfully removed object with ID %lld from registry"), ObjectId);
}

// Event handler for when an actor is destroyed outside of our control
UFUNCTION()
void USpacetimeDBSubsystem::OnActorDestroyed(AActor* DestroyedActor)
{
    if (!DestroyedActor)
        return;
    
    // Find this actor in our registry and remove it
    int64 ObjectId = FindObjectId(DestroyedActor);
    if (ObjectId != 0)
    {
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Actor %s was destroyed, removing from registry (ID: %lld)"), *DestroyedActor->GetName(), ObjectId);
        ObjectRegistry.Remove(ObjectId);
        ObjectToIdMap.Remove(DestroyedActor);
    }
}

// Update existing handler methods

void USpacetimeDBSubsystem::HandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object created - ID %llu, Class %s"), ObjectId, *ClassName);
    
    // Broadcast this event to BP/C++ delegates
    OnObjectCreated.Broadcast(static_cast<int64>(ObjectId), ClassName, DataJson);
    
    // Create the actual object in the game
    UObject* NewObject = SpawnObjectFromServer(static_cast<int64>(ObjectId), ClassName, DataJson);
    if (!NewObject)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to spawn object from server"));
    }
}

void USpacetimeDBSubsystem::HandleObjectDestroyed(uint64 ObjectId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object destroyed - ID %llu"), ObjectId);
    
    // Broadcast this event to BP/C++ delegates
    OnObjectDestroyed.Broadcast(static_cast<int64>(ObjectId));
    
    // Destroy the actual object in the game
    DestroyObjectFromServer(static_cast<int64>(ObjectId));
}

void USpacetimeDBSubsystem::HandleObjectIdRemapped(uint64 TempId, uint64 ServerId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object ID remapped - Temp ID %llu -> Server ID %llu"), TempId, ServerId);
    
    // Broadcast this event to BP/C++ delegates
    OnObjectIdRemapped.Broadcast(static_cast<int64>(TempId), static_cast<int64>(ServerId));
    
    // Update our object registry
    UObject* Object = FindObjectById(static_cast<int64>(TempId));
    if (Object)
    {
        // Update the registry with the new ID
        ObjectRegistry.Remove(static_cast<int64>(TempId));
        ObjectRegistry.Add(static_cast<int64>(ServerId), Object);
        
        // Update the reverse lookup
        ObjectToIdMap.Remove(Object);
        ObjectToIdMap.Add(Object, static_cast<int64>(ServerId));
        
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Updated registry for object ID remap: %llu -> %llu"), TempId, ServerId);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Could not find object with temp ID %llu for remapping"), TempId);
    }
}

void USpacetimeDBSubsystem::HandlePropertyUpdated(uint64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Use Verbose log level since this could be high frequency
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Property updated - Object %llu, Property %s"), ObjectId, *PropertyName);
    
    // Find the object in our registry
    UObject* Object = FindObjectById(static_cast<int64>(ObjectId));
    
    // Prepare update info to broadcast
    FSpacetimeDBPropertyUpdateInfo UpdateInfo;
    UpdateInfo.ObjectId = static_cast<int64>(ObjectId);
    UpdateInfo.Object = Object;
    UpdateInfo.PropertyName = PropertyName;
    UpdateInfo.RawJsonValue = ValueJson;
    
    // TODO: Parse the JSON value into a proper FSpacetimeDBPropertyValue
    // This would involve a JSON parsing system
    
    // Broadcast the update
    OnPropertyUpdated.Broadcast(UpdateInfo);
    
    if (Object)
    {
        // Apply the property to the object
        // TODO: Implement property application
        // This should parse the JSON and apply it to the actual UObject property
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Cannot apply property %s - Object with ID %llu not found"), *PropertyName, ObjectId);
    }
} 