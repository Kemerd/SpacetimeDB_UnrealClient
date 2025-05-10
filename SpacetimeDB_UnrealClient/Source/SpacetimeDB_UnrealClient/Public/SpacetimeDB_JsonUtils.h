// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Json.h"
#include "UObject/Object.h"
#include "JsonObjectConverter.h"
#include "SpacetimeDB_Types.h"
#include "SpacetimeDB_JsonUtils.generated.h"

/**
 * Utility class for JSON serialization and deserialization for SpacetimeDB.
 */
UCLASS()
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBJsonUtils : public UObject
{
	GENERATED_BODY()

public:
	/**
	 * Serializes a UProperty to a JSON string.
	 * @param Property - The property to serialize.
	 * @param ValuePtr - Pointer to the property value.
	 * @return JSON string representation of the property.
	 */
	static FString SerializePropertyToJson(FProperty* Property, const void* ValuePtr);

	/**
	 * Deserializes a JSON string to a property.
	 * @param Property - The property to deserialize into.
	 * @param ValuePtr - Pointer to where the property value should be stored.
	 * @param JsonString - JSON string to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToProperty(FProperty* Property, void* ValuePtr, const FString& JsonString);

	/**
	 * Serializes a UStruct to a JSON string.
	 * @param StructType - The struct type to serialize.
	 * @param StructPtr - Pointer to the struct instance.
	 * @return JSON string representation of the struct.
	 */
	static FString SerializeStructToJson(UScriptStruct* StructType, const void* StructPtr);

	/**
	 * Deserializes a JSON string to a UStruct.
	 * @param StructType - The struct type to deserialize into.
	 * @param StructPtr - Pointer to where the struct should be stored.
	 * @param JsonString - JSON string to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToStruct(UScriptStruct* StructType, void* StructPtr, const FString& JsonString);

	/**
	 * Deserializes a JSON value to a primitive type.
	 * @param JsonValue - JSON value to deserialize.
	 * @param OutValue - Output parameter for the deserialized value.
	 * @return True if deserialization was successful.
	 */
	template<typename T>
	static bool DeserializeJsonToPrimitive(const TSharedPtr<FJsonValue>& JsonValue, T& OutValue);

	/**
	 * Serializes an array property to a JSON string.
	 * @param ArrayProperty - The array property to serialize.
	 * @param ArrayPtr - Pointer to the array.
	 * @return JSON string representation of the array.
	 */
	static FString SerializeArrayToJson(FArrayProperty* ArrayProperty, const void* ArrayPtr);

	/**
	 * Deserializes a JSON string to an array property.
	 * @param ArrayProperty - The array property to deserialize into.
	 * @param ArrayPtr - Pointer to where the array should be stored.
	 * @param JsonString - JSON string to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToArray(FArrayProperty* ArrayProperty, void* ArrayPtr, const FString& JsonString);

	/**
	 * Serializes a map property to a JSON string.
	 * @param MapProperty - The map property to serialize.
	 * @param MapPtr - Pointer to the map.
	 * @return JSON string representation of the map.
	 */
	static FString SerializeMapToJson(FMapProperty* MapProperty, const void* MapPtr);

	/**
	 * Deserializes a JSON string to a map property.
	 * @param MapProperty - The map property to deserialize into.
	 * @param MapPtr - Pointer to where the map should be stored.
	 * @param JsonString - JSON string to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToMap(FMapProperty* MapProperty, void* MapPtr, const FString& JsonString);

	/**
	 * Serializes a set property to a JSON string.
	 * @param SetProperty - The set property to serialize.
	 * @param SetPtr - Pointer to the set.
	 * @return JSON string representation of the set.
	 */
	static FString SerializeSetToJson(FSetProperty* SetProperty, const void* SetPtr);

	/**
	 * Deserializes a JSON string to a set property.
	 * @param SetProperty - The set property to deserialize into.
	 * @param SetPtr - Pointer to where the set should be stored.
	 * @param JsonString - JSON string to deserialize.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToSet(FSetProperty* SetProperty, void* SetPtr, const FString& JsonString);

	/**
	 * Converts a vector to a JSON object.
	 * @param Vector - The vector to convert.
	 * @return JSON object representation of the vector.
	 */
	static TSharedPtr<FJsonObject> VectorToJson(const FVector& Vector);

	/**
	 * Converts a JSON object to a vector.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutVector - Output parameter for the converted vector.
	 * @return True if conversion was successful.
	 */
	static bool JsonToVector(const TSharedPtr<FJsonObject>& JsonObject, FVector& OutVector);

	/**
	 * Converts a rotator to a JSON object.
	 * @param Rotator - The rotator to convert.
	 * @return JSON object representation of the rotator.
	 */
	static TSharedPtr<FJsonObject> RotatorToJson(const FRotator& Rotator);

	/**
	 * Converts a JSON object to a rotator.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutRotator - Output parameter for the converted rotator.
	 * @return True if conversion was successful.
	 */
	static bool JsonToRotator(const TSharedPtr<FJsonObject>& JsonObject, FRotator& OutRotator);

	/**
	 * Converts a transform to a JSON object.
	 * @param Transform - The transform to convert.
	 * @return JSON object representation of the transform.
	 */
	static TSharedPtr<FJsonObject> TransformToJson(const FTransform& Transform);

	/**
	 * Converts a JSON object to a transform.
	 * @param JsonObject - The JSON object to convert.
	 * @param OutTransform - Output parameter for the converted transform.
	 * @return True if conversion was successful.
	 */
	static bool JsonToTransform(const TSharedPtr<FJsonObject>& JsonObject, FTransform& OutTransform);

	/**
	 * Serializes RPC arguments to a JSON string.
	 * @param Args - Array of JSON values representing the arguments.
	 * @return JSON string representation of the arguments.
	 */
	static FString SerializeRpcArgsToJson(const TArray<TSharedPtr<FJsonValue>>& Args);

	/**
	 * Deserializes a JSON string to RPC arguments.
	 * @param JsonString - JSON string representation of the arguments.
	 * @param OutArgs - Output parameter for the deserialized arguments.
	 * @return True if deserialization was successful.
	 */
	static bool DeserializeJsonToRpcArgs(const FString& JsonString, TArray<TSharedPtr<FJsonValue>>& OutArgs);

	/**
	 * Serializes an RPC result to a JSON string.
	 * @param Result - The RPC result as a JSON value.
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