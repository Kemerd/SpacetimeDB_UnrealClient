#include "SpacetimeDB_PropertyValue.h"
#include "SpacetimeDB_JsonUtils.h"
#include "SpacetimeDBClient.h"
#include "JsonObjectConverter.h"
#include "Dom/JsonObject.h"
#include "Serialization/JsonWriter.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "UObject/UnrealType.h"

// Convert the PropertyValue to a JSON string
FString FSpacetimeDBPropertyValue::ToJsonString() const
{
    TSharedPtr<FJsonObject> JsonObject = MakeShared<FJsonObject>();
    
    // Add the type field
    JsonObject->SetStringField("type", StaticEnum<ESpacetimeDBPropertyType>()->GetNameStringByValue(static_cast<int64>(Type)));
    
    // Add the appropriate value field based on the type
    switch (Type)
    {
    case ESpacetimeDBPropertyType::Bool:
        JsonObject->SetBoolField("value", BoolValue);
        break;
    case ESpacetimeDBPropertyType::Byte:
        JsonObject->SetNumberField("value", ByteValue);
        break;
    case ESpacetimeDBPropertyType::Int32:
        JsonObject->SetNumberField("value", Int32Value);
        break;
    case ESpacetimeDBPropertyType::Int64:
        JsonObject->SetNumberField("value", Int64Value);
        break;
    case ESpacetimeDBPropertyType::UInt32:
        JsonObject->SetNumberField("value", UInt32Value);
        break;
    case ESpacetimeDBPropertyType::UInt64:
        JsonObject->SetNumberField("value", UInt64Value);
        break;
    case ESpacetimeDBPropertyType::Float:
        JsonObject->SetNumberField("value", FloatValue);
        break;
    case ESpacetimeDBPropertyType::Double:
        JsonObject->SetNumberField("value", DoubleValue);
        break;
    case ESpacetimeDBPropertyType::String:
    case ESpacetimeDBPropertyType::Name:
    case ESpacetimeDBPropertyType::Text:
    case ESpacetimeDBPropertyType::ClassReference:
        JsonObject->SetStringField("value", StringValue);
        break;
    case ESpacetimeDBPropertyType::Vector:
        JsonObject->SetObjectField("value", USpacetimeDBJsonUtils::VectorToJson(VectorValue));
        break;
    case ESpacetimeDBPropertyType::Rotator:
        JsonObject->SetObjectField("value", USpacetimeDBJsonUtils::RotatorToJson(RotatorValue));
        break;
    case ESpacetimeDBPropertyType::Quat:
        {
            TSharedPtr<FJsonObject> QuatObj = MakeShared<FJsonObject>();
            QuatObj->SetNumberField("x", QuatValue.X);
            QuatObj->SetNumberField("y", QuatValue.Y);
            QuatObj->SetNumberField("z", QuatValue.Z);
            QuatObj->SetNumberField("w", QuatValue.W);
            JsonObject->SetObjectField("value", QuatObj);
        }
        break;
    case ESpacetimeDBPropertyType::Transform:
        JsonObject->SetObjectField("value", USpacetimeDBJsonUtils::TransformToJson(TransformValue));
        break;
    case ESpacetimeDBPropertyType::Color:
        {
            TSharedPtr<FJsonObject> ColorObj = MakeShared<FJsonObject>();
            ColorObj->SetNumberField("r", ColorValue.R);
            ColorObj->SetNumberField("g", ColorValue.G);
            ColorObj->SetNumberField("b", ColorValue.B);
            ColorObj->SetNumberField("a", ColorValue.A);
            JsonObject->SetObjectField("value", ColorObj);
        }
        break;
    case ESpacetimeDBPropertyType::ObjectReference:
        JsonObject->SetNumberField("value", ObjectReferenceValue.Value);
        break;
    case ESpacetimeDBPropertyType::Array:
    case ESpacetimeDBPropertyType::Map:
    case ESpacetimeDBPropertyType::Set:
    case ESpacetimeDBPropertyType::Custom:
        // For JSON values, we need to parse the JSON string and include it directly
        {
            TSharedPtr<FJsonValue> JsonValue;
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonValue);
            if (FJsonSerializer::Deserialize(Reader, JsonValue))
            {
                JsonObject->SetField("value", JsonValue);
            }
            else
            {
                // If parsing fails, include the raw string
                JsonObject->SetStringField("value", JsonValue);
            }
        }
        break;
    case ESpacetimeDBPropertyType::None:
        // No value field for None type
        break;
    default:
        UE_LOG(LogTemp, Error, TEXT("Unknown property type in ToJsonString"));
        break;
    }
    
    // Serialize the JSON object to a string
    FString OutputString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
    
    return OutputString;
}

