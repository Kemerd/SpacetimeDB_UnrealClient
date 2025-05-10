#include "SpacetimeDB_JsonUtils.h"
#include "SpacetimeDB_Types.h"

#include "Dom/JsonObject.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonWriter.h"
#include "UObject/PropertyPortFlags.h"
#include "UObject/TextProperty.h" // For PropertyPortFlags
#include "UObject/PropertyAccessUtil.h" // For text import/export utilities

// Serializes a property to JSON string
FString USpacetimeDBJsonUtils::SerializePropertyToJson(FProperty* Property, const void* ValuePtr)
{
    TSharedPtr<FJsonValue> JsonValue = SerializePropertyToJsonValue(Property, ValuePtr);
    return JsonValueToString(JsonValue);
}

// Deserializes JSON string to a property
bool USpacetimeDBJsonUtils::DeserializeJsonToProperty(FProperty* Property, void* ValuePtr, const FString& JsonString)
{
    TSharedPtr<FJsonValue> JsonValue;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonValue) || !JsonValue.IsValid())
    {
        return false;
    }
    
    return DeserializeJsonValueToProperty(Property, ValuePtr, JsonValue);
}

// Serializes a UStruct to JSON string
FString USpacetimeDBJsonUtils::SerializeStructToJson(UScriptStruct* StructType, const void* StructPtr)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    for (TFieldIterator<FProperty> It(StructType); It; ++It)
    {
        FProperty* Property = *It;
        const void* ValuePtr = Property->ContainerPtrToValuePtr<uint8>(StructPtr);
        
        TSharedPtr<FJsonValue> JsonValue = SerializePropertyToJsonValue(Property, ValuePtr);
        if (JsonValue.IsValid())
        {
            JsonObject->SetField(Property->GetName(), JsonValue);
        }
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes JSON string to a UStruct
bool USpacetimeDBJsonUtils::DeserializeJsonToStruct(UScriptStruct* StructType, void* StructPtr, const FString& JsonString)
{
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        return false;
    }
    
    for (TFieldIterator<FProperty> It(StructType); It; ++It)
    {
        FProperty* Property = *It;
        const FString PropertyName = Property->GetName();
        
        TSharedPtr<FJsonValue> JsonValue = JsonObject->TryGetField(PropertyName);
        if (JsonValue.IsValid())
        {
            void* ValuePtr = Property->ContainerPtrToValuePtr<void>(StructPtr);
            DeserializeJsonValueToProperty(Property, ValuePtr, JsonValue);
        }
    }
    
    return true;
}

// Serializes an array to JSON string
FString USpacetimeDBJsonUtils::SerializeArrayToJson(FArrayProperty* ArrayProperty, const void* ArrayPtr)
{
    FScriptArrayHelper ArrayHelper(ArrayProperty, ArrayPtr);
    FProperty* InnerProperty = ArrayProperty->Inner;
    
    TArray<TSharedPtr<FJsonValue>> JsonArray;
    for (int32 i = 0; i < ArrayHelper.Num(); i++)
    {
        void* ElementPtr = ArrayHelper.GetRawPtr(i);
        TSharedPtr<FJsonValue> ElementValue = SerializePropertyToJsonValue(InnerProperty, ElementPtr);
        if (ElementValue.IsValid())
        {
            JsonArray.Add(ElementValue);
        }
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonArray, Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes JSON string to an array
bool USpacetimeDBJsonUtils::DeserializeJsonToArray(FArrayProperty* ArrayProperty, void* ArrayPtr, const FString& JsonString)
{
    TArray<TSharedPtr<FJsonValue>> JsonArray;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonArray))
    {
        return false;
    }
    
    FScriptArrayHelper ArrayHelper(ArrayProperty, ArrayPtr);
    ArrayHelper.EmptyValues();
    
    FProperty* InnerProperty = ArrayProperty->Inner;
    for (const TSharedPtr<FJsonValue>& JsonValue : JsonArray)
    {
        if (JsonValue.IsValid())
        {
            const int32 NewIndex = ArrayHelper.AddValue();
            void* ElementPtr = ArrayHelper.GetRawPtr(NewIndex);
            DeserializeJsonValueToProperty(InnerProperty, ElementPtr, JsonValue);
        }
    }
    
    return true;
}

