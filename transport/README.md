# Media HTTP transport

HttpRequest follows HTTP 301/302/303/307/308 by default, including redirects to
another origin. Caller-supplied headers, including authentication tokens, keep
their values; Host is derived from the destination URL. Relative Location values
are resolved against the current URL and signed destination queries are preserved.

Set request.follow_redirects = false to receive the original redirect status
without requesting its destination. The enabled path follows up to ten hops,
within one header deadline and cancellation scope.

This is a transport API setting. The new transport has not yet been connected
to the daemon's server settings or the add-server UI.
