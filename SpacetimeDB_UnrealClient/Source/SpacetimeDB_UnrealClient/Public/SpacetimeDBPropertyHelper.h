#pragma once

#include "CoreMinimal.h"
#include "Dom/JsonObject.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "Misc/Base64.h"
#include "UObject/TextProperty.h"
#include "UObject/UnrealType.h"

/**
 * Utility class for handling property serialization and deserialization
 * between SpacetimeDB and Unreal Engine objects.
 */
class SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBPropertyHelper
{
public:
    /**
     * Deserializes a JSON value and applies it to a UObject property
     * 
     * @param Object The UObject to update
     * @param PropertyName The name of the property to update
     * @param ValueJson The JSON value to apply
     * @return True if the property was successfully updated
     */
    static bool ApplyJsonToProperty(UObject* Object, const FString& PropertyName, const FString& ValueJson);
    
    /**
     * Serializes a UObject property to JSON
     * 
     * @param Object The UObject containing the property
     * @param PropertyName The name of the property to serialize
     * @return The JSON string representation of the property
     */
    static FString SerializePropertyToJson(UObject* Object, const FString& PropertyName);

private:
    // Helper functions for specific property types
    static bool DeserializeAndApplyNumericProperty(FNumericProperty* NumericProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyBoolProperty(FBoolProperty* BoolProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyStrProperty(FStrProperty* StrProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyTextProperty(FTextProperty* TextProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyNameProperty(FNameProperty* NameProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyStructProperty(FStructProperty* StructProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyArrayProperty(FArrayProperty* ArrayProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyMapProperty(FMapProperty* MapProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyObjectProperty(FObjectProperty* ObjProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplySoftObjectProperty(FSoftObjectProperty* SoftObjProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplyEnumProperty(FEnumProperty* EnumProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);

    // Helper functions for property serialization
    static TSharedPtr<FJsonValue> SerializeNumericProperty(FNumericProperty* NumericProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeBoolProperty(FBoolProperty* BoolProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeStrProperty(FStrProperty* StrProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeTextProperty(FTextProperty* TextProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeNameProperty(FNameProperty* NameProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeStructProperty(FStructProperty* StructProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeArrayProperty(FArrayProperty* ArrayProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeMapProperty(FMapProperty* MapProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeObjectProperty(FObjectProperty* ObjProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeSoftObjectProperty(FSoftObjectProperty* SoftObjProp, const void* PropAddr);
    static TSharedPtr<FJsonValue> SerializeEnumProperty(FEnumProperty* EnumProp, const void* PropAddr);
}; 