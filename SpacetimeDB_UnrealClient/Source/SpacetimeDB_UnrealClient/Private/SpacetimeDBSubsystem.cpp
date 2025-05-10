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
#include "SpacetimeDBPropertyHelper.h"
#include "SpacetimeDB_JsonUtils.h"
#include "SpacetimeDBPredictionComponent.h"

// Static map to store subsystem instances by world context
static TMap<const UObject*, USpacetimeDBSubsystem*> GSubsystemInstances;

void USpacetimeDBSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
    Super::Initialize(Collection);
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Initializing"));
    
    // Store reference to this instance by GetWorld()
    GSubsystemInstances.Add(GetGameInstance(), this);
    
    // Register for client events
    OnConnectedHandle = Client.OnConnected.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleConnected);
    OnDisconnectedHandle = Client.OnDisconnected.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleDisconnected);
    OnIdentityReceivedHandle = Client.OnIdentityReceived.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleIdentityReceived);
    OnEventReceivedHandle = Client.OnEventReceived.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleEventReceived);
    OnErrorOccurredHandle = Client.OnErrorOccurred.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleErrorOccurred);
    
    // Register for object system events
    OnPropertyUpdatedHandle = Client.OnPropertyUpdated.AddUObject(this, &USpacetimeDBSubsystem::InternalHandlePropertyUpdated);
    OnObjectCreatedHandle = Client.OnObjectCreated.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleObjectCreated);
    OnObjectDestroyedHandle = Client.OnObjectDestroyed.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleObjectDestroyed);
    OnObjectIdRemappedHandle = Client.OnObjectIdRemapped.AddUObject(this, &USpacetimeDBSubsystem::InternalHandleObjectIdRemapped);
}

void USpacetimeDBSubsystem::Deinitialize()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Deinitializing"));
    
    // Remove this instance from the global map
    GSubsystemInstances.Remove(GetGameInstance());
    
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

void USpacetimeDBSubsystem::InternalHandleConnected()
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Connected event received"));
    OnConnected.Broadcast();
    
    // Optional: Display a notification in game if desired
    if (GEngine)
    {
        GEngine->AddOnScreenDebugMessage(-1, 5.0f, FColor::Green, TEXT("Connected to SpacetimeDB"));
    }
}

void USpacetimeDBSubsystem::InternalHandleDisconnected(const FString& Reason)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Disconnected event received: %s"), *Reason);
    OnDisconnected.Broadcast(Reason);
    
    // Optional: Display a notification in game if desired
    if (GEngine)
    {
        GEngine->AddOnScreenDebugMessage(-1, 5.0f, FColor::Red, FString::Printf(TEXT("Disconnected from SpacetimeDB: %s"), *Reason));
    }
}

void USpacetimeDBSubsystem::InternalHandleIdentityReceived(const FString& Identity)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Identity received: %s"), *Identity);
    OnIdentityReceived.Broadcast(Identity);
}

void USpacetimeDBSubsystem::InternalHandleEventReceived(const FString& TableName, const FString& EventData)
{
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Event received for table %s: %s"), *TableName, *EventData);
    OnEventReceived.Broadcast(TableName, EventData);
}

void USpacetimeDBSubsystem::InternalHandleErrorOccurred(const FSpacetimeDBErrorInfo& ErrorInfo)
{
    UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Error occurred: %s"), *ErrorInfo.Message);
    OnErrorOccurred.Broadcast(ErrorInfo);
    
    // Optional: Display a notification in game for critical errors
    if (GEngine && ErrorInfo.Severity >= ESpacetimeDBErrorSeverity::Critical)
    {
        GEngine->AddOnScreenDebugMessage(-1, 10.0f, FColor::Red, FString::Printf(TEXT("SpacetimeDB Error: %s"), *ErrorInfo.Message));
    }
}

// Object system event handlers

void USpacetimeDBSubsystem::HandlePropertyUpdated(uint64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Delegate to the main property update handler
    InternalHandlePropertyUpdated(ObjectId, PropertyName, ValueJson);
}

void USpacetimeDBSubsystem::InternalHandlePropertyUpdated(uint64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Delegate to the main property update handler
    InternalOnPropertyUpdated(static_cast<int64>(ObjectId), PropertyName, ValueJson);
}

void USpacetimeDBSubsystem::HandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object created event - ID: %llu, Class: %s"), ObjectId, *ClassName);
    
    // Create the object
    UObject* NewObject = SpawnObjectFromServer(ObjectId, ClassName, DataJson);
    
    if (NewObject)
    {
        // Broadcast the object created event
        OnObjectCreated.Broadcast(ObjectId, ClassName, DataJson);
    }
}

void USpacetimeDBSubsystem::HandleObjectDestroyed(uint64 ObjectId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object destroyed event - ID: %llu"), ObjectId);
    
    // Broadcast the object destroyed event before actually destroying it
    OnObjectDestroyed.Broadcast(ObjectId);
    
    // Destroy the object
    DestroyObjectFromServer(ObjectId);
}

void USpacetimeDBSubsystem::HandleObjectIdRemapped(uint64 TempId, uint64 ServerId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object ID remapped - Temp ID: %llu, Server ID: %llu"), TempId, ServerId);
    
    // Broadcast the object ID remapped event
    OnObjectIdRemapped.Broadcast(TempId, ServerId);
    
    // TODO: Implement ID remapping in the object registry
}

