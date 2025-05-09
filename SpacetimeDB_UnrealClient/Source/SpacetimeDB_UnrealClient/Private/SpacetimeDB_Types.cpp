#include "SpacetimeDB_Types.h"
#include "SpacetimeDB_JsonUtils.h"

// Add an integer argument
void FSpacetimeDBRpcParams::AddInt(const FString& Name, int32 Value)
{
    FString ValueStr = FString::Printf(TEXT("%d"), Value);
    Arguments.Add(FSpacetimeDBRpcArg(Name, ValueStr, TEXT("int")));
}

// Add a float argument
void FSpacetimeDBRpcParams::AddFloat(const FString& Name, float Value)
{
    FString ValueStr = FString::Printf(TEXT("%f"), Value);
    Arguments.Add(FSpacetimeDBRpcArg(Name, ValueStr, TEXT("float")));
}

// Add a boolean argument
void FSpacetimeDBRpcParams::AddBool(const FString& Name, bool Value)
{
    FString ValueStr = Value ? TEXT("true") : TEXT("false");
    Arguments.Add(FSpacetimeDBRpcArg(Name, ValueStr, TEXT("bool")));
}

// Add a string argument
void FSpacetimeDBRpcParams::AddString(const FString& Name, const FString& Value)
{
    Arguments.Add(FSpacetimeDBRpcArg(Name, Value, TEXT("string")));
}

// Add a vector argument
void FSpacetimeDBRpcParams::AddVector(const FString& Name, const FVector& Value)
{
    TSharedPtr<FJsonObject> JsonObject = USpacetimeDBJsonUtils::VectorToJson(Value);
    FString JsonString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
    Writer->Close();
    
    Arguments.Add(FSpacetimeDBRpcArg(Name, JsonString, TEXT("vector")));
}

// Add a rotator argument
void FSpacetimeDBRpcParams::AddRotator(const FString& Name, const FRotator& Value)
{
    TSharedPtr<FJsonObject> JsonObject = USpacetimeDBJsonUtils::RotatorToJson(Value);
    FString JsonString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
    Writer->Close();
    
    Arguments.Add(FSpacetimeDBRpcArg(Name, JsonString, TEXT("rotator")));
}

// Add a transform argument
void FSpacetimeDBRpcParams::AddTransform(const FString& Name, const FTransform& Value)
{
    TSharedPtr<FJsonObject> JsonObject = USpacetimeDBJsonUtils::TransformToJson(Value);
    FString JsonString;
    TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&JsonString);
    FJsonSerializer::Serialize(JsonObject.ToSharedRef(), Writer);
    Writer->Close();
    
    Arguments.Add(FSpacetimeDBRpcArg(Name, JsonString, TEXT("transform")));
}

// Get an integer argument
int32 FSpacetimeDBRpcParams::GetInt(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("int"))
        {
            return FCString::Atoi(*Arg.Value);
        }
    }
    
    // Return 0 if not found or wrong type
    return 0;
}

// Get a float argument
float FSpacetimeDBRpcParams::GetFloat(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("float"))
        {
            return FCString::Atof(*Arg.Value);
        }
    }
    
    // Return 0.0f if not found or wrong type
    return 0.0f;
}

// Get a boolean argument
bool FSpacetimeDBRpcParams::GetBool(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("bool"))
        {
            return Arg.Value.ToLower() == TEXT("true");
        }
    }
    
    // Return false if not found or wrong type
    return false;
}

// Get a string argument
FString FSpacetimeDBRpcParams::GetString(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("string"))
        {
            return Arg.Value;
        }
    }
    
    // Return empty string if not found or wrong type
    return TEXT("");
}

// Get a vector argument
FVector FSpacetimeDBRpcParams::GetVector(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("vector"))
        {
            FVector Result = FVector::ZeroVector;
            
            TSharedPtr<FJsonObject> JsonObject;
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Arg.Value);
            if (FJsonSerializer::Deserialize(Reader, JsonObject) && JsonObject.IsValid())
            {
                USpacetimeDBJsonUtils::JsonToVector(JsonObject, Result);
            }
            
            return Result;
        }
    }
    
    // Return zero vector if not found or wrong type
    return FVector::ZeroVector;
}

// Get a rotator argument
FRotator FSpacetimeDBRpcParams::GetRotator(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("rotator"))
        {
            FRotator Result = FRotator::ZeroRotator;
            
            TSharedPtr<FJsonObject> JsonObject;
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Arg.Value);
            if (FJsonSerializer::Deserialize(Reader, JsonObject) && JsonObject.IsValid())
            {
                USpacetimeDBJsonUtils::JsonToRotator(JsonObject, Result);
            }
            
            return Result;
        }
    }
    
    // Return zero rotator if not found or wrong type
    return FRotator::ZeroRotator;
}

// Get a transform argument
FTransform FSpacetimeDBRpcParams::GetTransform(const FString& Name) const
{
    for (const FSpacetimeDBRpcArg& Arg : Arguments)
    {
        if (Arg.Name == Name && Arg.Type == TEXT("transform"))
        {
            FTransform Result = FTransform::Identity;
            
            TSharedPtr<FJsonObject> JsonObject;
            TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Arg.Value);
            if (FJsonSerializer::Deserialize(Reader, JsonObject) && JsonObject.IsValid())
            {
                USpacetimeDBJsonUtils::JsonToTransform(JsonObject, Result);
            }
            
            return Result;
        }
    }
    
    // Return identity transform if not found or wrong type
    return FTransform::Identity;
}