// Parse a PropertyValue from a JSON string
FSpacetimeDBPropertyValue FSpacetimeDBPropertyValue::FromJsonString(const FString& JsonString)
{
    FSpacetimeDBPropertyValue Result;
    
    // Parse the JSON string
    TSharedPtr<FJsonObject> JsonObject;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(JsonString);
    if (!FJsonSerializer::Deserialize(Reader, JsonObject) || !JsonObject.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("Failed to parse PropertyValue JSON: %s"), *JsonString);
        return Result;
    }
    
    // Get the type field
    FString TypeStr;
    if (!JsonObject->TryGetStringField("type", TypeStr))
    {
        UE_LOG(LogTemp, Error, TEXT("PropertyValue JSON missing 'type' field: %s"), *JsonString);
        return Result;
    }
    
    // Convert the type string to the enum
    const UEnum* PropertyTypeEnum = StaticEnum<ESpacetimeDBPropertyType>();
    int64 TypeValue = PropertyTypeEnum->GetValueByNameString(TypeStr);
    if (TypeValue == INDEX_NONE)
    {
        UE_LOG(LogTemp, Error, TEXT("Invalid PropertyValue type: %s"), *TypeStr);
        return Result;
    }
    
    Result.Type = static_cast<ESpacetimeDBPropertyType>(TypeValue);
    
    // Get the value field based on the type
    switch (Result.Type)
    {
    case ESpacetimeDBPropertyType::Bool:
        if (!JsonObject->TryGetBoolField("value", Result.BoolValue))
        {
            UE_LOG(LogTemp, Error, TEXT("Failed to get bool value from PropertyValue JSON"));
        }
        break;
    case ESpacetimeDBPropertyType::Byte:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.ByteValue = static_cast<uint8>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get byte value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Int32:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.Int32Value = static_cast<int32>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get int32 value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Int64:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.Int64Value = static_cast<int64>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get int64 value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::UInt32:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.UInt32Value = static_cast<uint32>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get uint32 value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::UInt64:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.UInt64Value = static_cast<uint64>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get uint64 value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Float:
        {
            double NumValue = 0;
            if (JsonObject->TryGetNumberField("value", NumValue))
            {
                Result.FloatValue = static_cast<float>(NumValue);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get float value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Double:
        if (!JsonObject->TryGetNumberField("value", Result.DoubleValue))
        {
            UE_LOG(LogTemp, Error, TEXT("Failed to get double value from PropertyValue JSON"));
        }
        break;
    case ESpacetimeDBPropertyType::String:
    case ESpacetimeDBPropertyType::Name:
    case ESpacetimeDBPropertyType::Text:
    case ESpacetimeDBPropertyType::ClassReference:
        if (!JsonObject->TryGetStringField("value", Result.StringValue))
        {
            UE_LOG(LogTemp, Error, TEXT("Failed to get string value from PropertyValue JSON"));
        }
        break;
    case ESpacetimeDBPropertyType::Vector:
        {
            const TSharedPtr<FJsonObject>* VectorObj;
            if (JsonObject->TryGetObjectField("value", VectorObj))
            {
                if (!USpacetimeDBJsonUtils::JsonToVector(*VectorObj, Result.VectorValue))
                {
                    UE_LOG(LogTemp, Error, TEXT("Failed to parse Vector from PropertyValue JSON"));
                }
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get Vector object from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Rotator:
        {
            const TSharedPtr<FJsonObject>* RotatorObj;
            if (JsonObject->TryGetObjectField("value", RotatorObj))
            {
                if (!USpacetimeDBJsonUtils::JsonToRotator(*RotatorObj, Result.RotatorValue))
                {
                    UE_LOG(LogTemp, Error, TEXT("Failed to parse Rotator from PropertyValue JSON"));
                }
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get Rotator object from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Quat:
        {
            const TSharedPtr<FJsonObject>* QuatObj;
            if (JsonObject->TryGetObjectField("value", QuatObj))
            {
                double X = 0, Y = 0, Z = 0, W = 1;
                (*QuatObj)->TryGetNumberField("x", X);
                (*QuatObj)->TryGetNumberField("y", Y);
                (*QuatObj)->TryGetNumberField("z", Z);
                (*QuatObj)->TryGetNumberField("w", W);
                Result.QuatValue = FQuat(X, Y, Z, W);
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get Quat object from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Transform:
        {
            const TSharedPtr<FJsonObject>* TransformObj;
            if (JsonObject->TryGetObjectField("value", TransformObj))
            {
                if (!USpacetimeDBJsonUtils::JsonToTransform(*TransformObj, Result.TransformValue))
                {
                    UE_LOG(LogTemp, Error, TEXT("Failed to parse Transform from PropertyValue JSON"));
                }
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get Transform object from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Color:
        {
            const TSharedPtr<FJsonObject>* ColorObj;
            if (JsonObject->TryGetObjectField("value", ColorObj))
            {
                double R = 0, G = 0, B = 0, A = 255;
                (*ColorObj)->TryGetNumberField("r", R);
                (*ColorObj)->TryGetNumberField("g", G);
                (*ColorObj)->TryGetNumberField("b", B);
                (*ColorObj)->TryGetNumberField("a", A);
                Result.ColorValue = FColor(
                    static_cast<uint8>(FMath::Clamp(R, 0.0, 255.0)),
                    static_cast<uint8>(FMath::Clamp(G, 0.0, 255.0)),
                    static_cast<uint8>(FMath::Clamp(B, 0.0, 255.0)),
                    static_cast<uint8>(FMath::Clamp(A, 0.0, 255.0))
                );
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get Color object from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::ObjectReference:
        {
            double ObjIdValue = 0;
            if (JsonObject->TryGetNumberField("value", ObjIdValue))
            {
                Result.ObjectReferenceValue = FSpacetimeDBObjectID(static_cast<int64>(ObjIdValue));
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get ObjectReference value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::Array:
    case ESpacetimeDBPropertyType::Map:
    case ESpacetimeDBPropertyType::Set:
    case ESpacetimeDBPropertyType::Custom:
        {
            const TSharedPtr<FJsonValue>* JsonValue;
            if (JsonObject->TryGetField("value", JsonValue))
            {
                // Serialize the value back to a string
                FString ValueString;
                TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ValueString);
                FJsonSerializer::Serialize((*JsonValue)->AsObject().ToSharedRef(), Writer);
                Result.JsonValue = ValueString;
            }
            else
            {
                UE_LOG(LogTemp, Error, TEXT("Failed to get JSON value from PropertyValue JSON"));
            }
        }
        break;
    case ESpacetimeDBPropertyType::None:
        // No value to parse for None type
        break;
    default:
        UE_LOG(LogTemp, Error, TEXT("Unknown property type in FromJsonString: %d"), static_cast<int32>(Result.Type));
        break;
    }
    
    return Result;
}

// Apply a property value to a UObject property
bool USpacetimeDBPropertyHandler::ApplyPropertyToObject(UObject* Object, const FString& PropertyName, const FSpacetimeDBPropertyValue& PropValue)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("ApplyPropertyToObject: Object is null"));
        return false;
    }
    
    // Find the property in the object
    FProperty* Property = Object->GetClass()->FindPropertyByName(FName(*PropertyName));
    if (!Property)
    {
        UE_LOG(LogTemp, Error, TEXT("ApplyPropertyToObject: Property '%s' not found in object of class '%s'"), 
            *PropertyName, *Object->GetClass()->GetName());
        return false;
    }
    
    // Get a pointer to the property memory in the object
    void* PropertyPtr = Property->ContainerPtrToValuePtr<void>(Object);
    
    // Apply the property value to the property
    return ApplyPropertyValueToProperty(Property, PropertyPtr, PropValue);
}

// Extract a property value from a UObject property
bool USpacetimeDBPropertyHandler::ExtractPropertyFromObject(UObject* Object, const FString& PropertyName, FSpacetimeDBPropertyValue& OutPropValue)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("ExtractPropertyFromObject: Object is null"));
        return false;
    }
    
    // Find the property in the object
    FProperty* Property = Object->GetClass()->FindPropertyByName(FName(*PropertyName));
    if (!Property)
    {
        UE_LOG(LogTemp, Error, TEXT("ExtractPropertyFromObject: Property '%s' not found in object of class '%s'"), 
            *PropertyName, *Object->GetClass()->GetName());
        return false;
    }
    
    // Get a pointer to the property memory in the object
    const void* PropertyPtr = Property->ContainerPtrToValuePtr<void>(Object);
    
    // Extract the property value from the property
    return ExtractPropertyValueFromProperty(Property, PropertyPtr, OutPropValue);
}

// Handle a property update from the server
bool USpacetimeDBPropertyHandler::HandlePropertyUpdate(int64 ObjectId, const FString& PropertyName, const FString& ValueJson)
{
    // Get the SpacetimeDB client
    USpacetimeDBClient* Client = USpacetimeDBClient::Get();
    if (!Client)
    {
        UE_LOG(LogTemp, Error, TEXT("HandlePropertyUpdate: SpacetimeDBClient is null"));
        return false;
    }
    
    // Find the object in our map
    UObject* Object = Client->GetObjectById(FSpacetimeDBObjectID(ObjectId));
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("HandlePropertyUpdate: Object with ID %lld not found"), ObjectId);
        return false;
    }
    
    // Parse the property value from the JSON string
    FSpacetimeDBPropertyValue PropValue = FSpacetimeDBPropertyValue::FromJsonString(ValueJson);
    
    // Apply the property value to the object
    return ApplyPropertyToObject(Object, PropertyName, PropValue);
}

// Apply a property value to a specific UProperty
bool USpacetimeDBPropertyHandler::ApplyPropertyValueToProperty(FProperty* Property, void* PropertyPtr, const FSpacetimeDBPropertyValue& PropValue)
{
    if (!Property || !PropertyPtr)
    {
        UE_LOG(LogTemp, Error, TEXT("ApplyPropertyValueToProperty: Property or PropertyPtr is null"));
        return false;
    }
    
    // Handle special cases based on property type
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Bool)
        {
            BoolProperty->SetPropertyValue(PropertyPtr, PropValue.BoolValue);
            return true;
        }
    }
    else if (FByteProperty* ByteProperty = CastField<FByteProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Byte)
        {
            ByteProperty->SetPropertyValue(PropertyPtr, PropValue.ByteValue);
            return true;
        }
    }
    else if (FIntProperty* IntProperty = CastField<FIntProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Int32)
        {
            IntProperty->SetPropertyValue(PropertyPtr, PropValue.Int32Value);
            return true;
        }
    }
    else if (FInt64Property* Int64Property = CastField<FInt64Property>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Int64)
        {
            Int64Property->SetPropertyValue(PropertyPtr, PropValue.Int64Value);
            return true;
        }
    }
    else if (FUInt32Property* UInt32Property = CastField<FUInt32Property>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::UInt32)
        {
            UInt32Property->SetPropertyValue(PropertyPtr, PropValue.UInt32Value);
            return true;
        }
    }
    else if (FUInt64Property* UInt64Property = CastField<FUInt64Property>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::UInt64)
        {
            UInt64Property->SetPropertyValue(PropertyPtr, PropValue.UInt64Value);
            return true;
        }
    }
    else if (FFloatProperty* FloatProperty = CastField<FFloatProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Float)
        {
            FloatProperty->SetPropertyValue(PropertyPtr, PropValue.FloatValue);
            return true;
        }
    }
    else if (FDoubleProperty* DoubleProperty = CastField<FDoubleProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Double)
        {
            DoubleProperty->SetPropertyValue(PropertyPtr, PropValue.DoubleValue);
            return true;
        }
    }
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::String)
        {
            StringProperty->SetPropertyValue(PropertyPtr, PropValue.StringValue);
            return true;
        }
    }
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Name)
        {
            NameProperty->SetPropertyValue(PropertyPtr, FName(*PropValue.StringValue));
            return true;
        }
    }
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Text)
        {
            TextProperty->SetPropertyValue(PropertyPtr, FText::FromString(PropValue.StringValue));
            return true;
        }
    }
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        // Handle structured types
        if (StructProperty->Struct->GetFName() == "Vector" && PropValue.Type == ESpacetimeDBPropertyType::Vector)
        {
            FVector* VectorPtr = static_cast<FVector*>(PropertyPtr);
            *VectorPtr = PropValue.VectorValue;
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Rotator" && PropValue.Type == ESpacetimeDBPropertyType::Rotator)
        {
            FRotator* RotatorPtr = static_cast<FRotator*>(PropertyPtr);
            *RotatorPtr = PropValue.RotatorValue;
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Quat" && PropValue.Type == ESpacetimeDBPropertyType::Quat)
        {
            FQuat* QuatPtr = static_cast<FQuat*>(PropertyPtr);
            *QuatPtr = PropValue.QuatValue;
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Transform" && PropValue.Type == ESpacetimeDBPropertyType::Transform)
        {
            FTransform* TransformPtr = static_cast<FTransform*>(PropertyPtr);
            *TransformPtr = PropValue.TransformValue;
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Color" && PropValue.Type == ESpacetimeDBPropertyType::Color)
        {
            FColor* ColorPtr = static_cast<FColor*>(PropertyPtr);
            *ColorPtr = PropValue.ColorValue;
            return true;
        }
        else if (PropValue.Type == ESpacetimeDBPropertyType::Custom)
        {
            // Use JSON to deserialize the custom struct
            return USpacetimeDBJsonUtils::DeserializeJsonToStruct(StructProperty->Struct, PropertyPtr, PropValue.JsonValue);
        }
    }
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::ObjectReference)
        {
            // Get the object by ID
            USpacetimeDBClient* Client = USpacetimeDBClient::Get();
            if (Client)
            {
                UObject* ReferencedObject = Client->GetObjectById(PropValue.ObjectReferenceValue);
                if (ReferencedObject && ReferencedObject->IsA(ObjectProperty->PropertyClass))
                {
                    ObjectProperty->SetObjectPropertyValue(PropertyPtr, ReferencedObject);
                    return true;
                }
            }
        }
    }
    else if (FClassProperty* ClassProperty = CastField<FClassProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::ClassReference)
        {
            // Find the class by name
            UClass* Class = FindObject<UClass>(ANY_PACKAGE, *PropValue.StringValue);
            if (Class && Class->IsChildOf(ClassProperty->MetaClass))
            {
                ClassProperty->SetPropertyValue(PropertyPtr, Class);
                return true;
            }
        }
    }
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Array)
        {
            // Use JSON to deserialize the array
            return USpacetimeDBJsonUtils::DeserializeJsonToArray(ArrayProperty, PropertyPtr, PropValue.JsonValue);
        }
    }
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Map)
        {
            // Use JSON to deserialize the map
            return USpacetimeDBJsonUtils::DeserializeJsonToMap(MapProperty, PropertyPtr, PropValue.JsonValue);
        }
    }
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        if (PropValue.Type == ESpacetimeDBPropertyType::Set)
        {
            // Use JSON to deserialize the set
            return USpacetimeDBJsonUtils::DeserializeJsonToSet(SetProperty, PropertyPtr, PropValue.JsonValue);
        }
    }
    
    // If we get here, the property type doesn't match or isn't supported
    UE_LOG(LogTemp, Error, TEXT("ApplyPropertyValueToProperty: Unsupported property type combination. Property: %s, PropValue type: %d"),
        *Property->GetName(), static_cast<int32>(PropValue.Type));
    return false;
}

