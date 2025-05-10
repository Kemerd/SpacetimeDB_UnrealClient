#include "SpacetimeDBPredictionComponent.h"
#include "GameFramework/Actor.h"
#include "GameFramework/Character.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "SpacetimeDBSubsystem.h"
#include "Engine/World.h"
#include "SpacetimeDBPropertyHelper.h"
#include "SpacetimeDB_PropertyValue.h"

// Helper functions for vector operations 
FORCEINLINE static float GetManhattanDistance(const FVector& A, const FVector& B)
{
	return FMath::Abs(A.X - B.X) + FMath::Abs(A.Y - B.Y) + FMath::Abs(A.Z - B.Z);
}

FORCEINLINE static float GetRotationError(const FQuat& A, const FQuat& B) 
{
	return FMath::RadiansToDegrees(FQuat::Error(A, B));
}

// Implementation of the One Euro Filter
float USpacetimeDBPredictionComponent::FOneEuroFilter::Filter(float InValue, float InDeltaTime)
{
	// On first call, initialize with input value
	if (LastTime <= 0.0f)
	{
		LastTime = InDeltaTime;
		LastValue = InValue;
		LastRawValue = InValue;
		Value = InValue;
		return InValue;
	}

	// Calculate the alpha value based on cutoff frequency
	const float Alpha = FMath::Exp(-2.0f * PI * MinCutoff * InDeltaTime);
	
	// Calculate the derivative of the input signal
	const float DValue = (InValue - LastRawValue) / InDeltaTime;
	LastRawValue = InValue;
	
	// Calculate the cutoff frequency for the derivative
	const float DAlpha = FMath::Exp(-2.0f * PI * DCutoff * InDeltaTime);
	
	// Filter the derivative
	const float DFiltered = FMath::Lerp(DValue, LastValue, DAlpha);
	LastValue = DFiltered;
	
	// Calculate the adaptive cutoff frequency based on movement speed
	const float Cutoff = MinCutoff + Beta * FMath::Abs(DFiltered);
	
	// Apply the filter to the input value
	const float AlphaCutoff = FMath::Exp(-2.0f * PI * Cutoff * InDeltaTime);
	Value = FMath::Lerp(InValue, Value, AlphaCutoff);
	
	return Value;
}

// Constructor
USpacetimeDBPredictionComponent::USpacetimeDBPredictionComponent()
{
	// Set this component to be initialized when the game starts, and to be ticked every frame
	PrimaryComponentTick.bCanEverTick = true;
	PrimaryComponentTick.TickGroup = TG_PrePhysics; // Process before physics to allow for prediction

	// Initialize default values
	bHasAuthority = false;
	CurrentSequence = 0;
	LastAcknowledgedSequence = -1;
}

void USpacetimeDBPredictionComponent::BeginPlay()
{
	Super::BeginPlay();
	
	// Check if this component is on a locally controlled pawn
	AActor* Owner = GetOwner();
	if (Owner)
	{
		// Check if this is a Character with a PlayerController
		ACharacter* OwnerCharacter = Cast<ACharacter>(Owner);
		if (OwnerCharacter && OwnerCharacter->IsLocallyControlled())
		{
			bHasAuthority = true;
		}
	}
}

void USpacetimeDBPredictionComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
	Super::TickComponent(DeltaTime, TickType, ThisTickFunction);

	// Only perform prediction on locally controlled pawns
	if (!bHasAuthority)
	{
		return;
	}

	// Clean up old history entries
	CleanupHistory();
}

void USpacetimeDBPredictionComponent::EndPlay(const EEndPlayReason::Type EndPlayReason)
{
	// Clear history on end play
	StateHistory.Empty();
	
	Super::EndPlay(EndPlayReason);
}

