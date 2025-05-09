#pragma once

#include "CoreMinimal.h"
#include "Dom/JsonObject.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonWriter.h"
#include "UObject/UnrealType.h"
#include "SpacetimeDB_JsonUtils.generated.h"

/**
 * Utility class for handling JSON serialization and deserialization for SpacetimeDB properties and RPCs.
 * Handles conversion between Unreal Engine data types and JSON representations used by the SpacetimeDB Rust modules.
 */
UCLASS()
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBJsonUtils : public UObject
{
	GENERATED_BODY()

public:
	/**
	 * Serializes a UProperty value to a JSON string.
	 * @param Property - The UProperty to serialize.
	 * @param ValuePtr - Pointer to the property value.
	 * @return JSON string representation of the property value.
	 */
	static FString SerializePropertyToJson(FProperty* Property, const void* ValuePtr);

	/**
	 * Deserializes a JSON string to a UProperty value.
	 * @param Property - The UProperty to deserialize into.
	 * @param ValuePtr - Pointer to where the property value should be stored.
	 * @param JsonString - JSON string representation of the property value.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToProperty(FProperty* Property, void* ValuePtr, const FString& JsonString);

	/**
	 * Serializes a UStruct to a JSON string.
	 * @param StructType - The UScriptStruct type.
	 * @param StructPtr - Pointer to the struct data.
	 * @return JSON string representation of the struct.
	 */
	static FString SerializeStructToJson(UScriptStruct* StructType, const void* StructPtr);

	/**
	 * Deserializes a JSON string to a UStruct.
	 * @param StructType - The UScriptStruct type.
	 * @param StructPtr - Pointer to where the struct data should be stored.
	 * @param JsonString - JSON string representation of the struct.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToStruct(UScriptStruct* StructType, void* StructPtr, const FString& JsonString);

	/**
	 * Serializes a primitive value to JSON.
	 * Handles bool, integer, float, and string types.
	 * @param Value - The value to serialize.
	 * @return JSON value object.
	 */
	template<typename T>
	static TSharedPtr<FJsonValue> SerializePrimitiveToJson(const T& Value);

	/**
	 * Deserializes a JSON value to a primitive value.
	 * @param JsonValue - The JSON value to deserialize.
	 * @param OutValue - Output parameter for the deserialized value.
	 * @return True if deserialization was successful.
	 */
	template<typename T>
	static bool DeserializeJsonToPrimitive(const TSharedPtr<FJsonValue>& JsonValue, T& OutValue);

	/**
	 * Serializes an array to a JSON array.
	 * @param ArrayProperty - The array property.
	 * @param ArrayPtr - Pointer to the array data.
	 * @return JSON string representation of the array.
	 */
	static FString SerializeArrayToJson(FArrayProperty* ArrayProperty, const void* ArrayPtr);

	/**
	 * Deserializes a JSON array to an array.
	 * @param ArrayProperty - The array property.
	 * @param ArrayPtr - Pointer to where the array data should be stored.
	 * @param JsonString - JSON string representation of the array.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToArray(FArrayProperty* ArrayProperty, void* ArrayPtr, const FString& JsonString);

	/**
	 * Serializes a map to a JSON object.
	 * @param MapProperty - The map property.
	 * @param MapPtr - Pointer to the map data.
	 * @return JSON string representation of the map.
	 */
	static FString SerializeMapToJson(FMapProperty* MapProperty, const void* MapPtr);

	/**
	 * Deserializes a JSON object to a map.
	 * @param MapProperty - The map property.
	 * @param MapPtr - Pointer to where the map data should be stored.
	 * @param JsonString - JSON string representation of the map.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToMap(FMapProperty* MapProperty, void* MapPtr, const FString& JsonString);

	/**
	 * Serializes a set to a JSON array.
	 * @param SetProperty - The set property.
	 * @param SetPtr - Pointer to the set data.
	 * @return JSON string representation of the set.
	 */
	static FString SerializeSetToJson(FSetProperty* SetProperty, const void* SetPtr);

	/**
	 * Deserializes a JSON array to a set.
	 * @param SetProperty - The set property.
	 * @param SetPtr - Pointer to where the set data should be stored.
	 * @param JsonString - JSON string representation of the set.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToSet(FSetProperty* SetProperty, void* SetPtr, const FString& JsonString);

	/**
	 * Helper to convert an FVector to JSON.
	 * @param Vector - The vector to convert.
	 * @return JSON object representation of the vector.
	 */
	static TSharedPtr<FJsonObject> VectorToJson(const FVector& Vector);

	/**
	 * Helper to convert JSON to an FVector.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutVector - Output parameter for the converted vector.
	 * @return True if conversion was successful.
	 */
	static bool JsonToVector(const TSharedPtr<FJsonObject>& JsonObject, FVector& OutVector);

	/**
	 * Helper to convert an FRotator to JSON.
	 * @param Rotator - The rotator to convert.
	 * @return JSON object representation of the rotator.
	 */
	static TSharedPtr<FJsonObject> RotatorToJson(const FRotator& Rotator);

	/**
	 * Helper to convert JSON to an FRotator.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutRotator - Output parameter for the converted rotator.
	 * @return True if conversion was successful.
	 */
	static bool JsonToRotator(const TSharedPtr<FJsonObject>& JsonObject, FRotator& OutRotator);

	/**
	 * Helper to convert an FTransform to JSON.
	 * @param Transform - The transform to convert.
	 * @return JSON object representation of the transform.
	 */
	static TSharedPtr<FJsonObject> TransformToJson(const FTransform& Transform);

	/**
	 * Helper to convert JSON to an FTransform.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutTransform - Output parameter for the converted transform.
	 * @return True if conversion was successful.
	 */
	static bool JsonToTransform(const TSharedPtr<FJsonObject>& JsonObject, FTransform& OutTransform);

	/**
	 * Serializes RPC arguments to a JSON string.
	 * @param Args - Array of RPC arguments.
	 * @return JSON string representation of the arguments.
	 */
	static FString SerializeRpcArgsToJson(const TArray<TSharedPtr<FJsonValue>>& Args);

	/**
	 * Deserializes a JSON string to RPC arguments.
	 * @param JsonString - JSON string representation of the arguments.
	 * @param OutArgs - Output array for the deserialized arguments.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToRpcArgs(const FString& JsonString, TArray<TSharedPtr<FJsonValue>>& OutArgs);

	/**
	 * Serializes RPC result to a JSON string.
	 * @param Result - The RPC result.
	 * @return JSON string representation of the result.
	 */
	static FString SerializeRpcResultToJson(const TSharedPtr<FJsonValue>& Result);

	/**
	 * Deserializes a JSON string to an RPC result.
	 * @param JsonString - JSON string representation of the result.
	 * @param OutResult - Output parameter for the deserialized result.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToRpcResult(const FString& JsonString, TSharedPtr<FJsonValue>& OutResult);

	/**
	 * Serializes spawn parameters to a JSON string.
	 * @param SpawnParams - The spawn parameters.
	 * @return JSON string representation of the spawn parameters.
	 */
	static FString SerializeSpawnParamsToJson(const FSpacetimeDBSpawnParams& SpawnParams);

	/**
	 * Deserializes a JSON string to spawn parameters.
	 * @param JsonString - JSON string representation of the spawn parameters.
	 * @param OutSpawnParams - Output parameter for the deserialized spawn parameters.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToSpawnParams(const FString& JsonString, FSpacetimeDBSpawnParams& OutSpawnParams);

private:
	/**
	 * Helper function to serialize a property value to a JSON value.
	 * @param Property - The property to serialize.
	 * @param ValuePtr - Pointer to the property value.
	 * @return JSON value representation of the property value.
	 */
	static TSharedPtr<FJsonValue> SerializePropertyToJsonValue(FProperty* Property, const void* ValuePtr);

	/**
	 * Helper function to deserialize a JSON value to a property value.
	 * @param Property - The property to deserialize into.
	 * @param ValuePtr - Pointer to where the property value should be stored.
	 * @param JsonValue - JSON value to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonValueToProperty(FProperty* Property, void* ValuePtr, const TSharedPtr<FJsonValue>& JsonValue);

	/**
	 * Helper function to convert a JSON value to a string.
	 * @param JsonValue - The JSON value to convert.
	 * @return String representation of the JSON value.
	 */
	static FString JsonValueToString(const TSharedPtr<FJsonValue>& JsonValue);
};