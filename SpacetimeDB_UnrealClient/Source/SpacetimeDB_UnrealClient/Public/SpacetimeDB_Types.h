#pragma once

#include "CoreMinimal.h"
#include "SpacetimeDB_Types.generated.h"

/**
 * Spawn parameters for creating objects in SpacetimeDB
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBSpawnParams
{
	GENERATED_BODY()

	/** Class name of the object to spawn */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FString ClassName;

	/** World location for spawning the actor */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FVector Location;

	/** World rotation for spawning the actor */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FRotator Rotation;

	/** Whether the object should be replicated to other clients */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	bool bReplicate = true;

	/** Initial properties to set on the spawned object */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	TMap<FString, FString> InitialProperties;

	/** Owner client ID (0 for server-owned) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	int64 OwnerClientId = 0;

	FSpacetimeDBSpawnParams() 
		: Location(FVector::ZeroVector)
		, Rotation(FRotator::ZeroRotator)
	{
	}
};

/**
 * RPC argument structure for SpacetimeDB RPCs
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBRpcArg
{
	GENERATED_BODY()

	/** Name of the argument */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FString Name;

	/** Value of the argument as a string (will be JSON for complex types) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FString Value;

	/** Type of the argument */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	FString Type;

	FSpacetimeDBRpcArg() {}

	FSpacetimeDBRpcArg(const FString& InName, const FString& InValue, const FString& InType)
		: Name(InName), Value(InValue), Type(InType)
	{
	}
};

/**
 * RPC parameters collection for SpacetimeDB RPCs
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBRpcParams
{
	GENERATED_BODY()

	/** Arguments for the RPC */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	TArray<FSpacetimeDBRpcArg> Arguments;

	/** Add an integer argument */
	void AddInt(const FString& Name, int32 Value);

	/** Add a float argument */
	void AddFloat(const FString& Name, float Value);

	/** Add a boolean argument */
	void AddBool(const FString& Name, bool Value);

	/** Add a string argument */
	void AddString(const FString& Name, const FString& Value);

	/** Add a vector argument */
	void AddVector(const FString& Name, const FVector& Value);

	/** Add a rotator argument */
	void AddRotator(const FString& Name, const FRotator& Value);

	/** Add a transform argument */
	void AddTransform(const FString& Name, const FTransform& Value);

	/** Get an integer argument */
	int32 GetInt(const FString& Name) const;

	/** Get a float argument */
	float GetFloat(const FString& Name) const;

	/** Get a boolean argument */
	bool GetBool(const FString& Name) const;

	/** Get a string argument */
	FString GetString(const FString& Name) const;

	/** Get a vector argument */
	FVector GetVector(const FString& Name) const;

	/** Get a rotator argument */
	FRotator GetRotator(const FString& Name) const;

	/** Get a transform argument */
	FTransform GetTransform(const FString& Name) const;
};

/**
 * Type to identify objects in SpacetimeDB
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBObjectID
{
	GENERATED_BODY()

	/** The unique ID of the object */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SpacetimeDB")
	int64 Value;

	FSpacetimeDBObjectID() : Value(0) {}
	FSpacetimeDBObjectID(int64 InValue) : Value(InValue) {}

	operator int64() const { return Value; }
};