// Extract a property value from a specific UProperty
bool USpacetimeDBPropertyHandler::ExtractPropertyValueFromProperty(FProperty* Property, const void* PropertyPtr, FSpacetimeDBPropertyValue& OutPropValue)
{
    if (!Property || !PropertyPtr)
    {
        UE_LOG(LogTemp, Error, TEXT("ExtractPropertyValueFromProperty: Property or PropertyPtr is null"));
        return false;
    }
    
    // Handle special cases based on property type
    if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(BoolProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FByteProperty* ByteProperty = CastField<FByteProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(ByteProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FIntProperty* IntProperty = CastField<FIntProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(IntProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FInt64Property* Int64Property = CastField<FInt64Property>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(Int64Property->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FUInt32Property* UInt32Property = CastField<FUInt32Property>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(UInt32Property->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FUInt64Property* UInt64Property = CastField<FUInt64Property>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(UInt64Property->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FFloatProperty* FloatProperty = CastField<FFloatProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(FloatProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FDoubleProperty* DoubleProperty = CastField<FDoubleProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(DoubleProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
    {
        OutPropValue = FSpacetimeDBPropertyValue(StringProperty->GetPropertyValue(PropertyPtr));
        return true;
    }
    else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
    {
        FName NameValue = NameProperty->GetPropertyValue(PropertyPtr);
        OutPropValue = FSpacetimeDBPropertyValue::MakeName(NameValue.ToString());
        return true;
    }
    else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
    {
        FText TextValue = TextProperty->GetPropertyValue(PropertyPtr);
        OutPropValue = FSpacetimeDBPropertyValue::MakeText(TextValue.ToString());
        return true;
    }
    else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
    {
        // Handle structured types
        if (StructProperty->Struct->GetFName() == "Vector")
        {
            const FVector* VectorPtr = static_cast<const FVector*>(PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue(*VectorPtr);
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Rotator")
        {
            const FRotator* RotatorPtr = static_cast<const FRotator*>(PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue(*RotatorPtr);
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Quat")
        {
            const FQuat* QuatPtr = static_cast<const FQuat*>(PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue(*QuatPtr);
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Transform")
        {
            const FTransform* TransformPtr = static_cast<const FTransform*>(PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue(*TransformPtr);
            return true;
        }
        else if (StructProperty->Struct->GetFName() == "Color")
        {
            const FColor* ColorPtr = static_cast<const FColor*>(PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue(*ColorPtr);
            return true;
        }
        else
        {
            // Use JSON to serialize the custom struct
            FString JsonString = USpacetimeDBJsonUtils::SerializeStructToJson(StructProperty->Struct, PropertyPtr);
            OutPropValue = FSpacetimeDBPropertyValue::MakeCustomJson(JsonString);
            return true;
        }
    }
    else if (FObjectProperty* ObjectProperty = CastField<FObjectProperty>(Property))
    {
        UObject* ReferencedObject = ObjectProperty->GetObjectPropertyValue(PropertyPtr);
        if (ReferencedObject)
        {
            // Get the object ID
            USpacetimeDBClient* Client = USpacetimeDBClient::Get();
            if (Client)
            {
                FSpacetimeDBObjectID ObjectId = Client->GetObjectId(ReferencedObject);
                if (ObjectId.Value != 0) // 0 is invalid/not found
                {
                    OutPropValue = FSpacetimeDBPropertyValue(ObjectId);
                    return true;
                }
            }
        }
        // If we couldn't get the object ID, use None
        OutPropValue.Type = ESpacetimeDBPropertyType::None;
        return true;
    }
    else if (FClassProperty* ClassProperty = CastField<FClassProperty>(Property))
    {
        UClass* Class = Cast<UClass>(ClassProperty->GetObjectPropertyValue(PropertyPtr));
        if (Class)
        {
            OutPropValue = FSpacetimeDBPropertyValue::MakeClassReference(Class->GetPathName());
            return true;
        }
        // If the class is null, use None
        OutPropValue.Type = ESpacetimeDBPropertyType::None;
        return true;
    }
    else if (FArrayProperty* ArrayProperty = CastField<FArrayProperty>(Property))
    {
        // Use JSON to serialize the array
        FString JsonString = USpacetimeDBJsonUtils::SerializeArrayToJson(ArrayProperty, PropertyPtr);
        OutPropValue = FSpacetimeDBPropertyValue::MakeArrayJson(JsonString);
        return true;
    }
    else if (FMapProperty* MapProperty = CastField<FMapProperty>(Property))
    {
        // Use JSON to serialize the map
        FString JsonString = USpacetimeDBJsonUtils::SerializeMapToJson(MapProperty, PropertyPtr);
        OutPropValue = FSpacetimeDBPropertyValue::MakeMapJson(JsonString);
        return true;
    }
    else if (FSetProperty* SetProperty = CastField<FSetProperty>(Property))
    {
        // Use JSON to serialize the set
        FString JsonString = USpacetimeDBJsonUtils::SerializeSetToJson(SetProperty, PropertyPtr);
        OutPropValue = FSpacetimeDBPropertyValue::MakeSetJson(JsonString);
        return true;
    }
    
    // If we get here, the property type isn't supported
    UE_LOG(LogTemp, Error, TEXT("ExtractPropertyValueFromProperty: Unsupported property type: %s"), *Property->GetClass()->GetName());
    OutPropValue.Type = ESpacetimeDBPropertyType::None;
    return false;
} 