void USpacetimeDBPredictionComponent::TakeStateSnapshot()
{
	// Don't take snapshots for non-authoritative components
	if (!bHasAuthority)
	{
		return;
	}

	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	// Create a new snapshot
	FStateSnapshot NewSnapshot;
	NewSnapshot.Timestamp = Owner->GetWorld()->GetTimeSeconds();
	NewSnapshot.Transform = Owner->GetActorTransform();
	NewSnapshot.SequenceNumber = CurrentSequence++;

	// Store velocity if this is a character
	ACharacter* Character = Cast<ACharacter>(Owner);
	if (Character)
	{
		UCharacterMovementComponent* MovementComp = Character->GetCharacterMovement();
		if (MovementComp)
		{
			NewSnapshot.Velocity = MovementComp->Velocity;
		}
	}

	// Copy the current input state
	NewSnapshot.InputState = CurrentInputs;

	// Capture custom tracked properties
	CaptureTrackedProperties(NewSnapshot.CustomState);

	// Add to history
	StateHistory.Add(NewSnapshot);
}

void USpacetimeDBPredictionComponent::ApplyPredictedChanges()
{
	// This is typically a stub method in this implementation
	// The actual prediction is done by Unreal's natural movement system
	// This method is a hook for game-specific prediction logic
	
	// If you want to add game-specific prediction logic, add it here
}

void USpacetimeDBPredictionComponent::ProcessServerUpdate(const FTransform& ServerTransform, 
	const FVector& ServerVelocity, int32 AckedSequence)
{
	// Skip processing if not authoritative
	if (!bHasAuthority)
	{
		return;
	}

	// Store the last acknowledged sequence
	LastAcknowledgedSequence = AckedSequence;

	// Find the snapshot that corresponds to this server update
	int32 MatchingSnapshotIndex = -1;
	for (int32 i = 0; i < StateHistory.Num(); ++i)
	{
		if (StateHistory[i].SequenceNumber == AckedSequence)
		{
			MatchingSnapshotIndex = i;
			break;
		}
	}

	// If we didn't find a matching snapshot, we can't reconcile
	if (MatchingSnapshotIndex == -1)
	{
		// If we got a server update for a sequence we don't have, just apply it directly
		ApplySmoothCorrection(ServerTransform, ServerVelocity, 0.0f); // No smoothing, direct update
		return;
	}

	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	// Get the current client state
	FTransform CurrentTransform = Owner->GetActorTransform();
	FVector CurrentVelocity = FVector::ZeroVector;

	// Get velocity if this is a character
	ACharacter* Character = Cast<ACharacter>(Owner);
	if (Character)
	{
		UCharacterMovementComponent* MovementComp = Character->GetCharacterMovement();
		if (MovementComp)
		{
			CurrentVelocity = MovementComp->Velocity;
		}
	}

	// Check if the error exceeds thresholds
	float PositionError = GetManhattanDistance(CurrentTransform.GetLocation(), ServerTransform.GetLocation());
	float RotationError = GetRotationError(CurrentTransform.GetRotation(), ServerTransform.GetRotation());
	float VelocityError = GetManhattanDistance(CurrentVelocity, ServerVelocity);

	bool bNeedsCorrection = 
		PositionError > PositionErrorThreshold ||
		RotationError > RotationErrorThreshold ||
		VelocityError > VelocityErrorThreshold;

	if (bNeedsCorrection)
	{
		// Apply smooth correction
		ApplySmoothCorrection(ServerTransform, ServerVelocity, SmoothingFactor);
		
		// Re-apply inputs since the matching snapshot
		// For now, we'll just rely on Unreal's movement prediction to correct itself
	}

	// Clean up history entries older than the acknowledged sequence
	for (int32 i = StateHistory.Num() - 1; i >= 0; i--)
	{
		if (StateHistory[i].SequenceNumber <= AckedSequence)
		{
			StateHistory.RemoveAt(0, i + 1);
			break;
		}
	}
}

void USpacetimeDBPredictionComponent::AddTrackedProperty(FName PropertyName)
{
	if (!TrackedProperties.Contains(PropertyName))
	{
		TrackedProperties.Add(PropertyName);
	}
}

void USpacetimeDBPredictionComponent::RegisterInputValue(FName InputName, float Value)
{
	CurrentInputs.Add(InputName, Value);
}

