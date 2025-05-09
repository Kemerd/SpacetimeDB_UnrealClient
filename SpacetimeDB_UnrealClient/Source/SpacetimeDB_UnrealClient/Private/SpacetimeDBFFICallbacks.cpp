#include "SpacetimeDBSubsystem.h"
#include "Engine/Engine.h"
#include "Misc/CoreDelegates.h"

// External C++ function declarations for FFI callbacks
extern "C" {
    // ... other existing callback declarations ...

    /**
     * Callback for when a property is updated on an object
     * @param object_id The ID of the object that was updated
     * @param property_name The name of the property that was updated
     * @param value_json The JSON representation of the new property value
     */
    void on_property_updated(uint64_t object_id, const char* property_name, const char* value_json);
}

// ... other existing callbacks ...

void on_property_updated(uint64_t object_id, const char* property_name, const char* value_json)
{
    // Convert C strings to Unreal strings
    FString PropertyName = UTF8_TO_TCHAR(property_name);
    FString ValueJson = UTF8_TO_TCHAR(value_json);

    // We need to ensure this callback happens on the game thread
    FFunctionGraphTask::CreateAndDispatchWhenReady([=]() {
        // Find the SpacetimeDB subsystem
        UGameInstance* GameInstance = nullptr;
        
        // Try to find a valid game instance
        if (GEngine && GEngine->GameViewport)
        {
            GameInstance = GEngine->GameViewport->GetGameInstance();
        }
        else if (GEngine && GEngine->GetWorldContexts().Num() > 0)
        {
            GameInstance = GEngine->GetWorldContexts()[0].OwningGameInstance;
        }
        
        if (!GameInstance)
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Failed to find GameInstance to handle property update"));
            return;
        }
        
        USpacetimeDBSubsystem* SpacetimeDB = GameInstance->GetSubsystem<USpacetimeDBSubsystem>();
        if (SpacetimeDB)
        {
            SpacetimeDB->HandlePropertyUpdate(object_id, PropertyName, ValueJson);
        }
        else
        {
            UE_LOG(LogTemp, Error, TEXT("SpacetimeDB: Failed to find SpacetimeDBSubsystem to handle property update"));
        }
    }, TStatId(), nullptr, ENamedThreads::GameThread);
} 