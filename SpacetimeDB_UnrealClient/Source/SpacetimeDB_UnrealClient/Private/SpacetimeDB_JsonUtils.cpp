#include "SpacetimeDB_JsonUtils.h"
#include "SpacetimeDB_Types.h"

#include "Dom/JsonObject.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonWriter.h"
#include "UObject/PropertyPortFlags.h"

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
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
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
    FJsonSerializer::Serialize(JsonArray, Writer);
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
            KeyProperty->ExportTextItem(KeyString, KeyPtr, nullptr, nullptr, PPF_None);
            
            TSharedPtr<FJsonValue> ValueJson = SerializePropertyToJsonValue(ValueProperty, ValuePtr);
            if (ValueJson.IsValid())
            {
                JsonObject->SetField(KeyString, ValueJson);
            }
        }
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
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
        
        KeyProperty->ImportText(*Pair.Key, TempKeyPtr, PPF_None, nullptr);
        
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
    FJsonSerializer::Serialize(JsonArray, Writer);
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
    JsonObject->SetNumberField("X", Vector.X);
    JsonObject->SetNumberField("Y", Vector.Y);
    JsonObject->SetNumberField("Z", Vector.Z);
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
    if (JsonObject->TryGetNumberField("X", X) && 
        JsonObject->TryGetNumberField("Y", Y) && 
        JsonObject->TryGetNumberField("Z", Z))
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
    JsonObject->SetNumberField("Pitch", Rotator.Pitch);
    JsonObject->SetNumberField("Yaw", Rotator.Yaw);
    JsonObject->SetNumberField("Roll", Rotator.Roll);
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
    if (JsonObject->TryGetNumberField("Pitch", Pitch) && 
        JsonObject->TryGetNumberField("Yaw", Yaw) && 
        JsonObject->TryGetNumberField("Roll", Roll))
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
    JsonObject->SetObjectField("Location", VectorToJson(Transform.GetLocation()));
    
    // Rotation (convert to Rotator for easier serialization)
    JsonObject->SetObjectField("Rotation", RotatorToJson(Transform.Rotator()));
    
    // Scale
    JsonObject->SetObjectField("Scale", VectorToJson(Transform.GetScale3D()));
    
    return JsonObject;
}

// Converts JSON object to FTransform
bool USpacetimeDBJsonUtils::JsonToTransform(const TSharedPtr<FJsonObject>& JsonObject, FTransform& OutTransform)
{
    if (!JsonObject.IsValid())
    {
        return false;
    }
    
    FVector Location = FVector::ZeroVector;
    FRotator Rotation = FRotator::ZeroRotator;
    FVector Scale = FVector(1.0f, 1.0f, 1.0f);
    
    // Location
    TSharedPtr<FJsonObject> LocationObj = JsonObject->GetObjectField("Location");
    if (LocationObj.IsValid())
    {
        JsonToVector(LocationObj, Location);
    }
    
    // Rotation
    TSharedPtr<FJsonObject> RotationObj = JsonObject->GetObjectField("Rotation");
    if (RotationObj.IsValid())
    {
        JsonToRotator(RotationObj, Rotation);
    }
    
    // Scale
    TSharedPtr<FJsonObject> ScaleObj = JsonObject->GetObjectField("Scale");
    if (ScaleObj.IsValid())
    {
        JsonToVector(ScaleObj, Scale);
    }
    
    OutTransform.SetLocation(Location);
    OutTransform.SetRotation(Rotation.Quaternion());
    OutTransform.SetScale3D(Scale);
    
    return true;
}