// Serializes a map to JSON string
FString USpacetimeDBJsonUtils::SerializeMapToJson(FMapProperty* MapProperty, const void* MapPtr)
{
    FScriptMapHelper MapHelper(MapProperty, MapPtr);
    FProperty* KeyProperty = MapProperty->KeyProp;
    FProperty* ValueProperty = MapProperty->ValueProp;
    
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    for (int32 i = 0; i < MapHelper.Num(); i++)
    {
        if (MapHelper.IsValidIndex(i))
        {
            void* KeyPtr = MapHelper.GetKeyPtr(i);
            void* ValuePtr = MapHelper.GetValuePtr(i);
            
            // Convert key to string - maps in JSON need string keys
            FString KeyString;
            FPropertyAccessUtils::PropertyToString(KeyProperty, KeyPtr, KeyString, nullptr);
            
            TSharedPtr<FJsonValue> ValueJson = SerializePropertyToJsonValue(ValueProperty, ValuePtr);
            if (ValueJson.IsValid())
            {
                JsonObject->SetField(KeyString, ValueJson);
            }
        }
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes JSON string to a map
bool USpacetimeDBJsonUtils::DeserializeJsonToMap(FMapProperty* MapProperty, void* MapPtr, const FString& JsonString)
{
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        return false;
    }
    
    FScriptMapHelper MapHelper(MapProperty, MapPtr);
    MapHelper.EmptyValues();
    
    FProperty* KeyProperty = MapProperty->KeyProp;
    FProperty* ValueProperty = MapProperty->ValueProp;
    
    for (const auto& Pair : JsonObject->Values)
    {
        void* TempKeyPtr = FMemory::Malloc(KeyProperty->GetSize(), KeyProperty->GetMinAlignment());
        KeyProperty->InitializeValue(TempKeyPtr);
        
        FPropertyAccessUtils::StringToProperty(Pair.Key, KeyProperty, TempKeyPtr, nullptr);
        
        int32 Index = MapHelper.AddDefaultValue_Invalid_NeedsRehash();
        void* KeyPtr = MapHelper.GetKeyPtr(Index);
        void* ValuePtr = MapHelper.GetValuePtr(Index);
        
        KeyProperty->CopyCompleteValue(KeyPtr, TempKeyPtr);
        DeserializeJsonValueToProperty(ValueProperty, ValuePtr, Pair.Value);
        
        KeyProperty->DestroyValue(TempKeyPtr);
        FMemory::Free(TempKeyPtr);
    }
    
    MapHelper.Rehash();
    return true;
}

// Serializes a set to JSON string
FString USpacetimeDBJsonUtils::SerializeSetToJson(FSetProperty* SetProperty, const void* SetPtr)
{
    FScriptSetHelper SetHelper(SetProperty, SetPtr);
    FProperty* ElementProperty = SetProperty->ElementProp;
    
    TArray<TSharedPtr<FJsonValue>> JsonArray;
    for (int32 i = 0; i < SetHelper.Num(); i++)
    {
        if (SetHelper.IsValidIndex(i))
        {
            void* ElementPtr = SetHelper.GetElementPtr(i);
            TSharedPtr<FJsonValue> ElementValue = SerializePropertyToJsonValue(ElementProperty, ElementPtr);
            if (ElementValue.IsValid())
            {
                JsonArray.Add(ElementValue);
            }
        }
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonArray, Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes JSON string to a set
bool USpacetimeDBJsonUtils::DeserializeJsonToSet(FSetProperty* SetProperty, void* SetPtr, const FString& JsonString)
{
    TArray<TSharedPtr<FJsonValue>> JsonArray;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonArray))
    {
        return false;
    }
    
    FScriptSetHelper SetHelper(SetProperty, SetPtr);
    SetHelper.EmptyElements();
    
    FProperty* ElementProperty = SetProperty->ElementProp;
    for (const TSharedPtr<FJsonValue>& JsonValue : JsonArray)
    {
        if (JsonValue.IsValid())
        {
            void* TempElementPtr = FMemory::Malloc(ElementProperty->GetSize(), ElementProperty->GetMinAlignment());
            ElementProperty->InitializeValue(TempElementPtr);
            
            if (DeserializeJsonValueToProperty(ElementProperty, TempElementPtr, JsonValue))
            {
                SetHelper.AddElement(TempElementPtr);
            }
            
            ElementProperty->DestroyValue(TempElementPtr);
            FMemory::Free(TempElementPtr);
        }
    }
    
    return true;
}

// Converts FVector to JSON object
TSharedPtr<FJsonObject> USpacetimeDBJsonUtils::VectorToJson(const FVector& Vector)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    JsonObject->SetNumberField(TEXT("X"), Vector.X);
    JsonObject->SetNumberField(TEXT("Y"), Vector.Y);
    JsonObject->SetNumberField(TEXT("Z"), Vector.Z);
    return JsonObject;
}