void USpacetimeDBSubsystem::InternalOnPropertyUpdated(int64 ObjectId, const FString& PropertyName, const FString& ValueJson)
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
    
    // Parse the JSON value into a proper FSpacetimeDBPropertyValue
    // This would use the existing PropertyValue parsing system
    UpdateInfo.PropertyValue = FSpacetimeDBPropertyValue::FromJsonString(ValueJson);
    
    if (Object)
    {
        // Apply the property to the object using our property helper
        bool bSuccess = FSpacetimeDBPropertyHelper::ApplyJsonToProperty(Object, PropertyName, ValueJson);
        
        if (bSuccess)
        {
            UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Successfully applied property %s to object %s (ID: %llu)"), 
                *PropertyName, *Object->GetName(), ObjectId);
        }
        else
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to apply property %s to object %s (ID: %llu)"), 
                *PropertyName, *Object->GetName(), ObjectId);
        }
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Cannot apply property %s - Object with ID %llu not found"), *PropertyName, ObjectId);
    }
    
    // Broadcast the update whether we successfully applied it or not
    OnPropertyUpdated.Broadcast(UpdateInfo);
}

// FFI callback handlers for property updates
void OnPropertyUpdatedCallback(uint64 object_id, const char* property_name_cstr, const char* value_json_cstr)
{
    USpacetimeDBSubsystem* Subsystem = GEngine->GetEngineSubsystem<USpacetimeDBSubsystem>();
    if (Subsystem)
    {
        FString PropertyName = FString(UTF8_TO_TCHAR(property_name_cstr));
        FString ValueJson = FString(UTF8_TO_TCHAR(value_json_cstr));
        Subsystem->InternalHandlePropertyUpdated(object_id, PropertyName, ValueJson);
    }
}

// Object Lifecycle Management

