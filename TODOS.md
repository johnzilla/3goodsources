# TODOS

## Rate limiting on GET /registry

**What:** Add rate limiting to the GET /registry endpoint.

**Why:** In a federation context, a malicious actor could create many fake endorsement
entries pointing at your node, causing excessive /registry fetches. The self-endorsement
guard prevents self-loops by pubkey, but arbitrary URL loops are not prevented.

**Pros:** Hardens the server for production federation use. Prevents abuse.

**Cons:** Adds tower-governor or similar dependency. More configuration surface.

**Context:** Currently no rate limiting on any endpoint. For the federation test (v3.0),
the self-endorsement pubkey guard is sufficient. This becomes important if the network
grows beyond a handful of nodes.

**Depends on:** Federation feature (v3.0) shipping first. Not blocking.
