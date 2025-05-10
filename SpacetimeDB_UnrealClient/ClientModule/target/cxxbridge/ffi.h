#pragma once
#include "UnrealReplication.h"
#include "bridge.h"
#include <array>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <type_traits>
#if __cplusplus >= 201703L
#include <string_view>
#endif

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

struct unsafe_bitcopy_t;

namespace {
template <typename T>
class impl;
} // namespace

#ifndef CXXBRIDGE1_RUST_STRING
#define CXXBRIDGE1_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, std::size_t);
  String(const char16_t *);
  String(const char16_t *, std::size_t);
#ifdef __cpp_char8_t
  String(const char8_t *s);
  String(const char8_t *s, std::size_t len);
#endif

  static String lossy(const std::string &) noexcept;
  static String lossy(const char *) noexcept;
  static String lossy(const char *, std::size_t) noexcept;
  static String lossy(const char16_t *) noexcept;
  static String lossy(const char16_t *, std::size_t) noexcept;

  String &operator=(const String &) & noexcept;
  String &operator=(String &&) & noexcept;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  const char *c_str() noexcept;

  std::size_t capacity() const noexcept;
  void reserve(size_t new_cap) noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const String &) const noexcept;
  bool operator!=(const String &) const noexcept;
  bool operator<(const String &) const noexcept;
  bool operator<=(const String &) const noexcept;
  bool operator>(const String &) const noexcept;
  bool operator>=(const String &) const noexcept;

  void swap(String &) noexcept;

  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  struct lossy_t;
  String(lossy_t, const char *, std::size_t) noexcept;
  String(lossy_t, const char16_t *, std::size_t) noexcept;
  friend void swap(String &lhs, String &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) & noexcept = default;

  explicit operator std::string() const;
#if __cplusplus >= 201703L
  explicit operator std::string_view() const;
#endif

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR
} // namespace cxxbridge1
} // namespace rust

#if __cplusplus >= 201402L
#define CXX_DEFAULT_VALUE(value) = value
#else
#define CXX_DEFAULT_VALUE(value)
#endif

namespace stdb {
  namespace ffi {
    enum class ConnectionState : ::std::uint8_t;
    enum class ReplicationCondition : ::std::uint8_t;
    struct ConnectionConfig;
    struct EventCallbackPointers;
  }
}

namespace stdb {
namespace ffi {
#ifndef CXXBRIDGE1_ENUM_stdb$ffi$ConnectionState
#define CXXBRIDGE1_ENUM_stdb$ffi$ConnectionState
enum class ConnectionState : ::std::uint8_t {
  Disconnected = 0,
  Connecting = 1,
  Connected = 2,
};
#endif // CXXBRIDGE1_ENUM_stdb$ffi$ConnectionState

#ifndef CXXBRIDGE1_ENUM_stdb$ffi$ReplicationCondition
#define CXXBRIDGE1_ENUM_stdb$ffi$ReplicationCondition
enum class ReplicationCondition : ::std::uint8_t {
  Never = 0,
  OnChange = 1,
  Initial = 2,
  Always = 3,
};
#endif // CXXBRIDGE1_ENUM_stdb$ffi$ReplicationCondition

#ifndef CXXBRIDGE1_STRUCT_stdb$ffi$ConnectionConfig
#define CXXBRIDGE1_STRUCT_stdb$ffi$ConnectionConfig
struct ConnectionConfig final {
  ::rust::String host;
  ::rust::String db_name;
  ::rust::String auth_token;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_stdb$ffi$ConnectionConfig

#ifndef CXXBRIDGE1_STRUCT_stdb$ffi$EventCallbackPointers
#define CXXBRIDGE1_STRUCT_stdb$ffi$EventCallbackPointers
struct EventCallbackPointers final {
  ::std::size_t on_connected CXX_DEFAULT_VALUE(0);
  ::std::size_t on_disconnected CXX_DEFAULT_VALUE(0);
  ::std::size_t on_property_updated CXX_DEFAULT_VALUE(0);
  ::std::size_t on_object_created CXX_DEFAULT_VALUE(0);
  ::std::size_t on_object_destroyed CXX_DEFAULT_VALUE(0);
  ::std::size_t on_error_occurred CXX_DEFAULT_VALUE(0);
  ::std::size_t on_object_id_remapped CXX_DEFAULT_VALUE(0);
  ::std::size_t on_event_received CXX_DEFAULT_VALUE(0);
  ::std::size_t on_component_added CXX_DEFAULT_VALUE(0);
  ::std::size_t on_component_removed CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_stdb$ffi$EventCallbackPointers

bool create_class(::std::string const &class_name, ::std::string const &parent_class_name) noexcept;

bool add_property(::std::string const &class_name, ::std::string const &property_name, ::std::string const &type_name, bool replicated, ::stdb::ffi::ReplicationCondition replication_condition, bool readonly, ::std::uint32_t flags) noexcept;

::std::unique_ptr<::std::string> get_property_definition(::std::string const &class_name, ::std::string const &property_name) noexcept;

::std::unique_ptr<::std::string> get_property_names_for_class(::std::string const &class_name) noexcept;

::std::unique_ptr<::std::string> get_registered_class_names() noexcept;

::std::unique_ptr<::std::string> export_property_definitions_as_json() noexcept;

bool import_property_definitions_from_json(::std::string const &json) noexcept;

::std::uint64_t register_object(::std::string const &class_name, ::std::string const &params) noexcept;

::std::unique_ptr<::std::string> get_object_class(::std::uint64_t object_id) noexcept;

bool set_property(::std::uint64_t object_id, ::std::string const &property_name, ::std::string const &value_json, bool replicate) noexcept;

::std::unique_ptr<::std::string> get_property(::std::uint64_t object_id, ::std::string const &property_name) noexcept;

bool dispatch_unreliable_rpc(::std::uint64_t object_id, ::std::string const &function_name, ::std::string const &params) noexcept;

bool connect_to_server(::stdb::ffi::ConnectionConfig config, ::stdb::ffi::EventCallbackPointers callbacks) noexcept;

bool disconnect_from_server() noexcept;

bool is_connected() noexcept;

::rust::String get_client_identity() noexcept;
} // namespace ffi
} // namespace stdb