// Converts JSON object to FVector
bool USpacetimeDBJsonUtils::JsonToVector(const TSharedPtr<FJsonObject>& JsonObject, FVector& OutVector)
{
    if (!JsonObject.IsValid())
    {
        return false;
    }
    
    double X = 0.0, Y = 0.0, Z = 0.0;
    if (JsonObject->TryGetNumberField(TEXT("X"), X) && 
        JsonObject->TryGetNumberField(TEXT("Y"), Y) && 
        JsonObject->TryGetNumberField(TEXT("Z"), Z))
    {
        OutVector.X = X;
        OutVector.Y = Y;
        OutVector.Z = Z;
        return true;
    }
    
    return false;
}

// Converts FRotator to JSON object
TSharedPtr<FJsonObject> USpacetimeDBJsonUtils::RotatorToJson(const FRotator& Rotator)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    JsonObject->SetNumberField(TEXT("Pitch"), Rotator.Pitch);
    JsonObject->SetNumberField(TEXT("Yaw"), Rotator.Yaw);
    JsonObject->SetNumberField(TEXT("Roll"), Rotator.Roll);
    return JsonObject;
}

// Converts JSON object to FRotator
bool USpacetimeDBJsonUtils::JsonToRotator(const TSharedPtr<FJsonObject>& JsonObject, FRotator& OutRotator)
{
    if (!JsonObject.IsValid())
    {
        return false;
    }
    
    double Pitch = 0.0, Yaw = 0.0, Roll = 0.0;
    if (JsonObject->TryGetNumberField(TEXT("Pitch"), Pitch) && 
        JsonObject->TryGetNumberField(TEXT("Yaw"), Yaw) && 
        JsonObject->TryGetNumberField(TEXT("Roll"), Roll))
    {
        OutRotator.Pitch = Pitch;
        OutRotator.Yaw = Yaw;
        OutRotator.Roll = Roll;
        return true;
    }
    
    return false;
}

// Converts FTransform to JSON object
TSharedPtr<FJsonObject> USpacetimeDBJsonUtils::TransformToJson(const FTransform& Transform)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    
    // Location
    JsonObject->SetObjectField(TEXT("Location"), VectorToJson(Transform.GetLocation()));
    
    // Rotation (convert to Rotator for easier serialization)
    JsonObject->SetObjectField(TEXT("Rotation"), RotatorToJson(Transform.Rotator()));
    
    // Scale
    JsonObject->SetObjectField(TEXT("Scale"), VectorToJson(Transform.GetScale3D()));
    
    return JsonObject;
}

// Converts JSON object to FTransform
bool USpacetimeDBJsonUtils::JsonToTransform(const TSharedPtr<FJsonObject>& JsonObject, FTransform& OutTransform)
{
    if (!JsonObject.IsValid())
    {
        return false;
    }
    
    // Location
    const TSharedPtr<FJsonObject>* LocationObj;
    FVector Location = FVector::ZeroVector;
    if (JsonObject->TryGetObjectField(TEXT("Location"), LocationObj) && JsonToVector(*LocationObj, Location))
    {
        OutTransform.SetLocation(Location);
    }
    else
    {
        return false;
    }
    
    // Rotation
    const TSharedPtr<FJsonObject>* RotationObj;
    FRotator Rotation = FRotator::ZeroRotator;
    if (JsonObject->TryGetObjectField(TEXT("Rotation"), RotationObj) && JsonToRotator(*RotationObj, Rotation))
    {
        OutTransform.SetRotation(Rotation.Quaternion());
    }
    else
    {
        return false;
    }
    
    // Scale
    const TSharedPtr<FJsonObject>* ScaleObj;
    FVector Scale = FVector::OneVector;
    if (JsonObject->TryGetObjectField(TEXT("Scale"), ScaleObj) && JsonToVector(*ScaleObj, Scale))
    {
        OutTransform.SetScale3D(Scale);
    }
    else
    {
        return false;
    }
    
    return true;
}

