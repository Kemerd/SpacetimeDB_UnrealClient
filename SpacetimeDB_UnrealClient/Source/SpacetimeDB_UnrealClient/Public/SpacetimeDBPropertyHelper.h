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
     * Serializes a UObject property to JSON
     * 
     * @param Object The UObject containing the property
     * @param PropertyName The name of the property to serialize
     * @return The JSON string representation of the property
     */
    static FString SerializePropertyToJson(UObject* Object, const FString& PropertyName);

    /**
     * Applies a JSON string to a property on an object.
     * 
     * @param Object The object that contains the property
     * @param PropertyName The name of the property to modify
     * @param Json The JSON string to apply
     * @return True if the property was successfully applied
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    static bool ApplyJsonToProperty(UObject* Object, const FString& PropertyName, const FString& Json);

    /**
     * Applies a JSON value to a property on an object.
     * 
     * @param Object The object that contains the property
     * @param PropertyName The name of the property to modify
     * @param JsonValue The JSON value to apply
     * @return True if the property was successfully applied
     */
    UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Properties")
    static bool ApplyJsonValueToProperty(UObject* Object, const FString& PropertyName, const TSharedPtr<FJsonValue>& JsonValue);
    
    /**
     * Gets a property value as JSON from a UObject.
     * 
     * @param Object The object that contains the property
     * @param PropertyName The name of the property to get
     * @return JSON string representing the property value, or empty string if not found or error
     */
    static FString GetPropertyValueByName(UObject* Object, const FString& PropertyName);
    
    /**
     * Sets a property value on a UObject from a JSON string.
     * 
     * @param Object The object to update
     * @param PropertyName The name of the property to set
     * @param JsonValue The JSON string value to set
     * @return True if the property was successfully set
     */
    static bool SetPropertyValueByName(UObject* Object, const FString& PropertyName, const FString& JsonValue);

private:
    /**
     * Serializes a property to a JSON value.
     * 
     * @param Property The property to serialize
     * @param PropertyAddr Pointer to the property's memory
     * @return The serialized JSON value
     */
    static TSharedPtr<FJsonValue> SerializePropertyToJsonValue(FProperty* Property, const void* PropertyAddr);

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
    static bool DeserializeAndApplyEnumProperty(FEnumProperty* EnumProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);
    static bool DeserializeAndApplySoftObjectProperty(FSoftObjectProperty* SoftObjProp, void* PropAddr, const TSharedPtr<FJsonValue>& JsonValue);

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