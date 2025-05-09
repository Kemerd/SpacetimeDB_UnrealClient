Key:
[ ] - Pending
[X] - Completed

---

# SpacetimeDB Unreal Client - Fix List

## I. Critical Missing Server-Side Functionality:

The following core modules/functionalities are declared in `ServerModule/src/lib.rs` and/or the `README.md` but their corresponding directories in `ServerModule/src/` are empty or implementations are critically incomplete:

1.  [X] **`ServerModule/src/connection/` is empty:**
    *   [X] Missing logic for `connection::handlers::register_client()` and `connection::handlers::unregister_client()` (called by `client_connected`/`client_disconnected` reducers in `ServerModule/src/lib.rs`).
    *   [X] Missing actual authentication/authorization logic (e.g., `connection::auth::can_spawn_actor()`, `connection::auth::is_admin()` used in `ServerModule/src/actor/spawn.rs` and `ServerModule/src/actor/lifecycle.rs`).
    *   [X] No handling of `ConnectionParams` from `SharedModule`.

2.  [X] **`ServerModule/src/rpc/` is empty:**
    *   [X] Missing server-side RPC function registration mechanism.
    *   [X] Missing logic to dispatch incoming RPC calls from clients to registered Rust functions.
    *   [X] Missing implementation for sending RPCs from server to client(s) (Multicast, OwnerOnly).

3.  [X] **`ServerModule/src/relevancy/` is empty:**
    *   [X] Missing all logic for network relevancy determination (distance-based, zone-based, owner-based, etc.).
    *   [X] Missing management of `RelevancySettings`, `ZoneMembership` from `SharedModule`.
    *   [X] No integration with property replication to send updates only to relevant clients.

## II. Incomplete Server-Side Implementations:

1.  [X] **`ServerModule/src/property/mod.rs`:**
    *   [X] `set_object_property()` reducer:
        *   [X] Explicitly commented as placeholder: `// In a full implementation, we would check against property constraints`.
        *   [X] Explicitly commented as placeholder: `// In a real implementation, we would access the UObject table and update the property, then track the change for replication`.
        *   [X] Relies on undefined `ClientInfo` and `UObject` tables (or unclear mapping to existing tables like `ObjectInstance`).
    *   [X] `init()` function:
        *   [X] Explicitly commented as placeholder: `// In a real implementation, we would: 1. Load property definitions from configuration 2. Initialize the property registry 3. Set up replication schedules`.

2.  [X] **`ServerModule/src/actor/spawn.rs`:**
    *   [X] `initialize_default_components()`:
        *   [X] Explicitly commented as placeholder: `// In a real implementation, you'd have a system to define which components are needed for each class`.
        *   [X] Uses hardcoded `component_class_id: 101`.

3.  [X] **`ServerModule/src/actor/lifecycle.rs`:**
    *   [X] `destroy_actor()`:
        *   [X] Replaced placeholder comment with proper implementation including destruction timestamp tracking and notification of relevant systems.
    *   [X] `cleanup_destroyed_actors()`:
        *   [X] Fixed timestamp logic to use proper `destroyed_at` timestamp instead of `created_at`.
        *   [X] Implemented comprehensive cleanup logic for destroying actors, including components, properties, transform data, relevancy data, and RPC handlers.

4.  [X] **Dynamic Class/Property Discovery (General Server Issue):**
    *   [X] Implemented a proper code generation system where:
        *   [X] The SpacetimeDBCodeGenerator is the single source of truth for all class definitions (both core engine and game-specific)
        *   [X] Removed hardcoded class definitions from the Rust codebase
        *   [X] Updated the code generator to explicitly generate code for both core engine classes and game-specific classes
        *   [X] Created a clean separation between core engine classes (IDs 1-99) and game-specific classes (IDs 100+)

## III. Redundancies & Consolidation Issues:

1.  [X] **`ObjectLifecycleState` Enum:**
    *   [X] Defined in `SharedModule/src/object.rs`.
    *   [X] Redefined in `SharedModule/src/lifecycle.rs`.
    *   [X] Redefined in `ServerModule/src/object/mod.rs`.
    *   [X] **FIX:** Consolidated to a single definition in `SharedModule/src/lifecycle.rs`. Updated all other modules to import and use it from there. Modified `ClientModule/src/object/mod.rs` to import from `stdb_shared::lifecycle` instead of `stdb_shared::object`.

2.  [X] **`ActorLifecycleState` Enum:**
    *   [X] Defined in `SharedModule/src/lifecycle.rs`.
    *   [X] Redefined in `ServerModule/src/actor/mod.rs`.
    *   [X] **FIX:** Consolidated to the single definition in `SharedModule/src/lifecycle.rs`. Updated `ServerModule/src/actor/mod.rs` to import and use it from `stdb_shared::lifecycle`.

