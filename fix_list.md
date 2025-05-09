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

3.  [ ] **`ServerModule/src/actor/lifecycle.rs`:**
    *   [ ] `destroy_actor()`:
        *   [ ] Explicitly commented as placeholder for cleanup: `// In a real implementation, you'd have a system to clean up destroyed actors`.
    *   [ ] `cleanup_destroyed_actors()`:
        *   [ ] Timestamp logic for deleting `Destroyed` actors seems to use `created_at` instead of a destruction timestamp, and comment suggests it's placeholder: `// (In a real implementation, you'd check timestamps)`.
        *   [ ] Comment `// In a real implementation, you might clean up related data, or delay deletion further...` indicates incomplete logic.

4.  [ ] **Dynamic Class/Property Discovery (General Server Issue):**
    *   [ ] Both `ServerModule/src/object/class.rs` and `ServerModule/src/actor/init.rs` rely on hardcoding lists of Unreal classes and some of their properties. A "complete replacement" system would require dynamic discovery and registration of all relevant UClasses and UProperties from the active Unreal project, likely via FFI calls from Unreal to Rust during an initialization phase or a build-time code generation step.

## III. Redundancies & Consolidation Issues:

1.  [ ] **`ObjectLifecycleState` Enum:**
    *   [ ] Defined in `SharedModule/src/object.rs`.
    *   [ ] Redefined in `SharedModule/src/lifecycle.rs`.
    *   [ ] Redefined in `ServerModule/src/object/mod.rs`.
    *   [ ] **FIX:** Consolidate to a single definition in `SharedModule/src/lifecycle.rs`. Update all other modules to import and use it from there.

2.  [ ] **`ActorLifecycleState` Enum:**
    *   [ ] Defined in `SharedModule/src/lifecycle.rs`.
    *   [ ] Redefined in `ServerModule/src/actor/mod.rs`.
    *   [ ] **FIX:** Consolidate to the single definition in `SharedModule/src/lifecycle.rs`. Update `ServerModule/src/actor/mod.rs` to import and use it.

3.  [ ] **`PropertyType` Enum:**
    *   [ ] Defined in `SharedModule/src/property.rs`.
    *   [ ] Redefined (almost identically) in `ServerModule/src/property/mod.rs`.
    *   [ ] **FIX:** Remove the definition from `ServerModule/src/property/mod.rs` and have it use the one from `SharedModule/src/property.rs`.

4.  [ ] **`PropertyValue` Enum:**
    *   [ ] Defined in `SharedModule/src/property.rs`.
    *   [ ] Redefined (almost identically, minor difference in struct variants vs. tuple variants) in `ServerModule/src/property/mod.rs`.
    *   [ ] **FIX:** Remove the definition from `ServerModule/src/property/mod.rs` and have it use the one from `SharedModule/src/property.rs`. Ensure struct/tuple variants are harmonized if necessary, or that `SharedModule`'s version is canonical.

5.  [ ] **`ActorId` vs. `ObjectId` Types:**
    *   [ ] `ObjectId` defined in `SharedModule/src/object.rs` (as `u64`).
    *   [ ] `ActorId` defined in `SharedModule/src/actor.rs` (as `u64`).
    *   [ ] `ObjectId` defined in `ServerModule/src/object/mod.rs` (as `u64`).
    *   [ ] `ActorId` defined in `ServerModule/src/actor/mod.rs` (as `u64`).
    *   [ ] **FIX:** Clarify the relationship. If Actors are Objects, `ObjectId` should likely be used for both. Consolidate to a single definition in `SharedModule/src/object.rs` for `ObjectId`. If a distinct `ActorId` is truly needed for some reason separate from its `ObjectId`, this needs clear documentation; otherwise, remove `ActorId`.

6.  [ ] **Actor Tables vs. Object Tables in `ServerModule`:**
    *   [ ] `ServerModule/src/object/mod.rs` defines `ObjectClass`, `ObjectInstance`, `ObjectProperty`.
    *   [ ] `ServerModule/src/actor/mod.rs` defines `ActorClass`, `ActorInfo`, `ActorProperty`.
    *   [ ] There's significant conceptual overlap. If Actors are a specialization of Objects, consider if actor-specific data can be stored in distinct tables linked by `ObjectId` or if the Object tables can be extended/used more directly to avoid near-duplicate table structures.
    *   [ ] **FIX:** Review and potentially refactor to reduce redundancy and clarify the data model for objects vs. actors on the server. For example, `ActorInfo` could simply be an `ObjectId` that links to an `ObjectInstance` which has an `is_actor` flag.

## IV. Minor Issues & Points to Clarify:

1.  [ ] **`PropertyValue::None` Type (`SharedModule/src/property.rs`):**
    *   [ ] The `get_type()` method for `PropertyValue::None` defaults to `PropertyType::Bool`. Clarify if `None` should have its own `PropertyType` or if this default is intentional and consistently handled.

2.  [ ] **`ActorID` in `README.md` Example:**
    *   [ ] The `README.md` C++ example uses `FObjectID ActorID`. Ensure this aligns with the chosen `ObjectId`/`ActorId` type from Rust once consolidated.

3.  [ ] **Configuration Constants (`SharedModule/src/constants.rs` vs. `README.md`):**
    *   [ ] Constants like `MAX_OBJECTS` are compile-time in `constants.rs`. The `README.md` implies runtime configuration (e.g., `SPACETIME_MAX_OBJECTS`). Clarify how these are intended to work (e.g., are constants defaults, overridden by runtime config?).

4.  [ ] **Static ID Generator (`ServerModule/src/actor/spawn.rs`):**
    *   [ ] The `unsafe static mut NEXT_ACTOR_ID` is simple. Consider if SpacetimeDB offers more robust unique ID generation mechanisms or if this needs to be persisted across server restarts.

---
## V. ClientModule Evaluation (To Be Added)

1.  [ ] *(Placeholder for ClientModule analysis)* 

---
## V. Critical Missing Client-Side Functionality:

1.  [ ] **`ClientModule/src/net/mod.rs` - Incoming Data Processing:**
    *   [ ] The mechanism for receiving, parsing, and dispatching specific SpacetimeDB table updates (which represent property changes, object creation/destruction) and server-to-client RPCs to the registered FFI callbacks (e.g., `invoke_on_property_updated`, `invoke_on_object_created` via `ffi.rs`) or client RPC handlers (`ClientModule/src/rpc/mod.rs::handle_server_call`) is largely missing or not apparent. This is essential for the client to reflect server state.
    *   [ ] While `client.on_subscription_applied()` exists, the continuous processing of differential updates from subscriptions needs to be clearly implemented and tied to the FFI event callbacks.

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