// Serializes RPC arguments to a JSON string
FString USpacetimeDBJsonUtils::SerializeRpcArgsToJson(const TArray<TSharedPtr<FJsonValue>>& Args)
{
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(Args, *Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes a JSON string to RPC arguments
bool USpacetimeDBJsonUtils::DeserializeJsonToRpcArgs(const FString& JsonString, TArray<TSharedPtr<FJsonValue>>& OutArgs)
{
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    return FJsonSerializer::Deserialize(Reader, OutArgs);
}

// Serializes RPC result to a JSON string
FString USpacetimeDBJsonUtils::SerializeRpcResultToJson(const TSharedPtr<FJsonValue>& Result)
{
    if (!Result.IsValid())
    {
        return TEXT("null");
    }
    
    return JsonValueToString(Result);
}

// Deserializes a JSON string to an RPC result
bool USpacetimeDBJsonUtils::DeserializeJsonToRpcResult(const FString& JsonString, TSharedPtr<FJsonValue>& OutResult)
{
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    return FJsonSerializer::Deserialize(Reader, OutResult);
}

// Serializes spawn parameters to a JSON string
FString USpacetimeDBJsonUtils::SerializeSpawnParamsToJson(const FSpacetimeDBSpawnParams& SpawnParams)
{
    TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
    
    JsonObject->SetStringField("ClassName", SpawnParams.ClassName);
    JsonObject->SetObjectField("Location", VectorToJson(SpawnParams.Location));
    JsonObject->SetObjectField("Rotation", RotatorToJson(SpawnParams.Rotation));
    JsonObject->SetBoolField("Replicate", SpawnParams.bReplicate);
    JsonObject->SetNumberField("OwnerClientId", SpawnParams.OwnerClientId);
    
    // Serialize initial properties map
    TSharedPtr<FJsonObject> PropertiesObject = MakeShareable(new FJsonObject);
    for (const auto& Pair : SpawnParams.InitialProperties)
    {
        PropertiesObject->SetStringField(Pair.Key, Pair.Value);
    }
    JsonObject->SetObjectField("InitialProperties", PropertiesObject);
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), *Writer, false);
    Writer->Close();
    
    return OutputString;
}

// Deserializes a JSON string to spawn parameters
bool USpacetimeDBJsonUtils::DeserializeJsonToSpawnParams(const FString& JsonString, FSpacetimeDBSpawnParams& OutSpawnParams)
{
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        return false;
    }
    
    JsonObject->TryGetStringField("ClassName", OutSpawnParams.ClassName);
    JsonObject->TryGetBoolField("Replicate", OutSpawnParams.bReplicate);
    JsonObject->TryGetNumberField("OwnerClientId", OutSpawnParams.OwnerClientId);
    
    // Location
    TSharedPtr<FJsonObject> LocationObj = JsonObject->GetObjectField("Location");
    if (LocationObj.IsValid())
    {
        JsonToVector(LocationObj, OutSpawnParams.Location);
    }
    
    // Rotation
    TSharedPtr<FJsonObject> RotationObj = JsonObject->GetObjectField("Rotation");
    if (RotationObj.IsValid())
    {
        JsonToRotator(RotationObj, OutSpawnParams.Rotation);
    }
    
    // Initial properties
    OutSpawnParams.InitialProperties.Empty();
    TSharedPtr<FJsonObject> PropertiesObj = JsonObject->GetObjectField("InitialProperties");
    if (PropertiesObj.IsValid())
    {
        for (const auto& Pair : PropertiesObj->Values)
        {
            FString Value;
            if (Pair.Value->TryGetString(Value))
            {
                OutSpawnParams.InitialProperties.Add(Pair.Key, Value);
            }
        }
    }
    
    return true;
}