int64 USpacetimeDBSubsystem::RequestSpawnObject(const FSpacetimeDBSpawnParams& Params)
{
    // Create a JSON object for the arguments
    TSharedPtr<FJsonObject> ArgsObject = MakeShareable(new FJsonObject());
    ArgsObject->SetStringField(TEXT("class_name"), Params.ClassName);
    ArgsObject->SetBoolField(TEXT("replicate"), Params.bReplicate);
    ArgsObject->SetNumberField(TEXT("owner_client_id"), static_cast<double>(Params.OwnerClientId)); // JSON numbers are double

    // Create Location JSON object
    TSharedPtr<FJsonObject> LocationObject = MakeShareable(new FJsonObject());
    LocationObject->SetNumberField(TEXT("x"), Params.Location.X);
    LocationObject->SetNumberField(TEXT("y"), Params.Location.Y);
    LocationObject->SetNumberField(TEXT("z"), Params.Location.Z);
    ArgsObject->SetObjectField(TEXT("location"), LocationObject);

    // Create Rotation JSON object
    TSharedPtr<FJsonObject> RotationObject = MakeShareable(new FJsonObject());
    RotationObject->SetNumberField(TEXT("pitch"), Params.Rotation.Pitch);
    RotationObject->SetNumberField(TEXT("yaw"), Params.Rotation.Yaw);
    RotationObject->SetNumberField(TEXT("roll"), Params.Rotation.Roll);
    ArgsObject->SetObjectField(TEXT("rotation"), RotationObject);

    // Create InitialProperties JSON object
    TSharedPtr<FJsonObject> PropertiesObject = MakeShareable(new FJsonObject());
    if (Params.InitialProperties.Num() > 0)
    {
        for (const auto& Pair : Params.InitialProperties)
        {
            // Assuming the FString value in InitialProperties is already a valid JSON representation of the property's value
            // If it's a raw string, number, bool, it might need to be wrapped or parsed into a FJsonValue first.
            // For simplicity, we'll try to parse it as JSON. If it fails, set as string.
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Pair.Value);
            TSharedPtr<FJsonValue> JsonPropValue;
            if (FJsonSerializer::Deserialize(Reader, JsonPropValue) && JsonPropValue.IsValid())
            {
                PropertiesObject->SetField(Pair.Key, JsonPropValue);
            }
            else
            {
                // Fallback to setting as a JSON string if parsing failed
                PropertiesObject->SetStringField(Pair.Key, Pair.Value);
                 UE_LOG(LogTemp, Warning, TEXT("RequestSpawnObject: InitialProperty '%s' for class '%s' was not valid JSON. Stored as string: %s"), *Pair.Key, *Params.ClassName, *Pair.Value);
            }
        }
    }
    ArgsObject->SetObjectField(TEXT("initial_properties"), PropertiesObject);

    // Convert the JSON object to a string
    FString ArgsJsonString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ArgsJsonString);
    FJsonSerializer::Serialize(ArgsObject.ToSharedRef(), Writer);

    // Convert properties object to string for logging
    FString PropertiesJsonString;
    TSharedRef<TJsonWriter<>> PropertiesWriter = TJsonWriterFactory<>::Create(&PropertiesJsonString);
    FJsonSerializer::Serialize(ArgsObject->GetObjectField(TEXT("initial_properties")).ToSharedRef(), PropertiesWriter);

    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: RequestSpawnObject with ClassName: %s, Location: %s, Rotation: %s, InitialProperties: %s"), 
        *Params.ClassName, *Params.Location.ToString(), *Params.Rotation.ToString(), *PropertiesJsonString);

    // Call the generic SpawnObject reducer
    bool bSuccess = CallReducer(TEXT("SpawnObject"), ArgsJsonString);
    if (!bSuccess)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to call SpawnObject reducer."));
        return 0; // Indicate failure
    }

    // Generate a temporary ID (actual ID will be remapped by server)
    // This part needs a robust way to generate temporary unique IDs
    // For now, using a simple counter or random number (placeholder)
    int64 TempId = FMath::RandRange(1000000000, 2000000000); 
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: SpawnObject request sent. Temporary ID: %lld"), TempId);
    return TempId;
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
    UClass* ObjectClass = nullptr;
    
    // Try to find the class by its path directly
    ObjectClass = FindObject<UClass>(nullptr, *ClassName);
    
    // If not found, try with potential package prefixes
    if (!ObjectClass)
    {
        // Try with a U prefix if not found
        if (!ClassName.StartsWith(TEXT("U")) && !ClassName.StartsWith(TEXT("A")))
        {
            FString PrefixedClassName = FString::Printf(TEXT("U%s"), *ClassName);
            ObjectClass = FindObject<UClass>(nullptr, *PrefixedClassName);
            
            // Also try with A prefix for actors
            if (!ObjectClass)
            {
                PrefixedClassName = FString::Printf(TEXT("A%s"), *ClassName);
                ObjectClass = FindObject<UClass>(nullptr, *PrefixedClassName);
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

void USpacetimeDBSubsystem::InternalHandleObjectCreated(uint64 ObjectId, const FString& ClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object created event - ID: %llu, Class: %s"), ObjectId, *ClassName);
    
    // Create the object
    UObject* NewObject = SpawnObjectFromServer(ObjectId, ClassName, DataJson);
    
    if (NewObject)
    {
        // Broadcast the object created event
        OnObjectCreated.Broadcast(ObjectId, ClassName, DataJson);
    }
}

void USpacetimeDBSubsystem::InternalHandleObjectDestroyed(uint64 ObjectId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object destroyed event - ID: %llu"), ObjectId);
    
    // Broadcast the object destroyed event before actually destroying it
    OnObjectDestroyed.Broadcast(ObjectId);
    
    // Destroy the object
    DestroyObjectFromServer(ObjectId);
}

void USpacetimeDBSubsystem::InternalHandleObjectIdRemapped(uint64 TempId, uint64 ServerId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Object ID remapped - Temp ID: %llu, Server ID: %llu"), TempId, ServerId);
    
    // Broadcast the object ID remapped event
    OnObjectIdRemapped.Broadcast(TempId, ServerId);
    
    // TODO: Implement ID remapping in the object registry
}

// Add methods for setting properties

bool USpacetimeDBSubsystem::SetPropertyValueFromJson(int64 ObjectId, const FString& PropertyName, const FString& ValueJson, bool bReplicateToServer)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SetPropertyValueFromJson - Not connected to SpacetimeDB"));
        return false;
    }
    
    // SECURITY: Check if client has authority to modify this object
    if (bReplicateToServer && !HasAuthority(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SetPropertyValueFromJson - Client does not have authority to modify object %lld"), ObjectId);
        return false;
    }
    
    // Find the object in our registry
    UObject* Object = FindObjectById(ObjectId);
    if (!Object)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SetPropertyValueFromJson - Object with ID %lld not found"), ObjectId);
        return false;
    }
    
    // Try to apply the property to the object
    bool bSuccess = FSpacetimeDBPropertyHelper::ApplyJsonToProperty(Object, PropertyName, ValueJson);
    
    // If successful and we need to replicate to server, send the update
    if (bSuccess && bReplicateToServer)
    {
        SendPropertyUpdateToServer(ObjectId, PropertyName, ValueJson);
    }
    
    return bSuccess;
}

bool USpacetimeDBSubsystem::SetPropertyValue(int64 ObjectId, const FString& PropertyName, UObject* Object, bool bReplicateToServer)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Cannot set property value with null object. Property: %s"), *PropertyName);
        return false;
    }

    // Serialize the property value to JSON
    FString ValueJson = FSpacetimeDBPropertyHelper::SerializePropertyToJson(Object, PropertyName);
    if (ValueJson.IsEmpty())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to serialize property %s on object %s"), 
            *PropertyName, *Object->GetName());
        return false;
    }

    // Apply locally first if Object isn't the target (it's a different object with the same property)
    UObject* TargetObject = FindObjectById(ObjectId);
    if (TargetObject && TargetObject != Object)
    {
        bool bLocalSuccess = FSpacetimeDBPropertyHelper::ApplyJsonToProperty(TargetObject, PropertyName, ValueJson);
        if (!bLocalSuccess)
        {
            UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Failed to apply property %s locally to target object %s"), 
                *PropertyName, *TargetObject->GetName());
            // Continue anyway to try server update
        }
    }

    // If we should replicate to the server, send the update
    if (bReplicateToServer)
    {
        return SendPropertyUpdateToServer(ObjectId, PropertyName, ValueJson);
    }

    return true;
}