3.  [X] **`PropertyType` Enum:**
    *   [X] Defined in `SharedModule/src/property.rs`.
    *   [X] Redefined (almost identically) in `ServerModule/src/property/mod.rs`.
    *   [X] **FIX:** Removed the definition from `ServerModule/src/property/mod.rs` and fixed the import to use the one from `stdb_shared::property` instead of the incorrect `crate::SharedModule::property`.

4.  [X] **`PropertyValue` Enum:**
    *   [X] Defined in `SharedModule/src/property.rs`.
    *   [X] Redefined (almost identically, minor difference in struct variants vs. tuple variants) in `ServerModule/src/property/mod.rs`.
    *   [X] **FIX:** Removed the redundant import in `ServerModule/src/object/mod.rs` and updated it to use the canonical version from `stdb_shared::property` instead of importing from the local property module.

5.  [X] **`ActorId` vs. `ObjectId` Types:**
    *   [X] `ObjectId` defined in `SharedModule/src/object.rs` (as `u64`).
    *   [X] `ActorId` defined in `SharedModule/src/actor.rs` (as `u64`).
    *   [X] `ObjectId` defined in `ServerModule/src/object/mod.rs` (as `u64`).
    *   [X] `ActorId` defined in `ServerModule/src/actor/mod.rs` (as `u64`).
    *   [X] **FIX:** Consolidated to use a single `ObjectId` type from `stdb_shared::object` module. Removed the `ActorId` type definitions and updated all references to use `ObjectId` instead. This reflects the inheritance hierarchy in Unreal Engine where Actors are a type of Object and should use the same ID system.

6.  [X] **Actor Tables vs. Object Tables in `ServerModule`:**
    *   [X] `ServerModule/src/object/mod.rs` defines `ObjectClass`, `ObjectInstance`, `ObjectProperty`.
    *   [X] `ServerModule/src/actor/mod.rs` defines `ActorClass`, `ActorInfo`, `ActorProperty`.
    *   [X] There's significant conceptual overlap. If Actors are a specialization of Objects, consider if actor-specific data can be stored in distinct tables linked by `ObjectId` or if the Object tables can be extended/used more directly to avoid near-duplicate table structures.
    *   [X] **FIX:** Consolidated actor and object tables by:
        *   Adding `is_actor: bool` flag to `ObjectInstance` table to distinguish actors from non-actor objects
        *   Moving actor-specific fields from `ActorInfo` into `ObjectInstance` (e.g., `hidden`)
        *   Renaming `ActorTransform` to `ObjectTransform` and updating field references
        *   Renaming `ActorComponent` to `ObjectComponent` and updating field references
        *   Removing the redundant actor-specific tables and updating all code to use the consolidated object tables
        *   Keeping `ActorId` as a type alias for `ObjectId` for backward compatibility and code clarity

## IV. Minor Issues & Points to Clarify:

1.  [X] **`PropertyValue::None` Type (`SharedModule/src/property.rs`):**
    *   [X] The `get_type()` method for `PropertyValue::None` defaults to `PropertyType::Bool`. Clarify if `None` should have its own `PropertyType` or if this default is intentional and consistently handled.
    *   [X] **FIX:** Added `PropertyType::None` enum variant and updated `get_type()` to return `PropertyType::None` for `PropertyValue::None` for greater type consistency and clarity.

2.  [X] **`ActorID` in `README.md` Example:**
    *   [X] The `README.md` C++ example uses `FObjectID ActorID`. Ensure this aligns with the chosen `ObjectId`/`ActorId` type from Rust once consolidated.
    *   [X] **FIX:** Updated the README.md examples to consistently use `FObjectID ObjectID` naming to align with the Rust codebase's use of `ObjectId` for both actors and non-actor objects.

3.  [X] **Configuration Constants (`SharedModule/src/constants.rs` vs. `README.md`):**
    *   [X] Constants like `MAX_OBJECTS` are compile-time in `constants.rs`. The `README.md` implies runtime configuration (e.g., `SPACETIME_MAX_OBJECTS`). Clarify how these are intended to work (e.g., are constants defaults, overridden by runtime config?).
    *   [X] **FIX:** Enhanced `constants.rs` to support both compile-time defaults and runtime overrides via environment variables. Added getter functions like `get_max_objects()` that check for environment variables like `SPACETIME_MAX_OBJECTS` before falling back to compile-time constants.

4.  [X] **Static ID Generator (`ServerModule/src/actor/spawn.rs`):**
    *   [X] The `unsafe static mut NEXT_ACTOR_ID` is simple. Consider if SpacetimeDB offers more robust unique ID generation mechanisms or if this needs to be persisted across server restarts.
    *   [X] **FIX:** Replaced the static ID generator with a persistent `IdCounter` table in SpacetimeDB. Added `initialize_id_counter` reducer to set up the counter and improved `generate_actor_id` to use the database-backed counter, ensuring IDs persist across server restarts and remain unique.


