#pragma once

#include "CoreMinimal.h"
#include "SpacetimeDB_Types.h"
#include "SpacetimeDB_PropertyValue.generated.h"

/**
 * Enum that mirrors the Rust PropertyType enum
 */
UENUM(BlueprintType)
enum class ESpacetimeDBPropertyType : uint8
{
    // Primitive types
    Bool        UMETA(DisplayName = "Boolean"),
    Byte        UMETA(DisplayName = "Byte"),
    Int32       UMETA(DisplayName = "Integer (32-bit)"),
    Int64       UMETA(DisplayName = "Integer (64-bit)"),
    UInt32      UMETA(DisplayName = "Unsigned Integer (32-bit)"),
    UInt64      UMETA(DisplayName = "Unsigned Integer (64-bit)"),
    Float       UMETA(DisplayName = "Float (32-bit)"),
    Double      UMETA(DisplayName = "Double (64-bit)"),
    String      UMETA(DisplayName = "String"),
    
    // Structured types
    Vector      UMETA(DisplayName = "Vector"),
    Rotator     UMETA(DisplayName = "Rotator"),
    Quat        UMETA(DisplayName = "Quaternion"),
    Transform   UMETA(DisplayName = "Transform"),
    Color       UMETA(DisplayName = "Color"),
    
    // Reference types
    ObjectReference UMETA(DisplayName = "Object Reference"),
    ClassReference  UMETA(DisplayName = "Class Reference"),
    
    // Container types
    Array       UMETA(DisplayName = "Array"),
    Map         UMETA(DisplayName = "Map"),
    Set         UMETA(DisplayName = "Set"),
    
    // Special types
    Name        UMETA(DisplayName = "Name"),
    Text        UMETA(DisplayName = "Text"),
    Custom      UMETA(DisplayName = "Custom"),
    
    // None type
    None        UMETA(DisplayName = "None")
};

