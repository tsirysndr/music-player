# Vendored SDK

Source: https://github.com/tsirysndr/dagger-sdk/tree/a86c5a8c90863411e626b6a67ddc8a70fd816710/purescript

This is the personal, unofficial PureScript SDK from commit `a86c5a8c90863411e626b6a67ddc8a70fd816710`.
The runtime, client, generator, and licenses are included. Development caches,
tests, examples, and the standalone schema snapshot are omitted. Module bindings
are regenerated from the beta.21 engine schema.

To update, copy the corresponding files from a reviewed SDK checkout, then run
`dagger generate -y` from `.dagger/ci` and commit the generated bindings.

Local packaging adjustment: retain `schema/LICENSE` in module sources and generated
clients, including after regeneration. The standalone schema JSON is omitted.
