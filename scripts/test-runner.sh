#!/bin/bash
# Auto-generated test runner from evolved FRACTRAN gateway
# Generation: 2 | Fitness: 90 | Calls: 180
set -euo pipefail
API="${1:-http://127.0.0.1:8443}"
TOKEN="Bearer test-runner"
OK=0; FAIL=0
check() {
  local code=$(curl -sf -o /dev/null -w '%{http_code}' --max-time 3 -H "Authorization: $TOKEN" "$@" 2>/dev/null || echo 000)
  if [ "$code" -ge 200 ] && [ "$code" -lt 300 ]; then OK=$((OK+1)); printf '  ✅ %s %s\n' "$code" "$2"; else FAIL=$((FAIL+1)); printf '  ❌ %s %s\n' "$code" "$2"; fi
}
echo '=== retro-sync test runner ==='
check -X POST -H 'Content-Type: application/json' -d '{"manifest":{"isrc":"EVO001"}}' "$API/api/manifest/mint"
check "$API/api/societies"
check -X POST -H 'Content-Type: application/json' -d '{"title":"Evolved","writers":[{"name":"FRACTRAN"}]}' "$API/api/register"
check -X POST -H 'Content-Type: application/json' -d '{"track":"evolved","format":"midi"}' "$API/api/upload"
check "$API/health"
check -X POST -H 'Content-Type: application/json' -d '{"release":{"title":"Evolved"}}' "$API/api/gateway/ern/push"
check -X POST -H 'Content-Type: application/json' -d '{"isrc":"EVO001","reason":"test"}' "$API/api/takedown"
check "$API/api/privacy/export/evo-user"
check -X POST -H 'Content-Type: application/json' -d '{"batch":[{"op":"upsert","key":"evo"}]}' "$API/api/durp/submit"
check "$API/api/vault/summary"
check -X POST -H 'Content-Type: application/json' -d '{"content_id":"evo"}' "$API/api/moderation/report"
check -X POST -H 'Content-Type: application/json' -d '{"title":"Evolved","territory":"US"}' "$API/api/gtms/classify"
check -X POST -H 'Content-Type: application/json' -d '{"data":"evolved"}' "$API/api/shard/decompose"
check -X POST -H 'Content-Type: application/json' -d '{"iswc":"T-000","territory":"CA"}' "$API/api/cmrra/licence"
check -X POST -H 'Content-Type: application/json' -d '{"programme":"Evolved"}' "$API/api/bbs/cue-sheet"
echo
echo "OK: $OK  FAIL: $FAIL  TOTAL: $((OK+FAIL))"
[ "$FAIL" -eq 0 ] && echo '∴ All tests passed. □' || echo '⚠ Some tests failed.'