/**
 * Struct to hold the various property value types
 * This struct mirrors the Rust PropertyValue enum, but uses a more C++-friendly approach
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBPropertyValue
{
    GENERATED_BODY()

    /** The type of this property value */
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
    ESpacetimeDBPropertyType Type;

    // Primitive values
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Bool"))
    bool BoolValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Byte"))
    uint8 ByteValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Int32"))
    int32 Int32Value;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Int64"))
    int64 Int64Value;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::UInt32"))
    uint32 UInt32Value;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::UInt64"))
    uint64 UInt64Value;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Float"))
    float FloatValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Double"))
    double DoubleValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::String || Type == ESpacetimeDBPropertyType::Name || Type == ESpacetimeDBPropertyType::Text || Type == ESpacetimeDBPropertyType::ClassReference"))
    FString StringValue;

    // Structured values
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Vector"))
    FVector VectorValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Rotator"))
    FRotator RotatorValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Quat"))
    FQuat QuatValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Transform"))
    FTransform TransformValue;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Color"))
    FColor ColorValue;

    // Reference values
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::ObjectReference"))
    FSpacetimeDBObjectID ObjectReferenceValue;

    // JSON string values for container and custom types
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB|Values", meta = (EditCondition = "Type == ESpacetimeDBPropertyType::Array || Type == ESpacetimeDBPropertyType::Map || Type == ESpacetimeDBPropertyType::Set || Type == ESpacetimeDBPropertyType::Custom"))
    FString JsonValue;

    // Constructors
    FSpacetimeDBPropertyValue()
        : Type(ESpacetimeDBPropertyType::None)
        , BoolValue(false)
        , ByteValue(0)
        , Int32Value(0)
        , Int64Value(0)
        , UInt32Value(0)
        , UInt64Value(0)
        , FloatValue(0.0f)
        , DoubleValue(0.0)
        , StringValue()
        , VectorValue(FVector::ZeroVector)
        , RotatorValue(FRotator::ZeroRotator)
        , QuatValue(FQuat::Identity)
        , TransformValue(FTransform::Identity)
        , ColorValue(FColor::Black)
        , ObjectReferenceValue(0)
        , JsonValue()
    {
    }

    // Conversion constructors for common types
    FSpacetimeDBPropertyValue(bool Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Bool; BoolValue = Value; }
    FSpacetimeDBPropertyValue(uint8 Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Byte; ByteValue = Value; }
    FSpacetimeDBPropertyValue(int32 Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Int32; Int32Value = Value; }
    FSpacetimeDBPropertyValue(int64 Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Int64; Int64Value = Value; }
    FSpacetimeDBPropertyValue(uint32 Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::UInt32; UInt32Value = Value; }
    FSpacetimeDBPropertyValue(uint64 Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::UInt64; UInt64Value = Value; }
    FSpacetimeDBPropertyValue(float Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Float; FloatValue = Value; }
    FSpacetimeDBPropertyValue(double Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Double; DoubleValue = Value; }
    FSpacetimeDBPropertyValue(const FString& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::String; StringValue = Value; }
    FSpacetimeDBPropertyValue(const FVector& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Vector; VectorValue = Value; }
    FSpacetimeDBPropertyValue(const FRotator& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Rotator; RotatorValue = Value; }
    FSpacetimeDBPropertyValue(const FQuat& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Quat; QuatValue = Value; }
    FSpacetimeDBPropertyValue(const FTransform& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Transform; TransformValue = Value; }
    FSpacetimeDBPropertyValue(const FColor& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::Color; ColorValue = Value; }
    FSpacetimeDBPropertyValue(const FSpacetimeDBObjectID& Value) : FSpacetimeDBPropertyValue() { Type = ESpacetimeDBPropertyType::ObjectReference; ObjectReferenceValue = Value; }

    // Factory methods for special types
    static FSpacetimeDBPropertyValue MakeArrayJson(const FString& JsonStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Array; 
        Value.JsonValue = JsonStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeMapJson(const FString& JsonStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Map; 
        Value.JsonValue = JsonStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeSetJson(const FString& JsonStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Set; 
        Value.JsonValue = JsonStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeCustomJson(const FString& JsonStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Custom; 
        Value.JsonValue = JsonStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeName(const FString& NameStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Name; 
        Value.StringValue = NameStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeText(const FString& TextStr) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::Text; 
        Value.StringValue = TextStr; 
        return Value; 
    }

    static FSpacetimeDBPropertyValue MakeClassReference(const FString& ClassName) { 
        FSpacetimeDBPropertyValue Value; 
        Value.Type = ESpacetimeDBPropertyType::ClassReference; 
        Value.StringValue = ClassName; 
        return Value; 
    }

    // Conversion methods to get values
    bool AsBool() const { check(Type == ESpacetimeDBPropertyType::Bool); return BoolValue; }
    uint8 AsByte() const { check(Type == ESpacetimeDBPropertyType::Byte); return ByteValue; }
    int32 AsInt32() const { check(Type == ESpacetimeDBPropertyType::Int32); return Int32Value; }
    int64 AsInt64() const { check(Type == ESpacetimeDBPropertyType::Int64); return Int64Value; }
    uint32 AsUInt32() const { check(Type == ESpacetimeDBPropertyType::UInt32); return UInt32Value; }
    uint64 AsUInt64() const { check(Type == ESpacetimeDBPropertyType::UInt64); return UInt64Value; }
    float AsFloat() const { check(Type == ESpacetimeDBPropertyType::Float); return FloatValue; }
    double AsDouble() const { check(Type == ESpacetimeDBPropertyType::Double); return DoubleValue; }
    const FString& AsString() const { check(Type == ESpacetimeDBPropertyType::String); return StringValue; }
    const FVector& AsVector() const { check(Type == ESpacetimeDBPropertyType::Vector); return VectorValue; }
    const FRotator& AsRotator() const { check(Type == ESpacetimeDBPropertyType::Rotator); return RotatorValue; }
    const FQuat& AsQuat() const { check(Type == ESpacetimeDBPropertyType::Quat); return QuatValue; }
    const FTransform& AsTransform() const { check(Type == ESpacetimeDBPropertyType::Transform); return TransformValue; }
    const FColor& AsColor() const { check(Type == ESpacetimeDBPropertyType::Color); return ColorValue; }
    const FSpacetimeDBObjectID& AsObjectReference() const { check(Type == ESpacetimeDBPropertyType::ObjectReference); return ObjectReferenceValue; }
    const FString& AsName() const { check(Type == ESpacetimeDBPropertyType::Name); return StringValue; }
    const FString& AsText() const { check(Type == ESpacetimeDBPropertyType::Text); return StringValue; }
    const FString& AsClassReference() const { check(Type == ESpacetimeDBPropertyType::ClassReference); return StringValue; }
    const FString& AsJson() const { 
        check(Type == ESpacetimeDBPropertyType::Array || 
              Type == ESpacetimeDBPropertyType::Map || 
              Type == ESpacetimeDBPropertyType::Set || 
              Type == ESpacetimeDBPropertyType::Custom); 
        return JsonValue; 
    }

    // Convert to JSON representation
    FString ToJsonString() const;

    // Parse from JSON representation
    static FSpacetimeDBPropertyValue FromJsonString(const FString& JsonString);
};

/**
 * Helper class for applying property values to UObjects
 */
UCLASS()
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBPropertyHandler : public UObject
{
    GENERATED_BODY()

public:
    /**
     * Apply a property value to a UObject property
     * @param Object - The target UObject
     * @param PropertyName - The name of the property to update
     * @param PropValue - The property value to apply
     * @return True if the property was successfully updated
     */
    static bool ApplyPropertyToObject(UObject* Object, const FString& PropertyName, const FSpacetimeDBPropertyValue& PropValue);

    /**
     * Extract a property value from a UObject property
     * @param Object - The source UObject
     * @param PropertyName - The name of the property to extract
     * @param OutPropValue - The extracted property value
     * @return True if the property was successfully extracted
     */
    static bool ExtractPropertyFromObject(UObject* Object, const FString& PropertyName, FSpacetimeDBPropertyValue& OutPropValue);

    /**
     * Handle a property update from the server
     * @param ObjectId - The ID of the object being updated
     * @param PropertyName - The name of the property being updated
     * @param ValueJson - The JSON string containing the property value
     * @return True if the property was successfully updated
     */
    static bool HandlePropertyUpdate(int64 ObjectId, const FString& PropertyName, const FString& ValueJson);

private:
    /**
     * Apply a property value to a specific UProperty
     * @param Property - The target UProperty
     * @param PropertyPtr - Pointer to the property memory
     * @param PropValue - The property value to apply
     * @return True if the property was successfully updated
     */
    static bool ApplyPropertyValueToProperty(FProperty* Property, void* PropertyPtr, const FSpacetimeDBPropertyValue& PropValue);

    /**
     * Extract a property value from a specific UProperty
     * @param Property - The source UProperty
     * @param PropertyPtr - Pointer to the property memory
     * @param OutPropValue - The extracted property value
     * @return True if the property was successfully extracted
     */
    static bool ExtractPropertyValueFromProperty(FProperty* Property, const void* PropertyPtr, FSpacetimeDBPropertyValue& OutPropValue);
}; 