// Serializes a property to a JsonValue
TSharedPtr<FJsonValue> USpacetimeDBJsonUtils::SerializePropertyToJsonValue(FProperty* Property, const void* ValuePtr)
{
    if (!Property || !ValuePtr)
    {
        return nullptr;
    }
    
    // Boolean
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        return MakeShared<FJsonValueBoolean>(BoolProperty->GetPropertyValue(ValuePtr));
    }
    // Byte
    else if (FByteProperty* ByteProperty = CastField<FByteProperty>(Property))
    {
        if (ByteProperty->IsEnum())
        {
            UEnum* Enum = ByteProperty->GetEnum();
            uint8 Value = ByteProperty->GetPropertyValue(ValuePtr);
            return MakeShared<FJsonValueString>(Enum->GetNameStringByValue(Value));
        }
        else
        {
            return MakeShared<FJsonValueNumber>(ByteProperty->GetPropertyValue(ValuePtr));
        }
    }
    // Numbers
    else if (FIntProperty* IntProperty = CastField<FIntProperty>(Property))
    {
        return MakeShared<FJsonValueNumber>(IntProperty->GetPropertyValue(ValuePtr));
    }
    else if (FInt64Property* Int64Property = CastField<FInt64Property>(Property))
    {
        return MakeShared<FJsonValueNumber>((double)Int64Property->GetPropertyValue(ValuePtr));
    }
    else if (FUInt32Property* UInt32Property = CastField<FUInt32Property>(Property))
    {
        return MakeShared<FJsonValueNumber>((double)UInt32Property->GetPropertyValue(ValuePtr));
    }
    else if (FUInt64Property* UInt64Property = CastField<FUInt64Property>(Property))
    {
        return MakeShared<FJsonValueNumber>((double)UInt64Property->GetPropertyValue(ValuePtr));
    }
    else if (FFloatProperty* FloatProperty = CastField<FFloatProperty>(Property))
    {
        return MakeShared<FJsonValueNumber>(FloatProperty->GetPropertyValue(ValuePtr));
    }
    else if (FDoubleProperty* DoubleProperty = CastField<FDoubleProperty>(Property))
    {
        return MakeShared<FJsonValueNumber>(DoubleProperty->GetPropertyValue(ValuePtr));
    }
    // String
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        return MakeShared<FJsonValueString>(StringProperty->GetPropertyValue(ValuePtr));
    }
    // Name
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        return MakeShared<FJsonValueString>(NameProperty->GetPropertyValue(ValuePtr).ToString());
    }
    // Text
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        return MakeShared<FJsonValueString>(TextProperty->GetPropertyValue(ValuePtr).ToString());
    }
    // Struct
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        // Handle common UE struct types
        if (StructProperty->Struct->GetFName() == TEXT("Vector"))
        {
            return MakeShared<FJsonValueObject>(VectorToJson(*static_cast<const FVector*>(ValuePtr)));
        }
        else if (StructProperty->Struct->GetFName() == TEXT("Rotator"))
        {
            return MakeShared<FJsonValueObject>(RotatorToJson(*static_cast<const FRotator*>(ValuePtr)));
        }
        else if (StructProperty->Struct->GetFName() == TEXT("Transform"))
        {
            return MakeShared<FJsonValueObject>(TransformToJson(*static_cast<const FTransform*>(ValuePtr)));
        }
        // Generic struct serialization
        else
        {
            TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
            for (TFieldIterator<FProperty> It(StructProperty->Struct); It; ++It)
            {
                FProperty* SubProperty = *It;
                const void* SubValuePtr = SubProperty->ContainerPtrToValuePtr<uint8>(ValuePtr);
                
                TSharedPtr<FJsonValue> SubJsonValue = SerializePropertyToJsonValue(SubProperty, SubValuePtr);
                if (SubJsonValue.IsValid())
                {
                    JsonObject->SetField(SubProperty->GetName(), SubJsonValue);
                }
            }
            return MakeShared<FJsonValueObject>(JsonObject);
        }
    }
    // Enum
    else if (FEnumProperty* EnumProperty = CastField<FEnumProperty>(Property))
    {
        UEnum* Enum = EnumProperty->GetEnum();
        int64 Value = EnumProperty->GetUnderlyingProperty()->GetSignedIntPropertyValue(ValuePtr);
        return MakeShared<FJsonValueString>(Enum->GetNameStringByValue(Value));
    }
    // Object/Class references
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        UObject* Object = ObjectProperty->GetObjectPropertyValue(ValuePtr);
        if (Object)
        {
            return MakeShared<FJsonValueString>(Object->GetPathName());
        }
        return MakeShared<FJsonValueNull>();
    }
    else if (FClassProperty* ClassProperty = CastField<FClassProperty>(Property))
    {
        UClass* Class = Cast<UClass>(ClassProperty->GetObjectPropertyValue(ValuePtr));
        if (Class)
        {
            return MakeShared<FJsonValueString>(Class->GetPathName());
        }
        return MakeShared<FJsonValueNull>();
    }
    // Array
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        FScriptArrayHelper ArrayHelper(ArrayProperty, ValuePtr);
        FProperty* InnerProperty = ArrayProperty->Inner;
        
        TArray<TSharedPtr<FJsonValue>> JsonArray;
        for (int32 i = 0; i < ArrayHelper.Num(); i++)
        {
            void* ElementPtr = ArrayHelper.GetRawPtr(i);
            TSharedPtr<FJsonValue> ElementValue = SerializePropertyToJsonValue(InnerProperty, ElementPtr);
            if (ElementValue.IsValid())
            {
                JsonArray.Add(ElementValue);
            }
        }
        
        return MakeShared<FJsonValueArray>(JsonArray);
    }
    // Map
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        FScriptMapHelper MapHelper(MapProperty, ValuePtr);
        FProperty* KeyProperty = MapProperty->KeyProp;
        FProperty* ValueProperty = MapProperty->ValueProp;
        
        TSharedPtr<FJsonObject> JsonObject = MakeShareable(new FJsonObject);
        for (int32 i = 0; i < MapHelper.Num(); i++)
        {
            if (MapHelper.IsValidIndex(i))
            {
                void* KeyPtr = MapHelper.GetKeyPtr(i);
                void* ValuePtr = MapHelper.GetValuePtr(i);
                
                // Convert key to string
                FString KeyString;
                FPropertyAccessUtils::PropertyToString(KeyProperty, KeyPtr, KeyString, nullptr);
                
                TSharedPtr<FJsonValue> ValueJson = SerializePropertyToJsonValue(ValueProperty, ValuePtr);
                if (ValueJson.IsValid())
                {
                    JsonObject->SetField(KeyString, ValueJson);
                }
            }
        }
        
        return MakeShared<FJsonValueObject>(JsonObject);
    }
    // Set
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        FScriptSetHelper SetHelper(SetProperty, ValuePtr);
        FProperty* ElementProperty = SetProperty->ElementProp;
        
        TArray<TSharedPtr<FJsonValue>> JsonArray;
        for (int32 i = 0; i < SetHelper.Num(); i++)
        {
            if (SetHelper.IsValidIndex(i))
            {
                void* ElementPtr = SetHelper.GetElementPtr(i);
                TSharedPtr<FJsonValue> ElementValue = SerializePropertyToJsonValue(ElementProperty, ElementPtr);
                if (ElementValue.IsValid())
                {
                    JsonArray.Add(ElementValue);
                }
            }
        }
        
        return MakeShared<FJsonValueArray>(JsonArray);
    }
    
    // Unsupported property type, convert to string (for debug purposes)
    FString ValueString;
    FPropertyAccessUtils::PropertyToString(Property, ValuePtr, ValueString, nullptr);
    return MakeShared<FJsonValueString>(ValueString);
}