bool USpacetimeDBSubsystem::SendPropertyUpdateToServer(int64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SendPropertyUpdateToServer - Not connected to SpacetimeDB"));
        return false;
    }
    
    // SECURITY: Check if client has authority to modify this object
    if (!HasAuthority(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: SendPropertyUpdateToServer - Client does not have authority to modify object %lld"), ObjectId);
        return false;
    }
    
    // Call the FFI function
    stdb::ffi::set_property(ObjectId, PropertyName.GetData(), ValueJson.GetData(), true);
    return true;
}

// RPC System Implementation

bool USpacetimeDBSubsystem::CallServerFunctionOnObject(UObject* TargetObject, const FString& FunctionName, const TArray<FStdbRpcArg>& Args)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: CallServerFunctionOnObject - Not connected to SpacetimeDB"));
        return false;
    }
    
    if (!TargetObject)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: CallServerFunctionOnObject - Target object is null"));
        return false;
    }
    
    // Get the object ID
    int64 ObjectId = GetObjectId(TargetObject);
    if (ObjectId == 0)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: CallServerFunctionOnObject - Cannot find SpacetimeDB ID for object %s"), *TargetObject->GetName());
        return false;
    }
    
    // Call through with the object ID
    return CallServerFunction(ObjectId, FunctionName, Args);
}

bool USpacetimeDBSubsystem::CallServerFunction(int64 ObjectId, const FString& FunctionName, const TArray<FStdbRpcArg>& Args)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: CallServerFunction - Not connected to SpacetimeDB"));
        return false;
    }
    
    // Check if this is a special RPC that doesn't require authority checks
    // For example, common RPCs that need to work on server-owned objects
    bool bIsSpecialRPC = 
        FunctionName.Equals(TEXT("set_owner")) ||
        FunctionName.Equals(TEXT("request_spawn")) ||
        FunctionName.Equals(TEXT("request_destroy")) ||
        FunctionName.StartsWith(TEXT("game_")) ||  // Game management RPCs can start with "game_"
        FunctionName.StartsWith(TEXT("server_"));  // Server management RPCs can start with "server_"
    
    // SECURITY: Check if client has authority to call RPCs on this object
    // Except for special RPCs that need to work regardless of authority
    if (!bIsSpecialRPC && !HasAuthority(ObjectId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: CallServerFunction - Client does not have authority to call RPC %s on object %lld"), 
            *FunctionName, ObjectId);
        return false;
    }
    
    // Serialize arguments to JSON
    FString ArgsJson = SerializeRpcArguments(Args);
    
    UE_LOG(LogTemp, Verbose, TEXT("SpacetimeDBSubsystem: Calling server function %s on object %lld with args: %s"), 
        *FunctionName, ObjectId, *ArgsJson);
    
    // Call the FFI function
    stdb::ffi::call_server_function(ObjectId, FunctionName.GetData(), ArgsJson.GetData());
    return true;
}

bool USpacetimeDBSubsystem::RegisterClientFunctionWithFFI(const FString& FunctionName)
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RegisterClientFunctionWithFFI - Not connected to SpacetimeDB"));
        return false;
    }
    
    if (FunctionName.IsEmpty())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: RegisterClientFunctionWithFFI - Function name is empty"));
        return false;
    }
    
    // Register the static callback function with the FFI
    bool bSuccess = stdb::ffi::register_client_function(
        FunctionName.GetData(), 
        reinterpret_cast<uintptr_t>(&USpacetimeDBSubsystem::HandleClientRpcFromFFI)
    );
    
    if (!bSuccess)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to register client function %s with FFI"), 
               *FunctionName);
    }
    else
    {
        UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Successfully registered client function %s with FFI"), 
               *FunctionName);
    }
    
    return bSuccess;
}

bool USpacetimeDBSubsystem::HandleClientRpcFromFFI(uint64 ObjectId, const char* ArgsJson)
{
    // This is a static function called from FFI, so we need to find the appropriate subsystem instance
    UGameInstance* GameInstance = nullptr;
    
    // Find all game instances
    for (TObjectIterator<UGameInstance> It; It; ++It)
    {
        GameInstance = *It;
        if (GameInstance && !GameInstance->IsPendingKill() && GameInstance->GetWorld() && GameInstance->GetWorld()->IsGameWorld())
        {
            break;
        }
    }
    
    if (!GameInstance)
    {
        UE_LOG(LogTemp, Error, TEXT("HandleClientRpcFromFFI: Failed to find valid game instance"));
        return false;
    }
    
    USpacetimeDBSubsystem* Subsystem = GSubsystemInstances.FindRef(GameInstance);
    if (!Subsystem)
    {
        UE_LOG(LogTemp, Error, TEXT("HandleClientRpcFromFFI: Failed to find SpacetimeDBSubsystem for game instance"));
        return false;
    }
    
    // Parse the args JSON to extract the function name
    FString ArgsJsonStr = UTF8_TO_TCHAR(ArgsJson);
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(ArgsJsonStr);
    
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("HandleClientRpcFromFFI: Failed to parse args JSON: %s"), *ArgsJsonStr);
        return false;
    }
    
    FString FunctionName;
    if (!JsonObject->TryGetStringField("function", FunctionName) || FunctionName.IsEmpty())
    {
        UE_LOG(LogTemp, Error, TEXT("HandleClientRpcFromFFI: Args JSON missing 'function' field: %s"), *ArgsJsonStr);
        return false;
    }
    
    // Extract the arguments if present
    TSharedPtr<FJsonObject> ArgsObj;
    if (!JsonObject->TryGetObjectField("args", ArgsObj))
    {
        ArgsObj = MakeShared<FJsonObject>();
    }
    
    // Hand off to the appropriate subsystem instance on the game thread
    AsyncTask(ENamedThreads::GameThread, [Subsystem, ObjectId, FunctionName, ArgsObj]() {
        Subsystem->HandleClientRpc(ObjectId, FunctionName, ArgsObj);
    });
    
    return true;
}

