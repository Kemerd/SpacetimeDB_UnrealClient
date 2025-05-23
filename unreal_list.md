Key:
[ ] - Pending
[X] - Completed

---

# Unreal Engine C++ Integration - Task List

This document outlines the tasks required on the Unreal Engine C++ side to fully integrate with the SpacetimeDB Rust client module and achieve the described replication functionality. This list assumes that the issues and placeholders identified in `fix_list.md` (Rust side) will be addressed.

## I. Core FFI Integration & Setup:

1.  [X] **Build System Integration:**
    *   [X] Ensure `SpacetimeDB_UnrealClient.Build.cs` correctly links against the compiled Rust `ClientModule` library (`stdb_client.lib` or `libstdb_client.a`).
    *   [X] Manage include paths for the CXX-generated C++ header for the FFI bridge (`ffi.rs` output).

2.  [X] **FFI Initialization & Shutdown:**
    *   [X] In a central Unreal subsystem (e.g., `USpacetimeDBSubsystem`):
        *   [X] On subsystem initialization, call the Rust FFI function `connect_to_server`, providing:
            *   [X] `ConnectionConfig` (Host, DB Name, Auth Token - likely from Unreal project settings or runtime configuration).
            *   [X] `EventCallbackPointers`: Pointers to static C++ wrapper functions that will handle callbacks from Rust.
        *   [X] On subsystem deinitialization, call the Rust FFI function `disconnect_from_server()`.

3.  [X] **Implement C++ Callback Wrappers:**
    *   [X] Create static C++ functions matching the signatures expected by Rust for each callback defined in `ClientModule/src/ffi.rs` (`EventCallbackPointers`). These functions will be passed to Rust via FFI.
    *   [X] These C++ wrappers should:
        *   [X] Convert Rust FFI data types (e.g., `c_char*`, `u64`) to appropriate C++/Unreal types (e.g., `FString`, `uint64`).
        *   [X] Safely queue the event data to be processed on the Unreal game thread (e.g., using a thread-safe queue or `AsyncTask(ENamedThreads::GameThread, ...)`).
        *   [X] **Callback Types:** `on_connected`, `on_disconnected`, `on_property_updated`, `on_object_created`, `on_object_destroyed`, `on_error_occurred`.

## II. Data Type Management & Serialization:

1.  [X] **Shared Type Mirroring:**
    *   [X] Define C++ structs equivalent to those in `SharedModule/src/types.rs` (e.g., `FStdbVector3`, `FStdbRotator`, `FStdbTransform`, `FStdbColor`) if direct mapping to Unreal types like `FVector`, `FRotator` isn't perfectly 1:1 or if intermediate structures are needed.
    *   [X] Implement conversion functions between these C++ structs and native Unreal types.

TODO USE PROTOBUFS INSTEAD

2.  [X] **JSON Processing for Properties & RPCs:**
    *   [X] **Crucial:** Develop robust C++ utility functions using Unreal's JSON libraries (`FJsonSerializer`, `FJsonDeserializer`, `TJsonReader`, `TJsonWriter`, `FJsonObject`, `FJsonValue`) to handle the JSON strings used extensively by the Rust modules for:
        *   [X] Property values (especially for containers like `TArray`, `TMap`, and custom UStructs which are passed as `PropertyValue::ArrayJson`, `MapJson`, `CustomJson`).
        *   [X] RPC arguments (`RpcCall::arguments_json`).
        *   [X] RPC results (`RpcResponse::result_json`).
        *   [X] Initial properties in `SpawnParams` and `ObjectDescription`.
    *   [X] This includes serializing Unreal data to JSON for `set_property` and `call_server_function` calls, and deserializing JSON from server updates/RPCs.

3.  [X] **`PropertyValue` Handling:**
    *   [X] Create C++ logic to interpret the `PropertyValue` enum (from `SharedModule`) received from Rust (likely as part of a JSON string or via a dedicated FFI struct).
    *   [X] Develop a system to apply these `PropertyValue` updates to Unreal UObject properties, handling type conversions and JSON deserialization for complex types.

## III. Subsystem Implementation (`USpacetimeDBSubsystem`):