// Deserializes a JsonValue to a property
bool USpacetimeDBJsonUtils::DeserializeJsonValueToProperty(FProperty* Property, void* ValuePtr, const TSharedPtr<FJsonValue>& JsonValue)
{
    if (!Property || !ValuePtr || !JsonValue.IsValid())
    {
        return false;
    }
    
    // Handle null values
    if (JsonValue->Type == EJson::Null)
    {
        Property->ClearValue(ValuePtr);
        return true;
    }
    
    // Boolean
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        if (JsonValue->Type == EJson::Boolean)
        {
            BoolProperty->SetPropertyValue(ValuePtr, JsonValue->AsBool());
            return true;
        }
    }
    // Byte
    else if (FByteProperty* ByteProperty = CastField<FByteProperty>(Property))
    {
        if (ByteProperty->IsEnum())
        {
            if (JsonValue->Type == EJson::String)
            {
                UEnum* Enum = ByteProperty->GetEnum();
                int64 Value = Enum->GetValueByNameString(JsonValue->AsString());
                if (Value != INDEX_NONE)
                {
                    ByteProperty->SetPropertyValue(ValuePtr, static_cast<uint8>(Value));
                    return true;
                }
            }
        }
        else if (JsonValue->Type == EJson::Number)
        {
            ByteProperty->SetPropertyValue(ValuePtr, static_cast<uint8>(JsonValue->AsNumber()));
            return true;
        }
    }
    // Numbers
    else if (FIntProperty* IntProperty = CastField<FIntProperty>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            IntProperty->SetPropertyValue(ValuePtr, static_cast<int32>(JsonValue->AsNumber()));
            return true;
        }
    }
    else if (FInt64Property* Int64Property = CastField<FInt64Property>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            Int64Property->SetPropertyValue(ValuePtr, static_cast<int64>(JsonValue->AsNumber()));
            return true;
        }
    }
    else if (FUInt32Property* UInt32Property = CastField<FUInt32Property>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            UInt32Property->SetPropertyValue(ValuePtr, static_cast<uint32>(JsonValue->AsNumber()));
            return true;
        }
    }
    else if (FUInt64Property* UInt64Property = CastField<FUInt64Property>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            UInt64Property->SetPropertyValue(ValuePtr, static_cast<uint64>(JsonValue->AsNumber()));
            return true;
        }
    }
    else if (FFloatProperty* FloatProperty = CastField<FFloatProperty>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            FloatProperty->SetPropertyValue(ValuePtr, static_cast<float>(JsonValue->AsNumber()));
            return true;
        }
    }
    else if (FDoubleProperty* DoubleProperty = CastField<FDoubleProperty>(Property))
    {
        if (JsonValue->Type == EJson::Number)
        {
            DoubleProperty->SetPropertyValue(ValuePtr, JsonValue->AsNumber());
            return true;
        }
    }
    // String
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            StringProperty->SetPropertyValue(ValuePtr, JsonValue->AsString());
            return true;
        }
    }
    // Name
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            NameProperty->SetPropertyValue(ValuePtr, FName(*JsonValue->AsString()));
            return true;
        }
    }
    // Text
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            TextProperty->SetPropertyValue(ValuePtr, FText::FromString(JsonValue->AsString()));
            return true;
        }
    }
    // Struct
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        if (JsonValue->Type == EJson::Object)
        {
            // Handle common UE struct types
            if (StructProperty->Struct->GetFName() == TEXT("Vector"))
            {
                FVector* VectorPtr = static_cast<FVector*>(ValuePtr);
                return JsonToVector(JsonValue->AsObject(), *VectorPtr);
            }
            else if (StructProperty->Struct->GetFName() == TEXT("Rotator"))
            {
                FRotator* RotatorPtr = static_cast<FRotator*>(ValuePtr);
                return JsonToRotator(JsonValue->AsObject(), *RotatorPtr);
            }
            else if (StructProperty->Struct->GetFName() == TEXT("Transform"))
            {
                FTransform* TransformPtr = static_cast<FTransform*>(ValuePtr);
                return JsonToTransform(JsonValue->AsObject(), *TransformPtr);
            }
            // Generic struct deserialization
            else
            {
                const TSharedPtr<FJsonObject>& JsonObject = JsonValue->AsObject();
                for (TFieldIterator<FProperty> It(StructProperty->Struct); It; ++It)
                {
                    FProperty* SubProperty = *It;
                    void* SubValuePtr = SubProperty->ContainerPtrToValuePtr<void>(ValuePtr);
                    
                    TSharedPtr<FJsonValue> SubJsonValue = JsonObject->TryGetField(SubProperty->GetName());
                    if (SubJsonValue.IsValid())
                    {
                        DeserializeJsonValueToProperty(SubProperty, SubValuePtr, SubJsonValue);
                    }
                }
                return true;
            }
        }
    }
    // Enum
    else if (FEnumProperty* EnumProperty = CastField<FEnumProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            UEnum* Enum = EnumProperty->GetEnum();
            int64 Value = Enum->GetValueByNameString(JsonValue->AsString());
            if (Value != INDEX_NONE)
            {
                void* UnderlyingValuePtr = EnumProperty->GetUnderlyingProperty()->ContainerPtrToValuePtr<void>(ValuePtr);
                EnumProperty->GetUnderlyingProperty()->SetIntPropertyValue(UnderlyingValuePtr, Value);
                return true;
            }
        }
    }
    // Object/Class references
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            // Find the object by path name
            UObject* Object = FindFirstObject<UObject>(*JsonValue->AsString(), EFindFirstObjectOptions::None);
            if (Object && Object->IsA(ObjectProperty->PropertyClass))
            {
                ObjectProperty->SetObjectPropertyValue(ValuePtr, Object);
                return true;
            }
        }
    }
    else if (FClassProperty* ClassProperty = CastField<FClassProperty>(Property))
    {
        if (JsonValue->Type == EJson::String)
        {
            // Find the class by path name
            UClass* Class = FindFirstObject<UClass>(*JsonValue->AsString(), EFindFirstObjectOptions::None);
            if (Class && Class->IsChildOf(ClassProperty->MetaClass))
            {
                ClassProperty->SetObjectPropertyValue(ValuePtr, Class);
                return true;
            }
        }
    }
    // Array
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        if (JsonValue->Type == EJson::Array)
        {
            FScriptArrayHelper ArrayHelper(ArrayProperty, ValuePtr);
            ArrayHelper.EmptyValues();
            
            FProperty* InnerProperty = ArrayProperty->Inner;
            const TArray<TSharedPtr<FJsonValue>>& JsonArray = JsonValue->AsArray();
            
            for (const TSharedPtr<FJsonValue>& ElementValue : JsonArray)
            {
                if (ElementValue.IsValid())
                {
                    const int32 NewIndex = ArrayHelper.AddValue();
                    void* ElementPtr = ArrayHelper.GetRawPtr(NewIndex);
                    DeserializeJsonValueToProperty(InnerProperty, ElementPtr, ElementValue);
                }
            }
            
            return true;
        }
    }
    // Map
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        if (JsonValue->Type == EJson::Object)
        {
            FScriptMapHelper MapHelper(MapProperty, ValuePtr);
            MapHelper.EmptyValues();
            
            FProperty* KeyProperty = MapProperty->KeyProp;
            FProperty* ValueProperty = MapProperty->ValueProp;
            
            const TSharedPtr<FJsonObject>& JsonObject = JsonValue->AsObject();
            for (const auto& Pair : JsonObject->Values)
            {
                void* TempKeyPtr = FMemory::Malloc(KeyProperty->GetSize(), KeyProperty->GetMinAlignment());
                KeyProperty->InitializeValue(TempKeyPtr);
                
                FPropertyAccessUtils::StringToProperty(Pair.Key, KeyProperty, TempKeyPtr, nullptr);
                
                int32 Index = MapHelper.AddDefaultValue_Invalid_NeedsRehash();
                void* KeyPtr = MapHelper.GetKeyPtr(Index);
                void* ValuePtr = MapHelper.GetValuePtr(Index);
                
                KeyProperty->CopyCompleteValue(KeyPtr, TempKeyPtr);
                DeserializeJsonValueToProperty(ValueProperty, ValuePtr, Pair.Value);
                
                KeyProperty->DestroyValue(TempKeyPtr);
                FMemory::Free(TempKeyPtr);
            }
            
            MapHelper.Rehash();
            return true;
        }
    }
    // Set
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        if (JsonValue->Type == EJson::Array)
        {
            FScriptSetHelper SetHelper(SetProperty, ValuePtr);
            SetHelper.EmptyElements();
            
            FProperty* ElementProperty = SetProperty->ElementProp;
            const TArray<TSharedPtr<FJsonValue>>& JsonArray = JsonValue->AsArray();
            
            for (const TSharedPtr<FJsonValue>& ElementValue : JsonArray)
            {
                if (ElementValue.IsValid())
                {
                    void* TempElementPtr = FMemory::Malloc(ElementProperty->GetSize(), ElementProperty->GetMinAlignment());
                    ElementProperty->InitializeValue(TempElementPtr);
                    
                    if (DeserializeJsonValueToProperty(ElementProperty, TempElementPtr, ElementValue))
                    {
                        SetHelper.AddElement(TempElementPtr);
                    }
                    
                    ElementProperty->DestroyValue(TempElementPtr);
                    FMemory::Free(TempElementPtr);
                }
            }
            
            return true;
        }
    }
    
    // Try generic string conversion for any property type as a fallback
    if (JsonValue->Type == EJson::String)
    {
        const FString& StringValue = JsonValue->AsString();
        return FPropertyAccessUtils::StringToProperty(StringValue, Property, ValuePtr, nullptr);
    }
    
    return false;
}

// Converts a JsonValue to string
FString USpacetimeDBJsonUtils::JsonValueToString(const TSharedPtr<FJsonValue>& JsonValue)
{
    if (!JsonValue.IsValid())
    {
        return FString();
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    
    if (JsonValue->Type == EJson::Object)
    {
        FJsonSerializer::Serialize(JsonValue->AsObject().ToSharedRef(), Writer);
    }
    else if (JsonValue->Type == EJson::Array)
    {
        FJsonSerializer::Serialize(JsonValue->AsArray(), Writer);
    }
    else
    {
        // For simple types like string, number, boolean
        TArray<TSharedPtr<FJsonValue>> SingleValueArray;
        SingleValueArray.Add(JsonValue);
        FJsonSerializer::Serialize(SingleValueArray, Writer);
        
        // Extract just the single value from the array
        if (OutputString.StartsWith(TEXT("[")) && OutputString.EndsWith(TEXT("]")))
        {
            OutputString = OutputString.Mid(1, OutputString.Len() - 2).TrimStartAndEnd();
        }
    }
    
    Writer->Close();
    return OutputString;
}