void USpacetimeDBSubsystem::HandleClientRpc(uint64 ObjectId, const FString& FunctionName, TSharedPtr<FJsonObject> ArgsObj)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Received client RPC %s for object %llu"), *FunctionName, ObjectId);
    
    // Serialize the arguments to a JSON string for ParseRpcArguments
    FString ArgsJson;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ArgsJson);
    FJsonSerializer::Serialize(ArgsObj.ToSharedRef(), Writer);
    
    // Parse the arguments into a more usable format
    TArray<FStdbRpcArg> Args = ParseRpcArguments(ArgsJson);
    
    // Find the registered handler for this function
    if (ClientRpcHandlers.Contains(FunctionName))
    {
        // Call the handler
        ClientRpcHandlers[FunctionName](ObjectId, Args);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: No handler registered for RPC function %s"), *FunctionName);
    }
    
    // Also broadcast the event for Blueprint listeners
    OnServerRpcReceived.Broadcast(ObjectId, FunctionName, Args);
}

TArray<FStdbRpcArg> USpacetimeDBSubsystem::ParseRpcArguments(const FString& ArgsJson)
{
    TArray<FStdbRpcArg> Args;
    
    // Parse the JSON string
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(ArgsJson);
    
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("ParseRpcArguments: Failed to parse JSON: %s"), *ArgsJson);
        return Args;
    }
    
    // Extract each field as an argument
    for (auto& Pair : JsonObject->Values)
    {
        FStdbRpcArg Arg;
        Arg.Name = Pair.Key;
        
        // Determine the type and value based on the JSON value
        if (Pair.Value->Type == EJson::Null)
        {
            Arg.Type = ESpacetimeDBValueType::Null;
        }
        else if (Pair.Value->Type == EJson::Boolean)
        {
            Arg.Type = ESpacetimeDBValueType::Bool;
            Arg.Value.SetBool(Pair.Value->AsBool());
        }
        else if (Pair.Value->Type == EJson::Number)
        {
            // Check if it's an integer or float
            double NumValue = Pair.Value->AsNumber();
            if (FMath::Frac(NumValue) == 0.0 && NumValue <= INT32_MAX && NumValue >= INT32_MIN)
            {
                Arg.Type = ESpacetimeDBValueType::Int;
                Arg.Value.SetInt(static_cast<int32>(NumValue));
            }
            else
            {
                Arg.Type = ESpacetimeDBValueType::Float;
                Arg.Value.SetFloat(static_cast<float>(NumValue));
            }
        }
        else if (Pair.Value->Type == EJson::String)
        {
            Arg.Type = ESpacetimeDBValueType::String;
            Arg.Value.SetString(Pair.Value->AsString());
        }
        else if (Pair.Value->Type == EJson::Object)
        {
            // JSON object - store as string representation
            Arg.Type = ESpacetimeDBValueType::CustomJson;
            FString JsonStr;
            TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonStr);
            FJsonSerializer::Serialize(Pair.Value->AsObject().ToSharedRef(), Writer);
            Arg.Value.SetCustomJson(JsonStr);
        }
        else if (Pair.Value->Type == EJson::Array)
        {
            // JSON array - store as string representation
            Arg.Type = ESpacetimeDBValueType::ArrayJson;
            FString JsonStr;
            TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonStr);
            FJsonSerializer::Serialize(Pair.Value->AsArray(), Writer);
            Arg.Value.SetArrayJson(JsonStr);
        }
        
        Args.Add(Arg);
    }
    
    return Args;
}

FString USpacetimeDBSubsystem::SerializeRpcArguments(const TArray<FStdbRpcArg>& Args)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShared<FJsonObject>();
    
    // Add each argument to the JSON object
    for (const FStdbRpcArg& Arg : Args)
    {
        switch (Arg.Type)
        {
            case ESpacetimeDBValueType::Null:
                JsonObject->SetField(Arg.Name, MakeShared<FJsonValueNull>());
                break;
                
            case ESpacetimeDBValueType::Bool:
                JsonObject->SetBoolField(Arg.Name, Arg.Value.GetBool());
                break;
                
            case ESpacetimeDBValueType::Int:
                JsonObject->SetNumberField(Arg.Name, Arg.Value.GetInt());
                break;
                
            case ESpacetimeDBValueType::Float:
                JsonObject->SetNumberField(Arg.Name, Arg.Value.GetFloat());
                break;
                
            case ESpacetimeDBValueType::String:
                JsonObject->SetStringField(Arg.Name, Arg.Value.GetString());
                break;
                
            case ESpacetimeDBValueType::CustomJson:
            {
                // Parse the custom JSON string and add it as an object
                TSharedPtr<FJsonObject> CustomObj;
                TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Arg.Value.GetCustomJson());
                if (FJsonSerializer::Deserialize(Reader, CustomObj) && CustomObj.IsValid())
                {
                    JsonObject->SetObjectField(Arg.Name, CustomObj);
                }
                else
                {
                    // Fallback to string if parsing fails
                    JsonObject->SetStringField(Arg.Name, Arg.Value.GetCustomJson());
                }
                break;
            }
                
            case ESpacetimeDBValueType::ArrayJson:
            {
                // Parse the array JSON string and add it as an array
                TArray<TSharedPtr<FJsonValue>> ArrayValues;
                TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Arg.Value.GetArrayJson());
                if (FJsonSerializer::Deserialize(Reader, ArrayValues))
                {
                    JsonObject->SetArrayField(Arg.Name, ArrayValues);
                }
                else
                {
                    // Fallback to string if parsing fails
                    JsonObject->SetStringField(Arg.Name, Arg.Value.GetArrayJson());
                }
                break;
            }
                
            default:
                // For unsupported types, add as null
                JsonObject->SetField(Arg.Name, MakeShared<FJsonValueNull>());
                break;
        }
    }
    
    // Serialize the JSON object to a string
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
    
    return OutputString;
}