This subsystem will be the main interface for Unreal C++ and Blueprints to interact with SpacetimeDB.

1.  [X] **Connection Management:**
    *   [X] Expose BlueprintCallable functions: `Connect(FString Host, FString DbName, FString AuthToken)`, `Disconnect()`, `IsConnected() -> bool`, `GetSpacetimeDBClientID() -> int64`.
    *   [X] Internally, these will call the FFI functions: `stdb::ffi::connect_to_server`, `stdb::ffi::disconnect_from_server`, `stdb::ffi::is_connected`, `stdb::ffi::get_client_id`.
    *   [X] Manage and broadcast Unreal delegates for connection events (e.g., `OnConnectedToSpacetimeDB`, `OnDisconnectedFromSpacetimeDB`, `OnSpacetimeDBError`).

2.  [X] **Object & Actor Lifecycle Management (Client-Side Mirroring):**
    *   [X] Maintain a TMap of `uint64 (ObjectId)` to `UObject*` (or a custom C++ wrapper class) for all replicated objects/actors known to the client.
    *   [X] **Object Creation (`on_object_created` callback):**
        *   [X] When Rust signals object creation via the FFI callback:
            *   [X] Deserialize the object description (likely JSON including `class_name`, initial properties, transform for actors).
            *   [X] Dynamically spawn the appropriate Unreal UObject/AActor using `StaticConstructObject_Internal` or `GetWorld()->SpawnActorDeferred` etc., based on `class_name`.
            *   [X] Apply initial properties (deserializing from JSON if needed).
            *   [X] If an actor, set its transform.
            *   [X] Store the `ObjectId` and the spawned `UObject*` in the local map.
            *   [X] Finalize spawning if `SpawnActorDeferred` was used.
    *   [X] **Object Destruction (`on_object_destroyed` callback):**
        *   [X] When Rust signals object destruction:
            *   [X] Find the corresponding `UObject*` using `ObjectId`.
            *   [X] Destroy the Unreal actor/object (e.g., `AActor::Destroy()`).
            *   [X] Remove from the local map.
    *   [X] **Spawning Local Requests (e.g., player input to spawn something):**
        *   [X] Provide C++/Blueprint functions e.g., `RequestSpawnActor(UClass* Class, FStdbSpawnParams Params)`.
        *   [X] This function will call `stdb::ffi::create_object(className, paramsJson)`.
        *   [X] Handle the asynchronous nature: the actor isn't truly spawned until the server confirms and the `on_object_created` callback fires.
    *   [X] **Destroying Local Requests:**
        *   [X] Provide C++/Blueprint functions e.g., `RequestDestroyActor(AActor* Actor)`.
        *   [X] This function will need to get the `ObjectId` for the `Actor` and call `stdb::ffi::destroy_object(objectId)`.

3.  [X] **Property Replication (Client-Side Application):**
    *   [X] **`on_property_updated` callback:**
        *   [X] When Rust signals a property update:
            *   [X] Receive `ObjectId`, `PropertyName` (FString), and `ValueJson` (FString).
            *   [X] Find the local `UObject*` using `ObjectId`.
            *   [X] Find the `FProperty*` on the `UObject` using `PropertyName`.
            *   [X] Deserialize `ValueJson` into a temporary C++ representation or directly into the property's memory using appropriate Unreal property functions and the JSON utilities.
            *   [X] Handle type checking and conversion carefully.
            *   [X] For UStructs, TArrays, TMaps, this will involve more complex JSON deserialization.
            *   [X] Trigger any relevant RepNotify functions for the updated property.
    *   [X] **Sending Property Updates (Client to Server):**
        *   [X] When a replicated property changes on a client (that has authority or for client-side prediction that needs server validation):
            *   [X] Get the `ObjectId`, `PropertyName`.
            *   [X] Serialize the new property value to a JSON string.
            *   [X] Call `stdb::ffi::set_property(objectId, propertyName, valueJson, bReplicateToServer)`.
        *   [X] This requires a system to hook into Unreal's property system or for game code to explicitly call this.

