#include "SpacetimeDBPropertyHelper.h"
#include "Engine/Engine.h"
#include "JsonObjectConverter.h"
#include "UObject/UnrealType.h"
#include "UObject/PropertyPortFlags.h"

bool FSpacetimeDBPropertyHelper::ApplyJsonToProperty(UObject* Object, const FString& PropertyName, const FString& ValueJson)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Cannot apply property update to null object. Property: %s"), *PropertyName);
        return false;
    }

    FProperty* Property = Object->GetClass()->FindPropertyByName(FName(*PropertyName));
    if (!Property)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Property not found: %s on object %s"), 
            *PropertyName, *Object->GetName());
        return false;
    }

    // Parse the JSON value
    TSharedPtr<FJsonValue> JsonValue;
    TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(ValueJson);
    if (!FJsonSerializer::Deserialize(Reader, JsonValue) || !JsonValue.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Failed to parse JSON for property %s: %s"), 
            *PropertyName, *ValueJson);
        return false;
    }

    // Get the address of the property in the object
    void* PropertyAddress = Property->ContainerPtrToValuePtr<void>(Object);
    bool bSuccess = false;

    // Handle different property types
    if (FNumericProperty* NumericProp = CastField<FNumericProperty>(Property))
    {
        bSuccess = DeserializeAndApplyNumericProperty(NumericProp, PropertyAddress, JsonValue);
    }
    else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(Property))
    {
        bSuccess = DeserializeAndApplyBoolProperty(BoolProp, PropertyAddress, JsonValue);
    }
    else if (FStrProperty* StrProp = CastField<FStrProperty>(Property))
    {
        bSuccess = DeserializeAndApplyStrProperty(StrProp, PropertyAddress, JsonValue);
    }
    else if (FTextProperty* TextProp = CastField<FTextProperty>(Property))
    {
        bSuccess = DeserializeAndApplyTextProperty(TextProp, PropertyAddress, JsonValue);
    }
    else if (FNameProperty* NameProp = CastField<FNameProperty>(Property))
    {
        bSuccess = DeserializeAndApplyNameProperty(NameProp, PropertyAddress, JsonValue);
    }
    else if (FStructProperty* StructProp = CastField<FStructProperty>(Property))
    {
        bSuccess = DeserializeAndApplyStructProperty(StructProp, PropertyAddress, JsonValue);
    }
    else if (FArrayProperty* ArrayProp = CastField<FArrayProperty>(Property))
    {
        bSuccess = DeserializeAndApplyArrayProperty(ArrayProp, PropertyAddress, JsonValue);
    }
    else if (FMapProperty* MapProp = CastField<FMapProperty>(Property))
    {
        bSuccess = DeserializeAndApplyMapProperty(MapProp, PropertyAddress, JsonValue);
    }
    else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(Property))
    {
        bSuccess = DeserializeAndApplyObjectProperty(ObjProp, PropertyAddress, JsonValue);
    }
    else if (FSoftObjectProperty* SoftObjProp = CastField<FSoftObjectProperty>(Property))
    {
        bSuccess = DeserializeAndApplySoftObjectProperty(SoftObjProp, PropertyAddress, JsonValue);
    }
    else if (FEnumProperty* EnumProp = CastField<FEnumProperty>(Property))
    {
        bSuccess = DeserializeAndApplyEnumProperty(EnumProp, PropertyAddress, JsonValue);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDB: Unsupported property type for %s"), *PropertyName);
        return false;
    }

    // Fire RepNotify if available
    if (bSuccess && Property->HasAnyPropertyFlags(CPF_RepNotify))
    {
        FName RepNotifyFuncName = Property->RepNotifyFunc;
        if (RepNotifyFuncName != NAME_None)
        {
            UFunction* RepNotifyFunc = Object->GetClass()->FindFunctionByName(RepNotifyFuncName);
            if (RepNotifyFunc)
            {
                // Check if the RepNotify function takes a parameter
                if (RepNotifyFunc->NumParms > 0)
                {
                    // Create a buffer for the parameter
                    uint8* Buffer = (uint8*)FMemory::Malloc(RepNotifyFunc->ParmsSize);
                    FMemory::Memzero(Buffer, RepNotifyFunc->ParmsSize);
                    
                    // Copy the property value to the parameter
                    for (TFieldIterator<FProperty> It(RepNotifyFunc); It && It->HasAnyPropertyFlags(CPF_Parm) && !It->HasAnyPropertyFlags(CPF_ReturnParm); ++It)
                    {
                        void* Parm = It->ContainerPtrToValuePtr<void>(Buffer);
                        It->CopyCompleteValue(Parm, PropertyAddress);
                        break; // Only copy the first parameter
                    }
                    
                    // Call the function
                    Object->ProcessEvent(RepNotifyFunc, Buffer);
                    
                    // Clean up
                    FMemory::Free(Buffer);
                }
                else
                {
                    // Call the function without parameters
                    Object->ProcessEvent(RepNotifyFunc, nullptr);
                }
            }
        }
    }

    return bSuccess;
}