int64 USpacetimeDBSubsystem::GetObjectId(UObject* Object) const
{
    if (!Object)
    {
        return 0;
    }
    
    const int64* ObjectId = ObjectToIdMap.Find(Object);
    if (ObjectId)
    {
        return *ObjectId;
    }
    
    return 0;
}

FString USpacetimeDBSubsystem::GetPropertyJsonValue(int64 ObjectId, const FString& PropertyName) const
{
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: GetPropertyJsonValue - Not connected to SpacetimeDB"));
        return FString();
    }
    
    // Call the FFI function to get the property value
    return stdb::ffi::get_property(ObjectId, PropertyName.GetData());
}

FSpacetimeDBPropertyValue USpacetimeDBSubsystem::GetPropertyValue(int64 ObjectId, const FString& PropertyName) const
{
    FString ValueJson = GetPropertyJsonValue(ObjectId, PropertyName);
    
    FSpacetimeDBPropertyValue PropertyValue;
    if (!ValueJson.IsEmpty())
    {
        // Parse the JSON value to determine the appropriate property type
        USpacetimeDBPropertyHelper::JsonToPropertyValue(ValueJson, PropertyValue);
    }
    
    return PropertyValue;
}

bool USpacetimeDBSubsystem::RegisterPredictionObject(const FObjectID& ObjectID)
{
	return register_prediction_object(ObjectID.ID);
}

bool USpacetimeDBSubsystem::UnregisterPredictionObject(const FObjectID& ObjectID)
{
	return unregister_prediction_object(ObjectID.ID);
}

int32 USpacetimeDBSubsystem::GetNextPredictionSequence(const FObjectID& ObjectID)
{
	return (int32)get_next_prediction_sequence(ObjectID.ID);
}

bool USpacetimeDBSubsystem::SendPredictedTransform(const FPredictedTransformData& TransformData)
{
	// Extract the transform components
	FVector Location = TransformData.Transform.GetLocation();
	FQuat Rotation = TransformData.Transform.GetRotation();
	FVector Scale = TransformData.Transform.GetScale3D();
	
	return send_predicted_transform(
		TransformData.ObjectID.ID,
		(SequenceNumber)TransformData.SequenceNumber,
		Location.X, Location.Y, Location.Z,
		Rotation.X, Rotation.Y, Rotation.Z, Rotation.W,
		Scale.X, Scale.Y, Scale.Z,
		TransformData.Velocity.X, TransformData.Velocity.Y, TransformData.Velocity.Z,
		TransformData.bHasVelocity
	);
}

int32 USpacetimeDBSubsystem::GetLastAckedSequence(const FObjectID& ObjectID)
{
	return (int32)get_last_acked_sequence(ObjectID.ID);
}

void USpacetimeDBSubsystem::ProcessServerTransformUpdate(const FObjectID& ObjectID, const FTransform& Transform, 
	const FVector& Velocity, int32 AckedSequence)
{
	// Look up the object in our object map
	if (UObject* Object = FindObjectById(ObjectID.ID))
	{
		if (AActor* Actor = Cast<AActor>(Object))
		{
			// Look for prediction component
			if (USpacetimeDBPredictionComponent* PredComp = Actor->FindComponentByClass<USpacetimeDBPredictionComponent>())
			{
				// Let the prediction component handle this
				PredComp->ProcessServerUpdate(Transform, Velocity, AckedSequence);
			}
			else 
			{
				// No prediction component, just set the transform directly
				Actor->SetActorTransform(Transform);
			}
		}
	}
}

bool USpacetimeDBSubsystem::HasAuthority(int64 ObjectId) const
{
    if (!IsConnected())
    {
        return false;
    }
    
    // Get the owner ID of the object
    int64 OwnerClientId = GetOwnerClientId(ObjectId);
    
    // Only the explicit owner has authority to modify an object
    return OwnerClientId == GetClientId();
}