---
## V. Critical Missing Client-Side Functionality:

1.  [X] **`ClientModule/src/net/mod.rs` - Incoming Data Processing:**
    *   [X] The mechanism for receiving, parsing, and dispatching specific SpacetimeDB table updates (which represent property changes, object creation/destruction) and server-to-client RPCs to the registered FFI callbacks (e.g., `invoke_on_property_updated`, `invoke_on_object_created` via `ffi.rs`) or client RPC handlers (`ClientModule/src/rpc/mod.rs::handle_server_call`) is largely missing or not apparent. This is essential for the client to reflect server state.
    *   [X] While `client.on_subscription_applied()` exists, the continuous processing of differential updates from subscriptions needs to be clearly implemented and tied to the FFI event callbacks.

2.  [ ] **`ClientModule/src/object/mod.rs` - Server Interaction for Lifecycle:**
    *   [ ] `create_object()`: Explicitly commented as placeholder: `// In a real implementation, we would request the server to create an object`. Currently only creates a local object with a temporary ID.
    *   [ ] `destroy_object()`: Explicitly commented as placeholder: `// In a real implementation, we would request the server to destroy the object`. Currently only updates local state.
    *   [ ] Missing logic for remapping temporary client-generated IDs to server-authoritative IDs upon creation confirmation.

3.  [ ] **`ClientModule/src/rpc/mod.rs` - Sending RPCs to Server:**
    *   [ ] `call_server_function()` & `send_rpc_to_server()`: Explicitly commented as placeholders: `// In a real implementation, we would use spacetimedb_sdk to call a reducer`. Client-to-server RPCs are not functionally implemented to use the SpacetimeDB SDK.

4.  [ ] **`ClientModule/src/object/mod.rs` - Incomplete Transform Handling:**
    *   [ ] Rotation conversion from `PropertyValue::Rotator` to `Quat` within `update_object_property` is a placeholder: `let quat = Quat::identity(); // Placeholder` and comment `// In a real implementation, we'd use proper conversion`.

## VI. Client-Side Redundancies & Design Points:

1.  [ ] **`ClientModule/src/net/mod.rs` - Redundant Connection Types:**
    *   [ ] `ConnectionState`, `ConnectionParams`, `ClientConnection` are redefined. 
    *   [ ] **FIX:** Use the definitions from `SharedModule/src/connection.rs`.

2.  [ ] **`ClientModule/src/actor/mod.rs` & `object/mod.rs` - Object/Actor Representation:**
    *   [ ] `ClientActor` struct is very similar to `ClientObject`. There are separate caches (`CLIENT_ACTORS`, `CLIENT_OBJECTS`).
    *   [ ] **FIX:** Consider consolidating into a single `ClientObject` representation and cache, using flags or component patterns to denote actor-specifics, mirroring the server-side consolidation suggestion.

3.  [ ] **Dual Property Storage (Client-Side):**
    *   [ ] Properties appear to be stored in `ClientModule/src/property/PROPERTY_CACHE` and also within `ClientModule/src/object/ClientObject.properties` (and subsequently potentially in `ClientActor` if it mirrors `ClientObject` properties).
    *   [ ] **FIX:** Clarify the need for dual storage. If `ClientObject` (or its consolidated form) is the primary representation, it should likely hold the authoritative client-side state for its properties, with the `property` module providing access/serialization utilities. The `PROPERTY_CACHE` might be redundant or its role needs to be clearly defined (e.g., only for raw incoming values before association with an object).

4.  [ ] **Property Definitions Population (`ClientModule/src/property/mod.rs`):**
    *   [ ] `PROPERTY_DEFINITIONS` cache: It's unclear how this cache is populated with all necessary property definitions for dynamic Unreal types. 
    *   [ ] **FIX (More of a Design Requirement):** Implement a mechanism for the client to obtain comprehensive class and property definitions (e.g., from the server during connection, or from a shared configuration generated during a build step).

5.  [ ] **Component System (`ClientModule/src/actor/mod.rs`):**
    *   [ ] The current system in `ClientActor` only tracks `ObjectId`s of components (`Vec<ObjectId>`).
    *   [ ] **FIX (More of a Design Requirement):** A more complete client-side component system would involve managing actual `ClientObject` (or equivalent) instances for components, allowing them to have their own properties and potentially replicated state.

## VII. FFI Layer (`ClientModule/src/ffi.rs`):

1.  [ ] **Completeness of Implementations:**
    *   [ ] While the FFI bridge definition in `ffi.rs` looks reasonable, the actual Rust implementations of many of the `extern "Rust"` functions (e.g., `