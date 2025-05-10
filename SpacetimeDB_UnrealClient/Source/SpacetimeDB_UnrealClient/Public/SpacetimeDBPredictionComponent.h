#pragma once

#include "CoreMinimal.h"
#include "Components/ActorComponent.h"
#include "SpacetimeDB_Types.h"
#include "SpacetimeDBPredictionComponent.generated.h"

/**
 * Structure to store the pre-update state for reconciliation
 */
USTRUCT()
struct FStateSnapshot
{
	GENERATED_BODY()

	/** The timestamp when this state was captured */
	UPROPERTY()
	float Timestamp = 0.0f;

	/** The transform at the time of capture */
	UPROPERTY()
	FTransform Transform;

	/** The velocity at the time of capture */
	UPROPERTY()
	FVector Velocity = FVector::ZeroVector;

	/** Custom state data (can be extended by game code) */
	UPROPERTY()
	TMap<FName, FSpacetimeDBPropertyValue> CustomState;

	/** The input state that led to this snapshot */
	UPROPERTY()
	TMap<FName, float> InputState;

	/** The sequence number of this snapshot - used to match with server acks */
	UPROPERTY()
	int32 SequenceNumber = 0;
};

/**
 * Component that handles client-side prediction and reconciliation for SpacetimeDB
 * This component should be added to actors that need prediction (typically player-controlled pawns)
 */
UCLASS(ClassGroup=(SpacetimeDB), meta=(BlueprintSpawnableComponent))
class SPACETIMEDB_UNREALCLIENT_API USpacetimeDBPredictionComponent : public UActorComponent
{
	GENERATED_BODY()

public:	
	USpacetimeDBPredictionComponent();

	virtual void BeginPlay() override;
	virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;
	virtual void EndPlay(const EEndPlayReason::Type EndPlayReason) override;

	/** 
	 * Take a snapshot of the current state that can be used for reconciliation later
	 * Call this before applying predicted changes
	 */
	UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
	void TakeStateSnapshot();

	/**
	 * Apply the current input to predict state changes locally
	 * This should be called after user input is processed
	 */
	UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
	void ApplyPredictedChanges();

	/**
	 * Process server update and perform reconciliation if needed
	 * This is called automatically when receiving server updates
	 * @param ServerState The authoritative state from the server
	 * @param AckedSequence The sequence number being acknowledged by the server
	 */
	UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
	void ProcessServerUpdate(const FTransform& ServerTransform, const FVector& ServerVelocity, int32 AckedSequence);

	/**
	 * Add a custom property to track for prediction
	 * @param PropertyName The name of the property
	 */
	UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
	void AddTrackedProperty(FName PropertyName);

	/**
	 * Register current input value for prediction
	 * @param InputName Name of the input (e.g., "MoveForward", "MoveRight")
	 * @param Value Current value of the input
	 */
	UFUNCTION(BlueprintCallable, Category = "SpacetimeDB|Prediction")
	void RegisterInputValue(FName InputName, float Value);

	/**
	 * Get the current sequence number
	 */
	UFUNCTION(BlueprintPure, Category = "SpacetimeDB|Prediction")
	int32 GetCurrentSequence() const { return CurrentSequence; }

private:
	/** History of state snapshots for reconciliation */
	UPROPERTY()
	TArray<FStateSnapshot> StateHistory;

	/** Maximum number of history entries to keep */
	UPROPERTY(EditDefaultsOnly, Category = "SpacetimeDB|Prediction", meta = (ClampMin = "1", ClampMax = "120"))
	int32 MaxHistoryLength = 60;

	/** Properties to track for prediction */
	UPROPERTY()
	TArray<FName> TrackedProperties;

	/** Current sequence number for prediction */
	UPROPERTY()
	int32 CurrentSequence = 0;

	/** Current input state */
	UPROPERTY()
	TMap<FName, float> CurrentInputs;

	/** Last acknowledged sequence from server */
	UPROPERTY()
	int32 LastAcknowledgedSequence = -1;

	/** Position error threshold before reconciliation (cm) */
	UPROPERTY(EditDefaultsOnly, Category = "SpacetimeDB|Prediction")
	float PositionErrorThreshold = 5.0f;

	/** Rotation error threshold before reconciliation (degrees) */
	UPROPERTY(EditDefaultsOnly, Category = "SpacetimeDB|Prediction")
	float RotationErrorThreshold = 10.0f;

	/** Velocity error threshold before reconciliation (cm/s) */
	UPROPERTY(EditDefaultsOnly, Category = "SpacetimeDB|Prediction")
	float VelocityErrorThreshold = 10.0f;

	/** How to smooth corrections (0-1, higher = more smoothing) */
	UPROPERTY(EditDefaultsOnly, Category = "SpacetimeDB|Prediction", meta = (ClampMin = "0.0", ClampMax = "0.99"))
	float SmoothingFactor = 0.8f;

	/** Whether this component is authorized to run prediction */
	UPROPERTY()
	bool bHasAuthority = false;

	/** Get property values for the tracked properties */
	void CaptureTrackedProperties(TMap<FName, FSpacetimeDBPropertyValue>& OutProperties);

	/** Apply tracked properties from a snapshot */
	void ApplyTrackedProperties(const TMap<FName, FSpacetimeDBPropertyValue>& Properties);

	/** Clean up old history entries */
	void CleanupHistory();

	/** Handles smooth correction of errors */
	void ApplySmoothCorrection(const FTransform& TargetTransform, const FVector& TargetVelocity, float BlendFactor);

	/** One Euro Filter implementation for smooth corrections */
	struct FOneEuroFilter
	{
		float Value = 0.0f;
		float LastValue = 0.0f;
		float LastRawValue = 0.0f;
		float LastTime = 0.0f;
		float MinCutoff = 1.0f;
		float Beta = 0.0f;
		float DCutoff = 1.0f;

		FOneEuroFilter() {}
		FOneEuroFilter(float InMinCutoff, float InBeta) : MinCutoff(InMinCutoff), Beta(InBeta) {}

		float Filter(float InValue, float InDeltaTime);
	};

	/** One Euro Filters for position components */
	FOneEuroFilter PositionFilterX = FOneEuroFilter(0.5f, 0.8f);
	FOneEuroFilter PositionFilterY = FOneEuroFilter(0.5f, 0.8f);
	FOneEuroFilter PositionFilterZ = FOneEuroFilter(0.5f, 0.8f);
}; 