int64 USpacetimeDBSubsystem::GetOwnerClientId(int64 ObjectId) const
{
    if (!IsConnected())
    {
        return 0;
    }
    
    // Get the object's owner_id property
    FString OwnerIdJson = GetPropertyJsonValue(ObjectId, TEXT("owner_id"));
    if (OwnerIdJson.IsEmpty() || OwnerIdJson == TEXT("null"))
    {
        return 0; // No owner (server-owned)
    }
    
    // Parse the JSON string to get the owner ID
    int64 OwnerClientId = FCString::Atoi64(*OwnerIdJson);
    return OwnerClientId;
}

bool USpacetimeDBSubsystem::HasOwnership(int64 ObjectId) const
{
    return GetOwnerClientId(ObjectId) == GetClientId();
}

bool USpacetimeDBSubsystem::RequestSetOwner(int64 ObjectId, int64 NewOwnerClientId)
{
    if (!IsConnected())
    {
        return false;
    }
    
    // This must go through a server RPC since changing ownership is a privileged operation
    return CallServerFunction(ObjectId, TEXT("set_owner"), 
        TArray<FStdbRpcArg>{FStdbRpcArg(TEXT("new_owner_id"), NewOwnerClientId)});
}

//============================
// Component Replication Implementation
//============================

UActorComponent* USpacetimeDBSubsystem::HandleComponentAdded(int64 ActorId, int64 ComponentId, const FString& ComponentClassName, const FString& DataJson)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: HandleComponentAdded - Actor: %lld, Component: %lld, Class: %s"), 
        ActorId, ComponentId, *ComponentClassName);
    
    // First check if the actor exists
    AActor* OwnerActor = Cast<AActor>(FindObjectById(ActorId));
    if (!OwnerActor)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: HandleComponentAdded - Actor with ID %lld not found"), ActorId);
        return nullptr;
    }
    
    // Check if the component already exists (could be a remap)
    if (UActorComponent* ExistingComponent = Cast<UActorComponent>(FindObjectById(ComponentId)))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Component with ID %lld already exists"), ComponentId);
        return ExistingComponent;
    }
    
    // Parse JSON data for component properties
    TSharedPtr<FJsonObject> ComponentDataJson;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(DataJson);
    if (!FJsonSerializer::Deserialize(Reader, ComponentDataJson))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to parse component data JSON: %s"), *DataJson);
        return nullptr;
    }
    
    // Find the component class by name
    UClass* ComponentClass = FindObject<UClass>(ANY_PACKAGE, *ComponentClassName);
    if (!ComponentClass)
    {
        // Try with a U prefix if not found
        if (!ComponentClassName.StartsWith(TEXT("U")))
        {
            FString PrefixedClassName = FString::Printf(TEXT("U%s"), *ComponentClassName);
            ComponentClass = FindObject<UClass>(ANY_PACKAGE, *PrefixedClassName);
        }
        
        if (!ComponentClass)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Could not find component class '%s'"), *ComponentClassName);
            return nullptr;
        }
    }
    
    // Verify it's actually a UActorComponent class
    if (!ComponentClass->IsChildOf(UActorComponent::StaticClass()))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Class '%s' is not a UActorComponent"), *ComponentClassName);
        return nullptr;
    }
    
    // Create the component
    UActorComponent* NewComponent = NewObject<UActorComponent>(OwnerActor, ComponentClass);
    if (!NewComponent)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to create component of class '%s'"), *ComponentClassName);
        return nullptr;
    }
    
    // Register the component with the actor
    NewComponent->RegisterComponent();
    
    // Apply initial properties from JSON if provided
    TSharedPtr<FJsonObject> PropertiesObj = ComponentDataJson->GetObjectField("properties");
    if (PropertiesObj.IsValid())
    {
        // Apply each property from JSON to the component
        for (const auto& Pair : PropertiesObj->Values)
        {
            const FString& PropertyName = Pair.Key;
            const TSharedPtr<FJsonValue>& JsonValue = Pair.Value;
            
            // Apply property using our property helper
            bool bSuccess = FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty(NewComponent, PropertyName, JsonValue);
            if (!bSuccess)
            {
                UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: Failed to apply property '%s' to component"), *PropertyName);
            }
        }
    }
    
    // Register the component in our registry
    ObjectRegistry.Add(ComponentId, NewComponent);
    ObjectToIdMap.Add(NewComponent, ComponentId);
    
    // Broadcast the component added event
    OnComponentAdded.Broadcast(ActorId, ComponentId, ComponentClassName);
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Successfully added component '%s' with ID %lld to actor %lld"), 
        *ComponentClassName, ComponentId, ActorId);
    
    return NewComponent;
}

bool USpacetimeDBSubsystem::HandleComponentRemoved(int64 ActorId, int64 ComponentId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: HandleComponentRemoved - Actor: %lld, Component: %lld"), 
        ActorId, ComponentId);
    
    // Find the actor and component
    AActor* OwnerActor = Cast<AActor>(FindObjectById(ActorId));
    UActorComponent* Component = Cast<UActorComponent>(FindObjectById(ComponentId));
    
    if (!OwnerActor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: HandleComponentRemoved - Actor with ID %lld not found"), ActorId);
        return false;
    }
    
    if (!Component)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: HandleComponentRemoved - Component with ID %lld not found"), ComponentId);
        return false;
    }
    
    // Verify the component is actually attached to this actor
    if (Component->GetOwner() != OwnerActor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: HandleComponentRemoved - Component %lld is not attached to actor %lld"), 
            ComponentId, ActorId);
        return false;
    }
    
    // Remove from registry first
    ObjectRegistry.Remove(ComponentId);
    ObjectToIdMap.Remove(Component);
    
    // Unregister and destroy the component
    Component->UnregisterComponent();
    Component->DestroyComponent();
    
    // Broadcast the component removed event
    OnComponentRemoved.Broadcast(ActorId, ComponentId);
    
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: Successfully removed component with ID %lld from actor %lld"), 
        ComponentId, ActorId);
    
    return true;
}