void USpacetimeDBPredictionComponent::CaptureTrackedProperties(TMap<FName, FSpacetimeDBPropertyValue>& OutProperties)
{
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	// Iterate over the member variable TrackedProperties
	for (const FName& PropName : this->TrackedProperties)
	{
		// Use FSpacetimeDBPropertyHelper (F instead of U)
		FString PropertyValueJson = FSpacetimeDBPropertyHelper::GetPropertyValueByName(Owner, PropName.ToString());
		
		// Attempt to deserialize the JSON string into FSpacetimeDBPropertyValue
		// This is a simplified approach; robust error handling and type checking would be needed here.
		FSpacetimeDBPropertyValue PropValue;
		TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(PropertyValueJson);
		TSharedPtr<FJsonValue> JsonValue;

		// Basic deserialization (assuming simple value types for now)
		// A more complete solution would involve inspecting FJsonValue::Type and setting accordingly
		if (FJsonSerializer::Deserialize(Reader, JsonValue) && JsonValue.IsValid())
		{
            if (JsonValue->Type == EJson::String)
            {
                PropValue.SetString(JsonValue->AsString());
            }
            else if (JsonValue->Type == EJson::Number)
            {
                // SpacetimeDBPropertyValue might need to differentiate between int/float
                // For now, assume float if it's a number
                PropValue.SetFloat(static_cast<float>(JsonValue->AsNumber()));
            }
            else if (JsonValue->Type == EJson::Boolean)
            {
                PropValue.SetBool(JsonValue->AsBool());
            }
            // Add more types as needed (Array, Object, Null)
            // For complex types, FSpacetimeDBPropertyValue would need dedicated parsing logic
		}
		OutProperties.Add(PropName, PropValue);
	}
}

void USpacetimeDBPredictionComponent::GetTrackedProperties(TMap<FName, FSpacetimeDBPropertyValue>& OutProperties)
{
	// Get the owning actor of this component.
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		// If there's no owner, we cannot retrieve properties.
		return;
	}

	// Clear any existing properties in the output map.
	OutProperties.Empty();

	// Iterate over all property names that are being tracked by this component.
	for (const FName& PropName : TrackedProperties)
	{
		// Retrieve the property value as a JSON string from the owner actor.
		// FSpacetimeDBPropertyHelper::GetPropertyValueByName is expected to serialize the property into a JSON string.
		FString JsonValueStr = FSpacetimeDBPropertyHelper::GetPropertyValueByName(Owner, PropName.ToString());
		
		if (!JsonValueStr.IsEmpty())
		{
			// Convert the JSON string representation of the property value into an FSpacetimeDBPropertyValue struct.
			FSpacetimeDBPropertyValue PropertyValue = FSpacetimeDBPropertyValue::FromJsonString(JsonValueStr);
			
			// Add the successfully converted property value to the output map.
			OutProperties.Add(PropName, PropertyValue);
		}
		else
		{
			// Log a warning if a tracked property could not be retrieved or was empty.
			UE_LOG(LogSpacetimeDB, Warning, TEXT("Could not get property value for tracked property '%s' on actor '%s'."), *PropName.ToString(), *Owner->GetName());
		}
	}
}