4.  [X] **RPC Handling:**
    *   [X] **Calling Server RPCs:**
        *   [X] Provide C++/Blueprint functions like `CallServerFunction(UObject* TargetObject, FName FunctionName, const TArray<FStdbRpcArg>& Args)`.
        *   [X] This function will:
            *   [X] Get `ObjectId` for `TargetObject`.
            *   [X] Serialize `Args` into a JSON string.
            *   [X] Call `stdb::ffi::call_server_function(objectId, functionNameStr, argsJson)`.
    *   [X] **Receiving Server-to-Client RPCs:**
        *   [X] The `ClientModule/src/net/mod.rs` needs to be fixed to parse incoming RPC messages and trigger `ClientModule/src/rpc/mod.rs::handle_server_call`.
        *   [X] `ClientModule/src/rpc/mod.rs::handle_server_call` then invokes a registered Rust handler.
        *   [X] To get this to C++, the `ClientModule/src/ffi.rs::register_client_function` allows C++ to pass a function pointer.
        *   [X] C++ needs to register static wrapper functions (similar to event callbacks) for each client-callable RPC.
        *   [X] These C++ RPC handler wrappers will:
            *   [X] Receive `ObjectId`, `FunctionName`, `ArgsJson`.
            *   [X] Find the target `UObject*`.
            *   [X] Deserialize `ArgsJson` into appropriate C++ types.
            *   [X] Call the actual UFunction on the `UObject` (e.g., via `ProcessEvent` or direct C++ call if possible).

## IV. Advanced Features & Considerations:

1.  [X] **Dynamic Class/Property Registration Info:**
    *   [X] Since the Rust server side relies on hardcoded class/property info (a major FIX item for Rust), the C++ client would need a way to provide this information to the Rust client module if dynamic registration is implemented in Rust. This might involve FFI calls during startup to register all known replicated UClasses and their UProperties with their type information and replication settings.
    *   [X] Alternatively, if the server becomes the source of truth for this, the client needs to fetch this schema information after connecting.
    *   [X] We may need to create a .bat or .sh that calls for instance Unreal CMD or some script inside of Unreal to generate some codegen'd file (it could even be a func that codegens rust code, from inside C++)

2.  [X] **Client-Side Prediction & Reconciliation:**
    *   [X] The `README.md` mentions client-side prediction. This is a complex topic not explicitly detailed in the Rust code. If required, C++ would need to:
        *   [X] Store pre-update state.
        *   [X] Apply input locally and predict changes.
        *   [X] When authoritative server state arrives (property updates), reconcile any differences.
        *   [X] Can possible use 1 euro algorithm as it is very popular

3.  [X] **Ownership & Authority:**
    *   [X] Implement logic to respect object/actor ownership as defined by the server (`owner_id`, `owner_identity`).
    *   [X] Only allow sending property updates or certain RPCs if the client has authority (e.g., for its PlayerController or owned Pawns).

4.  [X] **Component Replication:**
    *   [X] The current Rust `ClientActor.components` only stores `ObjectId`s. A full C++ implementation would need to:
        *   [X] Spawn/destroy actual `UActorComponent` instances on the client when the server signals component changes for an actor.
        *   [X] Handle replicated properties on these components.

5.  [X] **Error Handling & Logging:**
    *   [X] Properly handle errors returned from FFI calls.
    *   [X] Log SpacetimeDB events and errors using Unreal's logging system (`UE_LOG`).
    *   [X] Expose errors from `on_error_occurred` callback to game code/UI.

6.  [ ] **Threading Model:**
    *   [ ] Ensure all interactions with Unreal Engine objects (UObjects, Actors, UI) from FFI callbacks or network threads are done on the Game Thread.

7.  [X] **Configuration:**
    *   [X] Provide UProject settings (e.g., in Project Settings -> Plugins -> SpacetimeDB) for `SPACETIME_HOST`, `SPACETIME_DBNAME`, `SPACETIME_AUTH_TOKEN`, etc., and read these to configure the connection.

This list is comprehensive and assumes the goal is to match the full feature set implied by the `README.md` and a typical Unreal replication system. The actual C++ work will scale with how completely the Rust modules are implemented. 