FString FSpacetimeDBPropertyHelper::SerializePropertyToJson(UObject* Object, const FString& PropertyName)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Cannot serialize property from null object. Property: %s"), *PropertyName);
        return FString();
    }

    FProperty* Property = Object->GetClass()->FindPropertyByName(FName(*PropertyName));
    if (!Property)
    {
        UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Property not found for serialization: %s on object %s"), 
            *PropertyName, *Object->GetName());
        return FString();
    }

    // Get the address of the property in the object
    const void* PropertyAddress = Property->ContainerPtrToValuePtr<void>(Object);
    TSharedPtr<FJsonValue> JsonValue;

    // Handle different property types
    if (FNumericProperty* NumericProp = CastField<FNumericProperty>(Property))
    {
        JsonValue = SerializeNumericProperty(NumericProp, PropertyAddress);
    }
    else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(Property))
    {
        JsonValue = SerializeBoolProperty(BoolProp, PropertyAddress);
    }
    else if (FStrProperty* StrProp = CastField<FStrProperty>(Property))
    {
        JsonValue = SerializeStrProperty(StrProp, PropertyAddress);
    }
    else if (FTextProperty* TextProp = CastField<FTextProperty>(Property))
    {
        JsonValue = SerializeTextProperty(TextProp, PropertyAddress);
    }
    else if (FNameProperty* NameProp = CastField<FNameProperty>(Property))
    {
        JsonValue = SerializeNameProperty(NameProp, PropertyAddress);
    }
    else if (FStructProperty* StructProp = CastField<FStructProperty>(Property))
    {
        JsonValue = SerializeStructProperty(StructProp, PropertyAddress);
    }
    else if (FArrayProperty* ArrayProp = CastField<FArrayProperty>(Property))
    {
        JsonValue = SerializeArrayProperty(ArrayProp, PropertyAddress);
    }
    else if (FMapProperty* MapProp = CastField<FMapProperty>(Property))
    {
        JsonValue = SerializeMapProperty(MapProp, PropertyAddress);
    }
    else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(Property))
    {
        JsonValue = SerializeObjectProperty(ObjProp, PropertyAddress);
    }
    else if (FSoftObjectProperty* SoftObjProp = CastField<FSoftObjectProperty>(Property))
    {
        JsonValue = SerializeSoftObjectProperty(SoftObjProp, PropertyAddress);
    }
    else if (FEnumProperty* EnumProp = CastField<FEnumProperty>(Property))
    {
        JsonValue = SerializeEnumProperty(EnumProp, PropertyAddress);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("SpacetimeDB: Unsupported property type for serialization: %s"), *PropertyName);
        return FString();
    }

    if (!JsonValue.IsValid())
    {
        return FString();
    }

    // Serialize to string
    FString ResultString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&ResultString);
    FJsonSerializer::Serialize(JsonValue.ToSharedRef(), Writer);
    return ResultString;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyNumericProperty(FNumericProperty* NumericProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    if (NumericProp->IsFloatingPoint())
    {
        double Value = 0.0;
        if (JsonValue->TryGetNumber(Value))
        {
            NumericProp->SetFloatingPointPropertyValue(PropAddr, Value);
            return true;
        }
    }
    else // Integer property
    {
        int64 Value = 0;
        if (JsonValue->TryGetNumber(Value))
        {
            NumericProp->SetIntPropertyValue(PropAddr, Value);
            return true;
        }
    }
    return false;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyBoolProperty(FBoolProperty* BoolProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    bool Value = false;
    if (JsonValue->TryGetBool(Value))
    {
        BoolProp->SetPropertyValue(PropAddr, Value);
        return true;
    }
    return false;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyStrProperty(FStrProperty* StrProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    FString Value;
    if (JsonValue->TryGetString(Value))
    {
        StrProp->SetPropertyValue(PropAddr, Value);
        return true;
    }
    return false;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyTextProperty(FTextProperty* TextProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    FString StringValue;
    if (JsonValue->TryGetString(StringValue))
    {
        TextProp->SetPropertyValue(PropAddr, FText::FromString(StringValue));
        return true;
    }
    return false;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyNameProperty(FNameProperty* NameProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    FString StringValue;
    if (JsonValue->TryGetString(StringValue))
    {
        NameProp->SetPropertyValue(PropAddr, FName(*StringValue));
        return true;
    }
    return false;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyStructProperty(FStructProperty* StructProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    const TSharedPtr<FJsonObject>* JsonObject = nullptr;
    if (!JsonValue->TryGetObject(JsonObject) || !JsonObject)
    {
        return false;
    }

    // Special handling for known UE types
    UScriptStruct* Struct = StructProp->Struct;
    const FString StructName = Struct->GetName();

    // Specialized handling for common UE structs
    if (StructName == TEXT("Vector"))
    {
        FVector& Vector = *reinterpret_cast<FVector*>(PropAddr);
        (*JsonObject)->TryGetNumberField(TEXT("X"), Vector.X);
        (*JsonObject)->TryGetNumberField(TEXT("Y"), Vector.Y);
        (*JsonObject)->TryGetNumberField(TEXT("Z"), Vector.Z);
        return true;
    }
    else if (StructName == TEXT("Rotator"))
    {
        FRotator& Rotator = *reinterpret_cast<FRotator*>(PropAddr);
        (*JsonObject)->TryGetNumberField(TEXT("Pitch"), Rotator.Pitch);
        (*JsonObject)->TryGetNumberField(TEXT("Yaw"), Rotator.Yaw);
        (*JsonObject)->TryGetNumberField(TEXT("Roll"), Rotator.Roll);
        return true;
    }
    else if (StructName == TEXT("Transform"))
    {
        FTransform& Transform = *reinterpret_cast<FTransform*>(PropAddr);
        
        const TSharedPtr<FJsonObject>* LocationObj;
        if ((*JsonObject)->TryGetObjectField(TEXT("Location"), LocationObj))
        {
            FVector Location = Transform.GetLocation();
            (*LocationObj)->TryGetNumberField(TEXT("X"), Location.X);
            (*LocationObj)->TryGetNumberField(TEXT("Y"), Location.Y);
            (*LocationObj)->TryGetNumberField(TEXT("Z"), Location.Z);
            Transform.SetLocation(Location);
        }
        
        const TSharedPtr<FJsonObject>* RotationObj;
        if ((*JsonObject)->TryGetObjectField(TEXT("Rotation"), RotationObj))
        {
            FQuat Rotation = Transform.GetRotation();
            double X, Y, Z, W;
            if ((*RotationObj)->TryGetNumberField(TEXT("X"), X) &&
                (*RotationObj)->TryGetNumberField(TEXT("Y"), Y) &&
                (*RotationObj)->TryGetNumberField(TEXT("Z"), Z) &&
                (*RotationObj)->TryGetNumberField(TEXT("W"), W))
            {
                Rotation.X = X;
                Rotation.Y = Y;
                Rotation.Z = Z;
                Rotation.W = W;
                Transform.SetRotation(Rotation);
            }
        }
        
        const TSharedPtr<FJsonObject>* ScaleObj;
        if ((*JsonObject)->TryGetObjectField(TEXT("Scale"), ScaleObj))
        {
            FVector Scale = Transform.GetScale3D();
            (*ScaleObj)->TryGetNumberField(TEXT("X"), Scale.X);
            (*ScaleObj)->TryGetNumberField(TEXT("Y"), Scale.Y);
            (*ScaleObj)->TryGetNumberField(TEXT("Z"), Scale.Z);
            Transform.SetScale3D(Scale);
        }
        
        return true;
    }
    else if (StructName == TEXT("LinearColor"))
    {
        FLinearColor& Color = *reinterpret_cast<FLinearColor*>(PropAddr);
        (*JsonObject)->TryGetNumberField(TEXT("R"), Color.R);
        (*JsonObject)->TryGetNumberField(TEXT("G"), Color.G);
        (*JsonObject)->TryGetNumberField(TEXT("B"), Color.B);
        (*JsonObject)->TryGetNumberField(TEXT("A"), Color.A);
        return true;
    }
    else
    {
        // Generic handling for other structs using UE's built-in converter
        return FJsonObjectConverter::JsonObjectToUStruct((*JsonObject).ToSharedRef(), Struct, PropAddr, 0, 0);
    }
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyArrayProperty(FArrayProperty* ArrayProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    const TArray<TSharedPtr<FJsonValue>>* JsonArray;
    if (!JsonValue->TryGetArray(JsonArray))
    {
        return false;
    }

    // Create a helper to get the native array
    FScriptArrayHelper ArrayHelper(ArrayProp, PropAddr);
    
    // Reset the array and resize to match the JSON array
    ArrayHelper.EmptyValues();
    ArrayHelper.Resize(JsonArray->Num());
    
    // Get the inner property
    FProperty* InnerProp = ArrayProp->Inner;
    
    // Iterate through JSON array and apply values to array elements
    for (int32 i = 0; i < JsonArray->Num(); i++)
    {
        void* ElemPtr = ArrayHelper.GetRawPtr(i);
        
        // Handle different inner property types
        if (FNumericProperty* NumericProp = CastField<FNumericProperty>(InnerProp))
        {
            DeserializeAndApplyNumericProperty(NumericProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(InnerProp))
        {
            DeserializeAndApplyBoolProperty(BoolProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FStrProperty* StrProp = CastField<FStrProperty>(InnerProp))
        {
            DeserializeAndApplyStrProperty(StrProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FTextProperty* TextProp = CastField<FTextProperty>(InnerProp))
        {
            DeserializeAndApplyTextProperty(TextProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FNameProperty* NameProp = CastField<FNameProperty>(InnerProp))
        {
            DeserializeAndApplyNameProperty(NameProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FStructProperty* StructProp = CastField<FStructProperty>(InnerProp))
        {
            DeserializeAndApplyStructProperty(StructProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FArrayProperty* NestedArrayProp = CastField<FArrayProperty>(InnerProp))
        {
            DeserializeAndApplyArrayProperty(NestedArrayProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FMapProperty* MapProp = CastField<FMapProperty>(InnerProp))
        {
            DeserializeAndApplyMapProperty(MapProp, ElemPtr, (*JsonArray)[i]);
        }
        else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(InnerProp))
        {
            DeserializeAndApplyObjectProperty(ObjProp, ElemPtr, (*JsonArray)[i]);
        }
        // Add other property types as needed
    }
    
    return true;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyMapProperty(FMapProperty* MapProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    const TSharedPtr<FJsonObject>* JsonObject;
    if (!JsonValue->TryGetObject(JsonObject) || !JsonObject)
    {
        return false;
    }

    // Get the key and value properties
    FProperty* KeyProp = MapProp->KeyProp;
    FProperty* ValueProp = MapProp->ValueProp;
    
    // Create a helper to get the native map
    FScriptMapHelper MapHelper(MapProp, PropAddr);
    
    // Empty the map
    MapHelper.EmptyValues();
    
    // Iterate through JSON object and apply values to map elements
    for (const auto& Pair : (*JsonObject)->Values)
    {
        // Add an element
        int32 Index = MapHelper.AddDefaultValue_Invalid_NeedsRehash();
        
        // Get key and value pointers
        uint8* KeyPtr = MapHelper.GetKeyPtr(Index);
        uint8* ValuePtr = MapHelper.GetValuePtr(Index);
        
        // Handle key - usually a string or number
        if (FStrProperty* StrKeyProp = CastField<FStrProperty>(KeyProp))
        {
            StrKeyProp->SetPropertyValue(KeyPtr, Pair.Key);
        }
        else if (FNameProperty* NameKeyProp = CastField<FNameProperty>(KeyProp))
        {
            NameKeyProp->SetPropertyValue(KeyPtr, FName(*Pair.Key));
        }
        else if (FNumericProperty* NumericKeyProp = CastField<FNumericProperty>(KeyProp))
        {
            double KeyValue = FCString::Atod(*Pair.Key);
            NumericKeyProp->SetFloatingPointPropertyValue(KeyPtr, KeyValue);
        }
        
        // Handle value based on property type
        if (FNumericProperty* NumericProp = CastField<FNumericProperty>(ValueProp))
        {
            DeserializeAndApplyNumericProperty(NumericProp, ValuePtr, Pair.Value);
        }
        else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(ValueProp))
        {
            DeserializeAndApplyBoolProperty(BoolProp, ValuePtr, Pair.Value);
        }
        else if (FStrProperty* StrProp = CastField<FStrProperty>(ValueProp))
        {
            DeserializeAndApplyStrProperty(StrProp, ValuePtr, Pair.Value);
        }
        else if (FTextProperty* TextProp = CastField<FTextProperty>(ValueProp))
        {
            DeserializeAndApplyTextProperty(TextProp, ValuePtr, Pair.Value);
        }
        else if (FNameProperty* NameProp = CastField<FNameProperty>(ValueProp))
        {
            DeserializeAndApplyNameProperty(NameProp, ValuePtr, Pair.Value);
        }
        else if (FStructProperty* StructProp = CastField<FStructProperty>(ValueProp))
        {
            DeserializeAndApplyStructProperty(StructProp, ValuePtr, Pair.Value);
        }
        else if (FArrayProperty* ArrayProp = CastField<FArrayProperty>(ValueProp))
        {
            DeserializeAndApplyArrayProperty(ArrayProp, ValuePtr, Pair.Value);
        }
        else if (FMapProperty* NestedMapProp = CastField<FMapProperty>(ValueProp))
        {
            DeserializeAndApplyMapProperty(NestedMapProp, ValuePtr, Pair.Value);
        }
        else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(ValueProp))
        {
            DeserializeAndApplyObjectProperty(ObjProp, ValuePtr, Pair.Value);
        }
        // Add other property types as needed
    }
    
    // Rehash the map
    MapHelper.Rehash();
    
    return true;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyObjectProperty(FObjectProperty* ObjProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    FString ObjectPath;
    if (!JsonValue->TryGetString(ObjectPath))
    {
        return false;
    }
    
    // Try to find the object by path
    UObject* FoundObject = nullptr;
    if (!ObjectPath.IsEmpty())
    {
        FoundObject = FindObject<UObject>(nullptr, *ObjectPath);
        if (!FoundObject)
        {
            // Try to load the object if it wasn't found
            FoundObject = StaticLoadObject(ObjProp->PropertyClass, nullptr, *ObjectPath);
        }
    }
    
    // Set the property value
    ObjProp->SetObjectPropertyValue(PropAddr, FoundObject);
    return true;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplySoftObjectProperty(FSoftObjectProperty* SoftObjProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    FString ObjectPath;
    if (!JsonValue->TryGetString(ObjectPath))
    {
        return false;
    }
    
    // Create soft object path
    FSoftObjectPath SoftObjectPath(ObjectPath);
    FSoftObjectPtr& SoftObjectPtr = *reinterpret_cast<FSoftObjectPtr*>(PropAddr);
    SoftObjectPtr = SoftObjectPath;
    
    return true;
}

bool FSpacetimeDBPropertyHelper::DeserializeAndApplyEnumProperty(FEnumProperty* EnumProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue)
{
    // Try to get enum value as integer
    int64 IntValue;
    if (JsonValue->TryGetNumber(IntValue))
    {
        // Use the numeric property for storage
        FNumericProperty* UnderlyingProp = EnumProp->GetUnderlyingProperty();
        UnderlyingProp->SetIntPropertyValue(PropAddr, IntValue);
        return true;
    }
    
    // Try to get enum value as string (by name)
    FString StringValue;
    if (JsonValue->TryGetString(StringValue))
    {
        UEnum* Enum = EnumProp->GetEnum();
        int64 EnumValue = Enum->GetValueByName(FName(*StringValue));
        if (EnumValue != INDEX_NONE)
        {
            FNumericProperty* UnderlyingProp = EnumProp->GetUnderlyingProperty();
            UnderlyingProp->SetIntPropertyValue(PropAddr, EnumValue);
            return true;
        }
    }
    
    return false;
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeNumericProperty(FNumericProperty* NumericProp, const void* PropAddr)
{
    if (NumericProp->IsFloatingPoint())
    {
        double Value = NumericProp->GetFloatingPointPropertyValue(PropAddr);
        return MakeShared<FJsonValueNumber>(Value);
    }
    else // Integer property
    {
        int64 Value = NumericProp->GetSignedIntPropertyValue(PropAddr);
        return MakeShared<FJsonValueNumber>(Value);
    }
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeBoolProperty(FBoolProperty* BoolProp, const void* PropAddr)
{
    bool Value = BoolProp->GetPropertyValue(PropAddr);
    return MakeShared<FJsonValueBoolean>(Value);
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeStrProperty(FStrProperty* StrProp, const void* PropAddr)
{
    FString Value = StrProp->GetPropertyValue(PropAddr);
    return MakeShared<FJsonValueString>(Value);
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeTextProperty(FTextProperty* TextProp, const void* PropAddr)
{
    FText Value = TextProp->GetPropertyValue(PropAddr);
    return MakeShared<FJsonValueString>(Value.ToString());
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeNameProperty(FNameProperty* NameProp, const void* PropAddr)
{
    FName Value = NameProp->GetPropertyValue(PropAddr);
    return MakeShared<FJsonValueString>(Value.ToString());
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeStructProperty(FStructProperty* StructProp, const void* PropAddr)
{
    // Special handling for known UE types
    UScriptStruct* Struct = StructProp->Struct;
    const FString StructName = Struct->GetName();

    TSharedPtr<FJsonObject> JsonObject = MakeShared<FJsonObject>();
    
    // Specialized handling for common UE structs
    if (StructName == TEXT("Vector"))
    {
        const FVector& Vector = *reinterpret_cast<const FVector*>(PropAddr);
        JsonObject->SetNumberField(TEXT("X"), Vector.X);
        JsonObject->SetNumberField(TEXT("Y"), Vector.Y);
        JsonObject->SetNumberField(TEXT("Z"), Vector.Z);
    }
    else if (StructName == TEXT("Rotator"))
    {
        const FRotator& Rotator = *reinterpret_cast<const FRotator*>(PropAddr);
        JsonObject->SetNumberField(TEXT("Pitch"), Rotator.Pitch);
        JsonObject->SetNumberField(TEXT("Yaw"), Rotator.Yaw);
        JsonObject->SetNumberField(TEXT("Roll"), Rotator.Roll);
    }
    else if (StructName == TEXT("Transform"))
    {
        const FTransform& Transform = *reinterpret_cast<const FTransform*>(PropAddr);
        
        // Location
        TSharedPtr<FJsonObject> LocationObj = MakeShared<FJsonObject>();
        FVector Location = Transform.GetLocation();
        LocationObj->SetNumberField(TEXT("X"), Location.X);
        LocationObj->SetNumberField(TEXT("Y"), Location.Y);
        LocationObj->SetNumberField(TEXT("Z"), Location.Z);
        JsonObject->SetObjectField(TEXT("Location"), LocationObj);
        
        // Rotation (as quaternion)
        TSharedPtr<FJsonObject> RotationObj = MakeShared<FJsonObject>();
        FQuat Rotation = Transform.GetRotation();
        RotationObj->SetNumberField(TEXT("X"), Rotation.X);
        RotationObj->SetNumberField(TEXT("Y"), Rotation.Y);
        RotationObj->SetNumberField(TEXT("Z"), Rotation.Z);
        RotationObj->SetNumberField(TEXT("W"), Rotation.W);
        JsonObject->SetObjectField(TEXT("Rotation"), RotationObj);
        
        // Scale
        TSharedPtr<FJsonObject> ScaleObj = MakeShared<FJsonObject>();
        FVector Scale = Transform.GetScale3D();
        ScaleObj->SetNumberField(TEXT("X"), Scale.X);
        ScaleObj->SetNumberField(TEXT("Y"), Scale.Y);
        ScaleObj->SetNumberField(TEXT("Z"), Scale.Z);
        JsonObject->SetObjectField(TEXT("Scale"), ScaleObj);
    }
    else if (StructName == TEXT("LinearColor"))
    {
        const FLinearColor& Color = *reinterpret_cast<const FLinearColor*>(PropAddr);
        JsonObject->SetNumberField(TEXT("R"), Color.R);
        JsonObject->SetNumberField(TEXT("G"), Color.G);
        JsonObject->SetNumberField(TEXT("B"), Color.B);
        JsonObject->SetNumberField(TEXT("A"), Color.A);
    }
    else
    {
        // Generic handling for other structs
        FJsonObjectConverter::UStructToJsonObject(Struct, PropAddr, JsonObject.ToSharedRef(), 0, 0);
    }
    
    return MakeShared<FJsonValueObject>(JsonObject);
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeArrayProperty(FArrayProperty* ArrayProp, const void* PropAddr)
{
    // Create a helper to get the native array
    FScriptArrayHelper ArrayHelper(ArrayProp, PropAddr);
    
    // Get the inner property
    FProperty* InnerProp = ArrayProp->Inner;
    
    // Create JSON array
    TArray<TSharedPtr<FJsonValue>> JsonArray;
    JsonArray.SetNum(ArrayHelper.Num());
    
    // Iterate through array elements and serialize to JSON
    for (int32 i = 0; i < ArrayHelper.Num(); i++)
    {
        const void* ElemPtr = ArrayHelper.GetRawPtr(i);
        
        // Handle different inner property types
        if (FNumericProperty* NumericProp = CastField<FNumericProperty>(InnerProp))
        {
            JsonArray[i] = SerializeNumericProperty(NumericProp, ElemPtr);
        }
        else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(InnerProp))
        {
            JsonArray[i] = SerializeBoolProperty(BoolProp, ElemPtr);
        }
        else if (FStrProperty* StrProp = CastField<FStrProperty>(InnerProp))
        {
            JsonArray[i] = SerializeStrProperty(StrProp, ElemPtr);
        }
        else if (FTextProperty* TextProp = CastField<FTextProperty>(InnerProp))
        {
            JsonArray[i] = SerializeTextProperty(TextProp, ElemPtr);
        }
        else if (FNameProperty* NameProp = CastField<FNameProperty>(InnerProp))
        {
            JsonArray[i] = SerializeNameProperty(NameProp, ElemPtr);
        }
        else if (FStructProperty* StructProp = CastField<FStructProperty>(InnerProp))
        {
            JsonArray[i] = SerializeStructProperty(StructProp, ElemPtr);
        }
        else if (FArrayProperty* NestedArrayProp = CastField<FArrayProperty>(InnerProp))
        {
            JsonArray[i] = SerializeArrayProperty(NestedArrayProp, ElemPtr);
        }
        else if (FMapProperty* MapProp = CastField<FMapProperty>(InnerProp))
        {
            JsonArray[i] = SerializeMapProperty(MapProp, ElemPtr);
        }
        else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(InnerProp))
        {
            JsonArray[i] = SerializeObjectProperty(ObjProp, ElemPtr);
        }
        // Add other property types as needed
    }
    
    return MakeShared<FJsonValueArray>(JsonArray);
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeMapProperty(FMapProperty* MapProp, const void* PropAddr)
{
    // Get the key and value properties
    FProperty* KeyProp = MapProp->KeyProp;
    FProperty* ValueProp = MapProp->ValueProp;
    
    // Create a helper to get the native map
    FScriptMapHelper MapHelper(MapProp, PropAddr);
    
    // Create JSON object
    TSharedPtr<FJsonObject> JsonObject = MakeShared<FJsonObject>();
    
    // Iterate through map elements and serialize to JSON
    for (int32 i = 0; i < MapHelper.GetMaxIndex(); ++i)
    {
        if (!MapHelper.IsValidIndex(i))
        {
            continue;
        }
        
        // Get key and value pointers
        const uint8* KeyPtr = MapHelper.GetKeyPtr(i);
        const uint8* ValuePtr = MapHelper.GetValuePtr(i);
        
        // Get the key as string
        FString KeyString;
        if (FStrProperty* StrKeyProp = CastField<FStrProperty>(KeyProp))
        {
            KeyString = StrKeyProp->GetPropertyValue(KeyPtr);
        }
        else if (FNameProperty* NameKeyProp = CastField<FNameProperty>(KeyProp))
        {
            KeyString = NameKeyProp->GetPropertyValue(KeyPtr).ToString();
        }
        else if (FNumericProperty* NumericKeyProp = CastField<FNumericProperty>(KeyProp))
        {
            if (NumericKeyProp->IsInteger())
            {
                KeyString = FString::FromInt(NumericKeyProp->GetSignedIntPropertyValue(KeyPtr));
            }
            else
            {
                KeyString = FString::SanitizeFloat(NumericKeyProp->GetFloatingPointPropertyValue(KeyPtr));
            }
        }
        else
        {
            // For unsupported key types, try to convert to a string
            KeyString = FString::Printf(TEXT("Key_%d"), i);
        }
        
        // Serialize value based on property type
        TSharedPtr<FJsonValue> JsonValue;
        if (FNumericProperty* NumericProp = CastField<FNumericProperty>(ValueProp))
        {
            JsonValue = SerializeNumericProperty(NumericProp, ValuePtr);
        }
        else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(ValueProp))
        {
            JsonValue = SerializeBoolProperty(BoolProp, ValuePtr);
        }
        else if (FStrProperty* StrProp = CastField<FStrProperty>(ValueProp))
        {
            JsonValue = SerializeStrProperty(StrProp, ValuePtr);
        }
        else if (FTextProperty* TextProp = CastField<FTextProperty>(ValueProp))
        {
            JsonValue = SerializeTextProperty(TextProp, ValuePtr);
        }
        else if (FNameProperty* NameProp = CastField<FNameProperty>(ValueProp))
        {
            JsonValue = SerializeNameProperty(NameProp, ValuePtr);
        }
        else if (FStructProperty* StructProp = CastField<FStructProperty>(ValueProp))
        {
            JsonValue = SerializeStructProperty(StructProp, ValuePtr);
        }
        else if (FArrayProperty* ArrayProp = CastField<FArrayProperty>(ValueProp))
        {
            JsonValue = SerializeArrayProperty(ArrayProp, ValuePtr);
        }
        else if (FMapProperty* NestedMapProp = CastField<FMapProperty>(ValueProp))
        {
            JsonValue = SerializeMapProperty(NestedMapProp, ValuePtr);
        }
        else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(ValueProp))
        {
            JsonValue = SerializeObjectProperty(ObjProp, ValuePtr);
        }
        // Add other property types as needed
        
        if (JsonValue.IsValid())
        {
            JsonObject->SetField(KeyString, JsonValue);
        }
    }
    
    return MakeShared<FJsonValueObject>(JsonObject);
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeObjectProperty(FObjectProperty* ObjProp, const void* PropAddr)
{
    UObject* Object = ObjProp->GetObjectPropertyValue(PropAddr);
    if (Object)
    {
        FString ObjectPath = Object->GetPathName();
        return MakeShared<FJsonValueString>(ObjectPath);
    }
    
    return MakeShared<FJsonValueString>("");
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeSoftObjectProperty(FSoftObjectProperty* SoftObjProp, const void* PropAddr)
{
    const FSoftObjectPtr& SoftObjectPtr = *reinterpret_cast<const FSoftObjectPtr*>(PropAddr);
    return MakeShared<FJsonValueString>(SoftObjectPtr.ToString());
}

TSharedPtr<FJsonValue> FSpacetimeDBPropertyHelper::SerializeEnumProperty(FEnumProperty* EnumProp, const void* PropAddr)
{
    FNumericProperty* UnderlyingProp = EnumProp->GetUnderlyingProperty();
    int64 EnumValue = UnderlyingProp->GetSignedIntPropertyValue(PropAddr);
    
    // Get the enum name
    UEnum* Enum = EnumProp->GetEnum();
    FString EnumName = Enum->GetNameStringByValue(EnumValue);
    
    return MakeShared<FJsonValueString>(EnumName);
}

bool FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty(UObject* Object, const FString& PropertyName, const TSharedPtr<FJsonValue>& JsonValue)
{
    if (!Object)
    {
        UE_LOG(LogTemp, Error, TEXT("FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty - Object is null"));
        return false;
    }

    if (PropertyName.IsEmpty())
    {
        UE_LOG(LogTemp, Error, TEXT("FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty - Property name is empty"));
        return false;
    }

    if (!JsonValue.IsValid())
    {
        UE_LOG(LogTemp, Error, TEXT("FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty - JSON value is invalid"));
        return false;
    }

    // Find the property on the object
    UClass* ObjectClass = Object->GetClass();
    FProperty* Property = FindFProperty<FProperty>(ObjectClass, *PropertyName);
    if (!Property)
    {
        UE_LOG(LogTemp, Warning, TEXT("FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty - Property '%s' not found on object of class '%s'"), 
            *PropertyName, *ObjectClass->GetName());
        return false;
    }

    // Get the address of the property in the object
    void* PropertyAddress = Property->ContainerPtrToValuePtr<void>(Object);
    bool bSuccess = false;

    // Handle different property types - reuse existing helpers where possible
    if (FNumericProperty* NumericProp = CastField<FNumericProperty>(Property))
    {
        bSuccess = DeserializeAndApplyNumericProperty(NumericProp, PropertyAddress, JsonValue);
    }
    else if (FBoolProperty* BoolProp = CastField<FBoolProperty>(Property))
    {
        bSuccess = DeserializeAndApplyBoolProperty(BoolProp, PropertyAddress, JsonValue);
    }
    else if (FStrProperty* StrProp = CastField<FStrProperty>(Property))
    {
        bSuccess = DeserializeAndApplyStrProperty(StrProp, PropertyAddress, JsonValue);
    }
    else if (FTextProperty* TextProp = CastField<FTextProperty>(Property))
    {
        bSuccess = DeserializeAndApplyTextProperty(TextProp, PropertyAddress, JsonValue);
    }
    else if (FNameProperty* NameProp = CastField<FNameProperty>(Property))
    {
        bSuccess = DeserializeAndApplyNameProperty(NameProp, PropertyAddress, JsonValue);
    }
    else if (FStructProperty* StructProp = CastField<FStructProperty>(Property))
    {
        bSuccess = DeserializeAndApplyStructProperty(StructProp, PropertyAddress, JsonValue);
    }
    else if (FArrayProperty* ArrayProp = CastField<FArrayProperty>(Property))
    {
        bSuccess = DeserializeAndApplyArrayProperty(ArrayProp, PropertyAddress, JsonValue);
    }
    else if (FMapProperty* MapProp = CastField<FMapProperty>(Property))
    {
        bSuccess = DeserializeAndApplyMapProperty(MapProp, PropertyAddress, JsonValue);
    }
    else if (FObjectProperty* ObjProp = CastField<FObjectProperty>(Property))
    {
        bSuccess = DeserializeAndApplyObjectProperty(ObjProp, PropertyAddress, JsonValue);
    }
    else if (FSoftObjectProperty* SoftObjProp = CastField<FSoftObjectProperty>(Property))
    {
        bSuccess = DeserializeAndApplySoftObjectProperty(SoftObjProp, PropertyAddress, JsonValue);
    }
    else if (FEnumProperty* EnumProp = CastField<FEnumProperty>(Property))
    {
        bSuccess = DeserializeAndApplyEnumProperty(EnumProp, PropertyAddress, JsonValue);
    }
    else
    {
        UE_LOG(LogTemp, Warning, TEXT("FSpacetimeDBPropertyHelper::ApplyJsonValueToProperty - Unsupported property type for property '%s'"), 
            *PropertyName);
        return false;
    }

    // Fire RepNotify if available
    if (bSuccess && Property->HasAnyPropertyFlags(CPF_RepNotify))
    {
        FName RepNotifyFuncName = Property->RepNotifyFunc;
        if (RepNotifyFuncName != NAME_None)
        {
            UFunction* RepNotifyFunc = Object->GetClass()->FindFunctionByName(RepNotifyFuncName);
            if (RepNotifyFunc)
            {
                // Check if the RepNotify function takes a parameter
                if (RepNotifyFunc->NumParms > 0)
                {
                    // Create a buffer for the parameter
                    uint8* Buffer = (uint8*)FMemory::Malloc(RepNotifyFunc->ParmsSize);
                    FMemory::Memzero(Buffer, RepNotifyFunc->ParmsSize);
                    
                    // Copy the property value to the parameter
                    for (TFieldIterator<FProperty> It(RepNotifyFunc); It && It->HasAnyPropertyFlags(CPF_Parm) && !It->HasAnyPropertyFlags(CPF_ReturnParm); ++It)
                    {
                        void* Parm = It->ContainerPtrToValuePtr<void>(Buffer);
                        It->CopyCompleteValue(Parm, PropertyAddress);
                        break; // Only copy the first parameter
                    }
                    
                    // Call the function
                    Object->ProcessEvent(RepNotifyFunc, Buffer);
                    
                    // Clean up
                    FMemory::Free(Buffer);
                }
                else
                {
                    // Call the function without parameters
                    Object->ProcessEvent(RepNotifyFunc, nullptr);
                }
            }
        }
    }

    return bSuccess;
} 