package retrosync.authz

import future.keywords.if

default allow := false

# Allow if valid SPIFFE SVID and role claim present
allow if {
    input.spiffe_id
    startswith(input.spiffe_id, "spiffe://retrosync.media/")
    input.claims.role in {"artist", "admin", "distributor"}
}
