"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of normal dependencies for the Rust targets of that package.
_DEPENDENCIES = {
    "addons": {
    },
    "client": {
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "tokio-tungstenite": "//cargo/vendor/tokio-tungstenite-0.17.2:tokio_tungstenite",
        "tonic": "//cargo/vendor/tonic-0.8.2:tonic",
        "url": "//cargo/vendor/url-2.3.1:url",
    },
    "server": {
        "chrono": "//cargo/vendor/chrono-0.4.23:chrono",
        "cuid": "//cargo/vendor/cuid-1.2.0:cuid",
        "futures": "//cargo/vendor/futures-0.3.24:futures",
        "futures-channel": "//cargo/vendor/futures-channel-0.3.25:futures_channel",
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "owo-colors": "//cargo/vendor/owo-colors-3.5.0:owo_colors",
        "prost": "//cargo/vendor/prost-0.11.0:prost",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "tokio-tungstenite": "//cargo/vendor/tokio-tungstenite-0.17.2:tokio_tungstenite",
        "tonic": "//cargo/vendor/tonic-0.8.2:tonic",
        "tonic-web": "//cargo/vendor/tonic-web-0.4.0:tonic_web",
        "tungstenite": "//cargo/vendor/tungstenite-0.17.3:tungstenite",
        "uuid": "//cargo/vendor/uuid-1.2.1:uuid",
    },
    "entity": {
        "chrono": "//cargo/vendor/chrono-0.4.23:chrono",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
    },
    "types": {
        "lofty": "//cargo/vendor/lofty-0.9.0:lofty",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "tantivy": "//cargo/vendor/tantivy-0.18.1:tantivy",
    },
    "playback": {
        "cpal": "//cargo/vendor/cpal-0.14.1:cpal",
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "lazy_static": "//cargo/vendor/lazy_static-1.4.0:lazy_static",
        "librespot-protocol": "//cargo/vendor/librespot-protocol-0.4.2:librespot_protocol",
        "log": "//cargo/vendor/log-0.4.17:log",
        "owo-colors": "//cargo/vendor/owo-colors-3.5.0:owo_colors",
        "parking_lot": "//cargo/vendor/parking_lot-0.12.1:parking_lot",
        "rand": "//cargo/vendor/rand-0.8.5:rand",
        "rand_distr": "//cargo/vendor/rand_distr-0.4.3:rand_distr",
        "rb": "//cargo/vendor/rb-0.4.1:rb",
        "rodio": "//cargo/vendor/rodio-0.16.0:rodio",
        "symphonia": "//cargo/vendor/symphonia-0.5.1:symphonia",
        "thiserror": "//cargo/vendor/thiserror-1.0.37:thiserror",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "zerocopy": "//cargo/vendor/zerocopy-0.6.1:zerocopy",
    },
    "tracklist": {
        "atlist-rs": "//cargo/vendor/atlist-rs-0.2.1:atlist_rs",
        "rand": "//cargo/vendor/rand-0.8.5:rand",
    },
    "scanner": {
        "dirs": "//cargo/vendor/dirs-4.0.0:dirs",
        "futures": "//cargo/vendor/futures-0.3.24:futures",
        "lofty": "//cargo/vendor/lofty-0.9.0:lofty",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "mime_guess": "//cargo/vendor/mime_guess-2.0.4:mime_guess",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "walkdir": "//cargo/vendor/walkdir-2.3.2:walkdir",
    },
    "migration": {
        "dotenv": "//cargo/vendor/dotenv-0.15.0:dotenv",
        "sea-orm-migration": "//cargo/vendor/sea-orm-migration-0.9.3:sea_orm_migration",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
    },
    "settings": {
        "config": "//cargo/vendor/config-0.13.2:config",
        "dirs": "//cargo/vendor/dirs-4.0.0:dirs",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
        "toml": "//cargo/vendor/toml-0.5.9:toml",
        "uuid": "//cargo/vendor/uuid-1.2.1:uuid",
    },
    "storage": {
        "itertools": "//cargo/vendor/itertools-0.10.5:itertools",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "tantivy": "//cargo/vendor/tantivy-0.18.1:tantivy",
        "tempfile": "//cargo/vendor/tempfile-3.3.0:tempfile",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
    },
    "discovery": {
        "async-stream": "//cargo/vendor/async-stream-0.3.3:async_stream",
        "env_logger": "//cargo/vendor/env_logger-0.9.1:env_logger",
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "libmdns": "//cargo/vendor/libmdns-0.7.2:libmdns",
        "mdns": "//cargo/vendor/mdns-3.0.0:mdns",
        "mdns-sd": "//cargo/vendor/mdns-sd-0.5.8:mdns_sd",
        "owo-colors": "//cargo/vendor/owo-colors-3.5.0:owo_colors",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
    },
    "graphql": {
        "async-graphql": "//cargo/vendor/async-graphql-4.0.15:async_graphql",
        "async-graphql-tide": "//cargo/vendor/async-graphql-tide-4.0.15:async_graphql_tide",
        "chrono": "//cargo/vendor/chrono-0.4.23:chrono",
        "cuid": "//cargo/vendor/cuid-1.2.0:cuid",
        "futures-channel": "//cargo/vendor/futures-channel-0.3.25:futures_channel",
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "once_cell": "//cargo/vendor/once_cell-1.16.0:once_cell",
        "rand": "//cargo/vendor/rand-0.8.5:rand",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
        "slab": "//cargo/vendor/slab-0.4.7:slab",
        "tide": "//cargo/vendor/tide-0.16.0:tide",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
    },
    "webui": {
        "actix-cors": "//cargo/vendor/actix-cors-0.6.4:actix_cors",
        "actix-files": "//cargo/vendor/actix-files-0.6.2:actix_files",
        "actix-web": "//cargo/vendor/actix-web-4.2.1:actix_web",
        "async-graphql": "//cargo/vendor/async-graphql-4.0.15:async_graphql",
        "async-graphql-actix-web": "//cargo/vendor/async-graphql-actix-web-4.0.15:async_graphql_actix_web",
        "dirs": "//cargo/vendor/dirs-4.0.0:dirs",
        "mime_guess": "//cargo/vendor/mime_guess-2.0.4:mime_guess",
        "owo-colors": "//cargo/vendor/owo-colors-3.5.0:owo_colors",
        "rust-embed": "//cargo/vendor/rust-embed-6.4.1:rust_embed",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
    },
    "webui/musicplayer/src-tauri": {
        "actix-web": "//cargo/vendor/actix-web-4.2.1:actix_web",
        "async-graphql": "//cargo/vendor/async-graphql-4.0.15:async_graphql",
        "async-graphql-actix-web": "//cargo/vendor/async-graphql-actix-web-4.0.15:async_graphql_actix_web",
        "futures": "//cargo/vendor/futures-0.3.24:futures",
        "serde": "//cargo/vendor/serde-1.0.148:serde",
        "serde_json": "//cargo/vendor/serde_json-1.0.89:serde_json",
        "tauri": "//cargo/vendor/tauri-1.2.1:tauri",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "uuid": "//cargo/vendor/uuid-1.2.1:uuid",
    },
    "": {
        "clap": "//cargo/vendor/clap-3.2.22:clap",
        "crossterm": "//cargo/vendor/crossterm-0.25.0:crossterm",
        "dirs": "//cargo/vendor/dirs-4.0.0:dirs",
        "futures": "//cargo/vendor/futures-0.3.24:futures",
        "futures-channel": "//cargo/vendor/futures-channel-0.3.25:futures_channel",
        "lofty": "//cargo/vendor/lofty-0.9.0:lofty",
        "md5": "//cargo/vendor/md5-0.7.0:md5",
        "owo-colors": "//cargo/vendor/owo-colors-3.5.0:owo_colors",
        "sea-orm": "//cargo/vendor/sea-orm-0.9.3:sea_orm",
        "sea-orm-migration": "//cargo/vendor/sea-orm-migration-0.9.3:sea_orm_migration",
        "serde_json": "//cargo/vendor/serde_json-1.0.89:serde_json",
        "spinners": "//cargo/vendor/spinners-4.1.0:spinners",
        "tabled": "//cargo/vendor/tabled-0.8.0:tabled",
        "tokio": "//cargo/vendor/tokio-1.22.0:tokio",
        "tui": "//cargo/vendor/tui-0.19.0:tui",
        "tungstenite": "//cargo/vendor/tungstenite-0.17.3:tungstenite",
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of proc_macro dependencies for the Rust targets of that package.
_PROC_MACRO_DEPENDENCIES = {
    "addons": {
    },
    "client": {
    },
    "server": {
    },
    "entity": {
    },
    "types": {
    },
    "playback": {
        "async-trait": "//cargo/vendor/async-trait-0.1.57:async_trait",
    },
    "tracklist": {
    },
    "scanner": {
    },
    "migration": {
    },
    "settings": {
    },
    "storage": {
    },
    "discovery": {
    },
    "graphql": {
    },
    "webui": {
        "serde_derive": "//cargo/vendor/serde_derive-1.0.148:serde_derive",
    },
    "webui/musicplayer/src-tauri": {
    },
    "": {
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of normal dev dependencies for the Rust targets of that package.
_DEV_DEPENDENCIES = {
    "addons": {
    },
    "client": {
        "envtestkit": "//cargo/vendor/envtestkit-1.1.2:envtestkit",
        "tonic-web": "//cargo/vendor/tonic-web-0.4.0:tonic_web",
    },
    "server": {
    },
    "entity": {
    },
    "types": {
    },
    "playback": {
    },
    "tracklist": {
    },
    "scanner": {
    },
    "migration": {
    },
    "settings": {
    },
    "storage": {
    },
    "discovery": {
    },
    "graphql": {
    },
    "webui": {
        "futures-util": "//cargo/vendor/futures-util-0.3.25:futures_util",
        "serde_json": "//cargo/vendor/serde_json-1.0.89:serde_json",
        "surf": "//cargo/vendor/surf-2.3.2:surf",
    },
    "webui/musicplayer/src-tauri": {
    },
    "": {
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of proc_macro dev dependencies for the Rust targets of that package.
_DEV_PROC_MACRO_DEPENDENCIES = {
    "addons": {
    },
    "client": {
    },
    "server": {
    },
    "entity": {
    },
    "types": {
    },
    "playback": {
    },
    "tracklist": {
    },
    "scanner": {
    },
    "migration": {
    },
    "settings": {
    },
    "storage": {
    },
    "discovery": {
    },
    "graphql": {
        "ctor": "//cargo/vendor/ctor-0.1.26:ctor",
    },
    "webui": {
    },
    "webui/musicplayer/src-tauri": {
    },
    "": {
    },
}

def crate_deps(deps, package_name = None):
    """EXPERIMENTAL -- MAY CHANGE AT ANY TIME: Finds the fully qualified label of the requested crates for the package where this macro is called.

    WARNING: This macro is part of an expeirmental API and is subject to change.

    Args:
        deps (list): The desired list of crate targets.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()`.
    Returns:
        list: A list of labels to cargo-raze generated targets (str)
    """

    if not package_name:
        package_name = native.package_name()

    # Join both sets of dependencies
    dependencies = _flatten_dependency_maps([
        _DEPENDENCIES,
        _PROC_MACRO_DEPENDENCIES,
        _DEV_DEPENDENCIES,
        _DEV_PROC_MACRO_DEPENDENCIES,
    ])

    if not deps:
        return []

    missing_crates = []
    crate_targets = []
    for crate_target in deps:
        if crate_target not in dependencies[package_name]:
            missing_crates.append(crate_target)
        else:
            crate_targets.append(dependencies[package_name][crate_target])

    if missing_crates:
        fail("Could not find crates `{}` among dependencies of `{}`. Available dependencies were `{}`".format(
            missing_crates,
            package_name,
            dependencies[package_name],
        ))

    return crate_targets

def all_crate_deps(normal = False, normal_dev = False, proc_macro = False, proc_macro_dev = False, package_name = None):
    """EXPERIMENTAL -- MAY CHANGE AT ANY TIME: Finds the fully qualified label of all requested direct crate dependencies \
    for the package where this macro is called.

    If no parameters are set, all normal dependencies are returned. Setting any one flag will
    otherwise impact the contents of the returned list.

    Args:
        normal (bool, optional): If True, normal dependencies are included in the
            output list. Defaults to False.
        normal_dev (bool, optional): If True, normla dev dependencies will be
            included in the output list. Defaults to False.
        proc_macro (bool, optional): If True, proc_macro dependencies are included
            in the output list. Defaults to False.
        proc_macro_dev (bool, optional): If True, dev proc_macro dependencies are
            included in the output list. Defaults to False.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()`.

    Returns:
        list: A list of labels to cargo-raze generated targets (str)
    """

    if not package_name:
        package_name = native.package_name()

    # Determine the relevant maps to use
    all_dependency_maps = []
    if normal:
        all_dependency_maps.append(_DEPENDENCIES)
    if normal_dev:
        all_dependency_maps.append(_DEV_DEPENDENCIES)
    if proc_macro:
        all_dependency_maps.append(_PROC_MACRO_DEPENDENCIES)
    if proc_macro_dev:
        all_dependency_maps.append(_DEV_PROC_MACRO_DEPENDENCIES)

    # Default to always using normal dependencies
    if not all_dependency_maps:
        all_dependency_maps.append(_DEPENDENCIES)

    dependencies = _flatten_dependency_maps(all_dependency_maps)

    if not dependencies:
        return []

    return dependencies[package_name].values()

def _flatten_dependency_maps(all_dependency_maps):
    """Flatten a list of dependency maps into one dictionary.

    Dependency maps have the following structure:

    ```python
    DEPENDENCIES_MAP = {
        # The first key in the map is a Bazel package
        # name of the workspace this file is defined in.
        "package_name": {

            # An alias to a crate target.     # The label of the crate target the
            # Aliases are only crate names.   # alias refers to.
            "alias":                          "@full//:label",
        }
    }
    ```

    Args:
        all_dependency_maps (list): A list of dicts as described above

    Returns:
        dict: A dictionary as described above
    """
    dependencies = {}

    for dep_map in all_dependency_maps:
        for pkg_name in dep_map:
            if pkg_name not in dependencies:
                # Add a non-frozen dict to the collection of dependencies
                dependencies.setdefault(pkg_name, dict(dep_map[pkg_name].items()))
                continue

            duplicate_crate_aliases = [key for key in dependencies[pkg_name] if key in dep_map[pkg_name]]
            if duplicate_crate_aliases:
                fail("There should be no duplicate crate aliases: {}".format(duplicate_crate_aliases))

            dependencies[pkg_name].update(dep_map[pkg_name])

    return dependencies