void USpacetimeDBPredictionComponent::ApplyTrackedProperties(const TMap<FName, FSpacetimeDBPropertyValue>& Properties)
{
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	for (const auto& Pair : Properties)
	{
		const FName& PropName = Pair.Key;
		const FSpacetimeDBPropertyValue& Value = Pair.Value;

		// Convert FSpacetimeDBPropertyValue to a JSON string to use with SetPropertyValueByName
		// This is a placeholder. A robust solution would serialize FSpacetimeDBPropertyValue correctly.
		FString ValueJsonString;
        // TODO: Implement proper serialization of FSpacetimeDBPropertyValue to JSON string
        // For now, let's try to handle a few common types.
        // This is a simplified conversion and might not cover all cases or complex types.
        if (Value.IsString())
        {
            // Properly escape the string for JSON
            TSharedPtr<FJsonValueString> JsonStringValue = MakeShareable(new FJsonValueString(Value.GetString()));
            FJsonSerializer::Serialize(JsonStringValue.ToSharedRef(), TEXT(""), TJsonWriterFactory<TCHAR, TCondensedJsonPrintPolicy<TCHAR>>::Create(&ValueJsonString), false);

        }
        else if (Value.IsInt())
        {
            ValueJsonString = FString::Printf(TEXT("%lld"), Value.GetInt());
        }
        else if (Value.IsFloat())
        {
            // Ensure proper float to string conversion for JSON
            ValueJsonString = FString::Printf(TEXT("%f"), Value.GetFloat());
        }
        else if (Value.IsBool())
        {
            ValueJsonString = Value.GetBool() ? TEXT("true") : TEXT("false");
        }
        else if (Value.IsNull())
        {
            ValueJsonString = TEXT("null");
        }
        else
        {
            // For USTRUCTs, UOBJECTs, Arrays, Maps, a more complex serialization is needed.
            // This might involve recursively calling a serialization function or using Unreal's built-in JSON utilities
            // if FSpacetimeDBPropertyValue holds complex data.
            UE_LOG(LogTemp, Warning, TEXT("ApplyTrackedProperties: Property '%s' has a complex or unsupported type for simple JSON conversion. Value not applied."), *PropName.ToString());
            continue; // Skip this property if we can't easily convert it
        }
        
		// Use FSpacetimeDBPropertyHelper (F instead of U)
		bool bSuccess = FSpacetimeDBPropertyHelper::SetPropertyValueByName(Owner, PropName.ToString(), ValueJsonString);
        if (!bSuccess)
        {
            UE_LOG(LogTemp, Warning, TEXT("ApplyTrackedProperties: Failed to set property '%s' on actor '%s' with value: %s"), *PropName.ToString(), *Owner->GetName(), *ValueJsonString);
        }
	}
}

void USpacetimeDBPredictionComponent::CleanupHistory()
{
	// Remove history entries beyond the maximum length
	if (StateHistory.Num() > MaxHistoryLength)
	{
		int32 NumToRemove = StateHistory.Num() - MaxHistoryLength;
		StateHistory.RemoveAt(0, NumToRemove);
	}
}

void USpacetimeDBPredictionComponent::ApplySmoothCorrection(const FTransform& TargetTransform, 
	const FVector& TargetVelocity, float BlendFactor)
{
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	// Apply immediate correction if blend factor is 0
	if (BlendFactor <= 0.0f)
	{
		Owner->SetActorTransform(TargetTransform);
		
		// Set velocity if this is a character
		ACharacter* Character = Cast<ACharacter>(Owner);
		if (Character)
		{
			UCharacterMovementComponent* MovementComp = Character->GetCharacterMovement();
			if (MovementComp)
			{
				MovementComp->Velocity = TargetVelocity;
			}
		}
		return;
	}

	// Get current transforms
	FTransform CurrentTransform = Owner->GetActorTransform();
	
	// Get the world
	UWorld* World = Owner->GetWorld();
	if (!World)
	{
		return;
	}
	
	// Apply the One Euro filter for smooth correction
	float DeltaTime = World->GetDeltaSeconds();

	// Apply filtered position
	FVector CurrentLocation = CurrentTransform.GetLocation();
	FVector TargetLocation = TargetTransform.GetLocation();
	
	// Filter each component for smoother results
	FVector FilteredLocation;
	FilteredLocation.X = PositionFilterX.Filter(TargetLocation.X, DeltaTime);
	FilteredLocation.Y = PositionFilterY.Filter(TargetLocation.Y, DeltaTime);
	FilteredLocation.Z = PositionFilterZ.Filter(TargetLocation.Z, DeltaTime);
	
	// Blend between current and target position based on smoothing factor
	FVector NewLocation = FMath::Lerp(FilteredLocation, CurrentLocation, BlendFactor);
	
	// Apply rotation change - simpler method for rotation
	FQuat CurrentRotation = CurrentTransform.GetRotation();
	FQuat TargetRotation = TargetTransform.GetRotation();
	FQuat NewRotation = FQuat::Slerp(TargetRotation, CurrentRotation, BlendFactor);
	
	// Set the new transform
	FTransform NewTransform = CurrentTransform;
	NewTransform.SetLocation(NewLocation);
	NewTransform.SetRotation(NewRotation);
	Owner->SetActorTransform(NewTransform);
	
	// Apply velocity change for characters
	ACharacter* Character = Cast<ACharacter>(Owner);
	if (Character)
	{
		UCharacterMovementComponent* MovementComp = Character->GetCharacterMovement();
		if (MovementComp)
		{
			FVector CurrentVelocity = MovementComp->Velocity;
			FVector NewVelocity = FMath::Lerp(TargetVelocity, CurrentVelocity, BlendFactor);
			MovementComp->Velocity = NewVelocity;
		}
	}
}

