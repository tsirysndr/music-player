package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "scanner_lib",
    srcs = ["src/lib.rs"],
    deps = [
      "//settings:settings_lib",
      "//entity:entity_lib",
      "//storage:storage_lib",
      "//types:types_lib",
      "//migration:migration_lib",
    ] + all_crate_deps(),
)