// Serializes RPC arguments to a JSON string
FString USpacetimeDBJsonUtils::SerializeRpcArgsToJson(const TArray<TSharedPtr<FJsonValue>>& Args)
{
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(Args, Writer);
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
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
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

// Private helper to convert property to JSON value
TSharedPtr<FJsonValue> USpacetimeDBJsonUtils::SerializePropertyToJsonValue(FProperty* Property, const void* ValuePtr)
{
    if (!Property || !ValuePtr)
    {
        return nullptr;
    }
    
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        bool Value = BoolProperty->GetPropertyValue(ValuePtr);
        return MakeShareable(new FJsonValueBoolean(Value));
    }
    else if (FNumericProperty* NumericProperty = CastField<FNumericProperty>(Property))
    {
        // Handle numeric properties (int, float, etc.)
        if (NumericProperty->IsFloatingPoint())
        {
            double Value = NumericProperty->GetFloatingPointPropertyValue(ValuePtr);
            return MakeShareable(new FJsonValueNumber(Value));
        }
        else
        {
            int64 Value = NumericProperty->GetSignedIntPropertyValue(ValuePtr);
            return MakeShareable(new FJsonValueNumber(Value));
        }
    }
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        FString Value = StringProperty->GetPropertyValue(ValuePtr);
        return MakeShareable(new FJsonValueString(Value));
    }
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        FName Value = NameProperty->GetPropertyValue(ValuePtr);
        return MakeShareable(new FJsonValueString(Value.ToString()));
    }
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        FText Value = TextProperty->GetPropertyValue(ValuePtr);
        return MakeShareable(new FJsonValueString(Value.ToString()));
    }
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        FString JsonString = SerializeArrayToJson(ArrayProperty, ValuePtr);
        TArray<TSharedPtr<FJsonValue>> JsonArray;
        TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
        FJsonSerializer::Deserialize(Reader, JsonArray);
        return MakeShareable(new FJsonValueArray(JsonArray));
    }
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        FString JsonString = SerializeMapToJson(MapProperty, ValuePtr);
        TSharedPtr<FJsonObject> JsonObject;
        TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
        FJsonSerializer::Deserialize(Reader, JsonObject);
        return MakeShareable(new FJsonValueObject(JsonObject));
    }
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        FString JsonString = SerializeSetToJson(SetProperty, ValuePtr);
        TArray<TSharedPtr<FJsonValue>> JsonArray;
        TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
        FJsonSerializer::Deserialize(Reader, JsonArray);
        return MakeShareable(new FJsonValueArray(JsonArray));
    }
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        UScriptStruct* ScriptStruct = StructProperty->Struct;
        
        // Special handling for common struct types
        if (ScriptStruct == TBaseStructure<FVector>::Get())
        {
            const FVector* Vector = reinterpret_cast<const FVector*>(ValuePtr);
            TSharedPtr<FJsonObject> JsonObject = VectorToJson(*Vector);
            return MakeShareable(new FJsonValueObject(JsonObject));
        }
        else if (ScriptStruct == TBaseStructure<FRotator>::Get())
        {
            const FRotator* Rotator = reinterpret_cast<const FRotator*>(ValuePtr);
            TSharedPtr<FJsonObject> JsonObject = RotatorToJson(*Rotator);
            return MakeShareable(new FJsonValueObject(JsonObject));
        }
        else if (ScriptStruct == TBaseStructure<FTransform>::Get())
        {
            const FTransform* Transform = reinterpret_cast<const FTransform*>(ValuePtr);
            TSharedPtr<FJsonObject> JsonObject = TransformToJson(*Transform);
            return MakeShareable(new FJsonValueObject(JsonObject));
        }
        else
        {
            FString JsonString = SerializeStructToJson(ScriptStruct, ValuePtr);
            TSharedPtr<FJsonObject> JsonObject;
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
            FJsonSerializer::Deserialize(Reader, JsonObject);
            return MakeShareable(new FJsonValueObject(JsonObject));
        }
    }
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        UObject* Object = ObjectProperty->GetObjectPropertyValue(ValuePtr);
        if (Object)
        {
            // For UObjects, just store a reference by name
            return MakeShareable(new FJsonValueString(Object->GetName()));
        }
    }
    
    // Default case - try to export as text
    FString ExportedText;
    Property->ExportTextItem(ExportedText, ValuePtr, nullptr, nullptr, PPF_None);
    return MakeShareable(new FJsonValueString(ExportedText));
}

