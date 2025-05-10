#include "SpacetimeDBPredictionComponent.h"
#include "GameFramework/Actor.h"
#include "GameFramework/Character.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "SpacetimeDBSubsystem.h"
#include "Engine/World.h"
#include "SpacetimeDBPropertyHelper.h"

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
	AActor* Owner = GetOwner();
	if (!Owner)
	{
		return;
	}

	// Get property values for all tracked properties
	for (const FName& PropName : TrackedProperties)
	{
		FString JsonValue = FSpacetimeDBPropertyHelper::GetPropertyValueByName(Owner, PropName.ToString());
		if (!JsonValue.IsEmpty())
		{
			OutProperties.Add(PropName, JsonValue);
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