void USpacetimeDBPredictionComponent::ApplyServerUpdate(const FString& PropertyName, const FSpacetimeDBPropertyValue& PropValue)
{
	// Get the owning actor of this component.
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		// If there's no owner, we cannot apply any updates.
		UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Owner is null, cannot apply property '%s'."), *PropertyName);
		return;
	}

	// Find the FProperty on the Owner actor that matches the given PropertyName.
	FProperty* Property = Owner->GetClass()->FindPropertyByName(FName(*PropertyName));
	if (!Property)
	{
		// If the property is not found on the owner, log a warning.
		UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Property '%s' not found on actor '%s'."), *PropertyName, *Owner->GetName());
		return;
	}

	// Get a direct pointer to the memory location of the property within the Owner actor.
	void* PropertyAddress = Property->ContainerPtrToValuePtr<void>(Owner);

	// Based on the FProperty type, cast it and set its value using the provided PropValue.
	// This section handles various common property types.

	if (FNumericProperty* NumericProperty = CastField<FNumericProperty>(Property))
	{
		// Handle numeric types (float, int, double, etc.)
		if (NumericProperty->IsFloatingPoint())
		{
			// Check if the server-provided value is a float or double.
			if (PropValue.Type == ESpacetimeDBPropertyType::Float)
			{
				NumericProperty->SetFloatingPointPropertyValue(PropertyAddress, PropValue.AsFloat());
			}
			else if (PropValue.Type == ESpacetimeDBPropertyType::Double)
			{
				NumericProperty->SetFloatingPointPropertyValue(PropertyAddress, PropValue.AsDouble());
			}
			else
			{
				UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for float/double property '%s'. Expected Float/Double, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
			}
		}
		else // Integer types
		{
			// Check if the server-provided value is an int32 or int64.
			if (PropValue.Type == ESpacetimeDBPropertyType::Int32)
			{
				NumericProperty->SetIntPropertyValue(PropertyAddress, PropValue.AsInt32());
			}
			else if (PropValue.Type == ESpacetimeDBPropertyType::Int64)
			{
				NumericProperty->SetIntPropertyValue(PropertyAddress, PropValue.AsInt64());
			}
			else if (PropValue.Type == ESpacetimeDBPropertyType::Byte) // Also handle Byte as an integer type
			{
				NumericProperty->SetIntPropertyValue(PropertyAddress, PropValue.AsByte());
			}
			else
			{
				UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for integer property '%s'. Expected Int32/Int64/Byte, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
			}
		}
	}
	else if (FBoolProperty* BoolProperty = CastField<FBoolProperty>(Property))
	{
		// Handle boolean properties.
		if (PropValue.Type == ESpacetimeDBPropertyType::Bool)
		{
			BoolProperty->SetPropertyValue(PropertyAddress, PropValue.AsBool());
		}
		else
		{
			UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for bool property '%s'. Expected Bool, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
		}
	}
	else if (FStrProperty* StringProperty = CastField<FStrProperty>(Property))
	{
		// Handle FString properties.
		if (PropValue.Type == ESpacetimeDBPropertyType::String)
		{
			StringProperty->SetPropertyValue(PropertyAddress, PropValue.AsString());
		}
		else
		{
			UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for string property '%s'. Expected String, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
		}
	}
	else if (FNameProperty* NameProperty = CastField<FNameProperty>(Property))
	{
		// Handle FName properties. FNames are often stored as strings in SpacetimeDB.
		if (PropValue.Type == ESpacetimeDBPropertyType::Name || PropValue.Type == ESpacetimeDBPropertyType::String)
		{
			NameProperty->SetPropertyValue(PropertyAddress, FName(PropValue.AsString()));
		}
		else
		{
			UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for FName property '%s'. Expected Name or String, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
		}
	}
	else if (FTextProperty* TextProperty = CastField<FTextProperty>(Property))
	{
		// Handle FText properties. FTexts are often stored as strings in SpacetimeDB.
		if (PropValue.Type == ESpacetimeDBPropertyType::Text || PropValue.Type == ESpacetimeDBPropertyType::String)
		{
			TextProperty->SetPropertyValue(PropertyAddress, FText::FromString(PropValue.AsString()));
		}
		else
		{
			UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for FText property '%s'. Expected Text or String, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type));
		}
	}
	else if (FStructProperty* StructProperty = CastField<FStructProperty>(Property))
	{
		// Handle common UStruct types like FVector, FRotator, FTransform.
		// This relies on FSpacetimeDBPropertyHelper::SetPropertyValueByName to handle the JSON to Struct conversion.
		// The PropValue itself might be a direct struct or a JSON string for custom/complex structs.

		if (StructProperty->Struct == TBaseStructure<FVector>::Get())
		{
			if (PropValue.Type == ESpacetimeDBPropertyType::Vector)
			{
				*static_cast<FVector*>(PropertyAddress) = PropValue.AsVector();
			}
			else { UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for FVector property '%s'. Expected Vector, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type)); }
		}
		else if (StructProperty->Struct == TBaseStructure<FRotator>::Get())
		{
			if (PropValue.Type == ESpacetimeDBPropertyType::Rotator)
			{
				*static_cast<FRotator*>(PropertyAddress) = PropValue.AsRotator();
			}
			else { UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for FRotator property '%s'. Expected Rotator, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type)); }
		}
		else if (StructProperty->Struct == TBaseStructure<FTransform>::Get())
		{
			if (PropValue.Type == ESpacetimeDBPropertyType::Transform)
			{
				*static_cast<FTransform*>(PropertyAddress) = PropValue.AsTransform();
			}
			else { UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Type mismatch for FTransform property '%s'. Expected Transform, got %s"), *PropertyName, *UEnum::GetValueAsString(PropValue.Type)); }
		}
		// Potentially handle other FStructs here if they have direct ESpacetimeDBPropertyType counterparts
		// For more complex structs or those represented as JSON, we might need to use a helper.
		else if (PropValue.Type == ESpacetimeDBPropertyType::Custom || PropValue.Type == ESpacetimeDBPropertyType::Array || PropValue.Type == ESpacetimeDBPropertyType::Map)
		{
			// If the property is a USTRUCT and the value is JSON, attempt to set it using the helper.
			// This assumes the FString returned by AsJson() is the correct format.
			FString JsonString = PropValue.AsJson();
			if (!FSpacetimeDBPropertyHelper::SetPropertyValueByName(Owner, PropertyName, JsonString))
			{
				UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Failed to set struct property '%s' from JSON."), *PropertyName);
			}
		}
		else
		{
			UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Unhandled struct property type or mismatch for '%s'. Struct: %s, ValueType: %s"), *PropertyName, *StructProperty->Struct->GetName(), *UEnum::GetValueAsString(PropValue.Type));
		}
	}
	// Add handling for other property types as needed (e.g., FArrayProperty, FMapProperty, FObjectProperty).
	else
	{
		// Log a warning for unhandled property types.
		UE_LOG(LogSpacetimeDB, Warning, TEXT("ApplyServerUpdate: Unhandled property type for '%s': %s"), *PropertyName, *Property->GetClass()->GetName());
	}
}