// Private helper to convert JSON value to property
bool USpacetimeDBJsonUtils::DeserializeJsonValueToProperty(FProperty* Property, void* ValuePtr, const TSharedPtr<FJsonValue>& JsonValue)
{
    if (!Property || !ValuePtr || !JsonValue.IsValid())
    {
        return false;
    }
    
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        bool Value = false;
        if (JsonValue->TryGetBool(Value))
        {
            BoolProperty->SetPropertyValue(ValuePtr, Value);
            return true;
        }
    }
    else if (FNumericProperty* NumericProperty = CastField<FNumericProperty>(Property))
    {
        // Handle numeric properties (int, float, etc.)
        double DoubleValue;
        if (JsonValue->TryGetNumber(DoubleValue))
        {
            if (NumericProperty->IsFloatingPoint())
            {
                NumericProperty->SetFloatingPointPropertyValue(ValuePtr, DoubleValue);
            }
            else
            {
                NumericProperty->SetIntPropertyValue(ValuePtr, static_cast<int64>(DoubleValue));
            }
            return true;
        }
    }
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        FString Value;
        if (JsonValue->TryGetString(Value))
        {
            StringProperty->SetPropertyValue(ValuePtr, Value);
            return true;
        }
    }
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        FString Value;
        if (JsonValue->TryGetString(Value))
        {
            NameProperty->SetPropertyValue(ValuePtr, FName(*Value));
            return true;
        }
    }
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        FString Value;
        if (JsonValue->TryGetString(Value))
        {
            TextProperty->SetPropertyValue(ValuePtr, FText::FromString(Value));
            return true;
        }
    }
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        const TArray<TSharedPtr<FJsonValue>>* JsonArray;
        if (JsonValue->TryGetArray(JsonArray))
        {
            FString JsonString;
            TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
            FJsonSerializer::Serialize(*JsonArray, Writer);
            Writer->Close();
            
            return DeserializeJsonToArray(ArrayProperty, ValuePtr, JsonString);
        }
    }
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        const TSharedPtr<FJsonObject>* JsonObject;
        if (JsonValue->TryGetObject(JsonObject))
        {
            FString JsonString;
            TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
            FJsonSerializer::Serialize(*JsonObject, Writer);
            Writer->Close();
            
            return DeserializeJsonToMap(MapProperty, ValuePtr, JsonString);
        }
    }
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        const TArray<TSharedPtr<FJsonValue>>* JsonArray;
        if (JsonValue->TryGetArray(JsonArray))
        {
            FString JsonString;
            TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
            FJsonSerializer::Serialize(*JsonArray, Writer);
            Writer->Close();
            
            return DeserializeJsonToSet(SetProperty, ValuePtr, JsonString);
        }
    }
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        UScriptStruct* ScriptStruct = StructProperty->Struct;
        const TSharedPtr<FJsonObject>* JsonObject;
        
        // Special handling for common struct types
        if (JsonValue->TryGetObject(JsonObject))
        {
            if (ScriptStruct == TBaseStructure<FVector>::Get())
            {
                FVector* Vector = reinterpret_cast<FVector*>(ValuePtr);
                return JsonToVector(*JsonObject, *Vector);
            }
            else if (ScriptStruct == TBaseStructure<FRotator>::Get())
            {
                FRotator* Rotator = reinterpret_cast<FRotator*>(ValuePtr);
                return JsonToRotator(*JsonObject, *Rotator);
            }
            else if (ScriptStruct == TBaseStructure<FTransform>::Get())
            {
                FTransform* Transform = reinterpret_cast<FTransform*>(ValuePtr);
                return JsonToTransform(*JsonObject, *Transform);
            }
            else
            {
                FString JsonString;
                TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
                FJsonSerializer::Serialize(*JsonObject, Writer);
                Writer->Close();
                
                return DeserializeJsonToStruct(ScriptStruct, ValuePtr, JsonString);
            }
        }
    }
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        FString ObjectName;
        if (JsonValue->TryGetString(ObjectName))
        {
            // Find the object by name
            UObject* TargetObject = nullptr;
            
            // First look for the object in the global package
            TargetObject = FindObject<UObject>(ANY_PACKAGE, *ObjectName);
            
            // If not found and the property has a specific UClass, try to find by class
            if (!TargetObject && ObjectProperty->PropertyClass)
            {
                TargetObject = FindObject<UObject>(ANY_PACKAGE, *ObjectName, false);
            }
            
            ObjectProperty->SetObjectPropertyValue(ValuePtr, TargetObject);
            return TargetObject != nullptr;
        }
    }
    
    // Default case - try to import as text
    FString StringValue;
    if (JsonValue->TryGetString(StringValue))
    {
        Property->ImportText(*StringValue, ValuePtr, PPF_None, nullptr);
        return true;
    }
    
    return false;
}

// Helper to convert JSON value to string
FString USpacetimeDBJsonUtils::JsonValueToString(const TSharedPtr<FJsonValue>& JsonValue)
{
    if (!JsonValue.IsValid())
    {
        return TEXT("null");
    }
    
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonValue.ToSharedRef(), Writer);
    Writer->Close();
    
    return OutputString;
}