UActorComponent* USpacetimeDBSubsystem::GetComponentById(int64 ComponentId) const
{
    UObject* Object = FindObjectById(ComponentId);
    return Cast<UActorComponent>(Object);
}

TArray<int64> USpacetimeDBSubsystem::GetComponentIdsForActor(int64 ActorId) const
{
    TArray<int64> Result;
    
    // Get the actor
    AActor* Actor = Cast<AActor>(FindObjectById(ActorId));
    if (!Actor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: GetComponentIdsForActor - Actor with ID %lld not found"), ActorId);
        return Result;
    }
    
    // Request the components from the Rust client
    FString ArgsJson = FString::Printf(TEXT("{\"actor_id\":%lld}"), ActorId);
    FString ResultJson = Client.CallReducerSync(TEXT("get_components"), ArgsJson);
    
    // Parse the result
    TSharedPtr<FJsonObject> ResultObj;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(ResultJson);
    if (!FJsonSerializer::Deserialize(Reader, ResultObj))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: Failed to parse get_components result JSON: %s"), *ResultJson);
        return Result;
    }
    
    // Extract the component IDs
    TArray<TSharedPtr<FJsonValue>> ComponentIds = ResultObj->GetArrayField(TEXT("components"));
    for (const TSharedPtr<FJsonValue>& ComponentIdValue : ComponentIds)
    {
        Result.Add(static_cast<int64>(ComponentIdValue->AsNumber()));
    }
    
    return Result;
}

TArray<int64> USpacetimeDBSubsystem::GetComponentIdsForActorObject(AActor* Actor) const
{
    if (!Actor)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: GetComponentIdsForActorObject - Actor is null"));
        return TArray<int64>();
    }
    
    int64 ActorId = GetObjectId(Actor);
    if (ActorId == 0)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: GetComponentIdsForActorObject - Actor %s has no SpacetimeDB ID"), 
            *Actor->GetName());
        return TArray<int64>();
    }
    
    return GetComponentIdsForActor(ActorId);
}

int64 USpacetimeDBSubsystem::RequestAddComponent(int64 ActorId, const FString& ComponentClassName)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: RequestAddComponent - Actor: %lld, Component Class: %s"), 
        ActorId, *ComponentClassName);
    
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestAddComponent - Not connected to SpacetimeDB"));
        return 0;
    }
    
    // SECURITY: Check if client has authority to modify this actor
    if (!HasAuthority(ActorId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestAddComponent - Client does not have authority to modify actor %lld"), 
            ActorId);
        return 0;
    }
    
    // Find the actor in our registry
    AActor* Actor = Cast<AActor>(FindObjectById(ActorId));
    if (!Actor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestAddComponent - Actor with ID %lld not found"), ActorId);
        return 0;
    }
    
    // Create JSON args for the reducer call
    FString ArgsJson = FString::Printf(TEXT("{\"actor_id\":%lld,\"component_class\":\"%s\"}"), 
        ActorId, *ComponentClassName);
    
    // Call the reducer
    if (!Client.CallReducer(TEXT("create_and_attach_component"), ArgsJson))
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDBSubsystem: RequestAddComponent - Failed to call reducer"));
        return 0;
    }
    
    // NOTE: We don't have the component ID yet - it will come from the server via HandleComponentAdded
    // For now, return a placeholder/temporary ID
    return -1; // Temporary ID that will be replaced
}

bool USpacetimeDBSubsystem::RequestRemoveComponent(int64 ActorId, int64 ComponentId)
{
    UE_LOG(LogTemp, Log, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Actor: %lld, Component: %lld"), 
        ActorId, ComponentId);
    
    if (!IsConnected())
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Not connected to SpacetimeDB"));
        return false;
    }
    
    // SECURITY: Check if client has authority to modify this actor
    if (!HasAuthority(ActorId))
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Client does not have authority to modify actor %lld"), 
            ActorId);
        return false;
    }
    
    // Find the actor and component
    AActor* Actor = Cast<AActor>(FindObjectById(ActorId));
    UActorComponent* Component = Cast<UActorComponent>(FindObjectById(ComponentId));
    
    if (!Actor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Actor with ID %lld not found"), ActorId);
        return false;
    }
    
    if (!Component)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Component with ID %lld not found"), ComponentId);
        return false;
    }
    
    // Verify the component is actually attached to this actor
    if (Component->GetOwner() != Actor)
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDBSubsystem: RequestRemoveComponent - Component %lld is not attached to actor %lld"), 
            ComponentId, ActorId);
        return false;
    }
    
    // Create JSON args for the reducer call
    FString ArgsJson = FString::Printf(TEXT("{\"actor_id\":%lld,\"component_id\":%lld}"), 
        ActorId, ComponentId);
    
    // Call the reducer
    return Client.CallReducer(TEXT("remove_component"), ArgsJson);
} 