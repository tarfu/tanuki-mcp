#!/bin/bash
set -euo pipefail

echo "Creating PAT for e2e tests..." >&2
TOKEN=$(docker exec tanuki-mcp-e2e-gitlab gitlab-rails runner "
  user = User.find_by(username: 'root')
  user.personal_access_tokens.find_by(name: 'e2e-test')&.revoke!
  token = user.personal_access_tokens.create!(
    name: 'e2e-test',
    scopes: ['api', 'read_user', 'read_repository', 'write_repository'],
    expires_at: 1.day.from_now
  )
  puts token.token
" 2>/dev/null | tail -1)

if [ -z "$TOKEN" ]; then
  echo "ERROR: Failed to create PAT" >&2
  exit 1
fi
echo "PAT created successfully" >&2

echo "Testing API connectivity..." >&2
if curl -sf -H "PRIVATE-TOKEN: $TOKEN" "http://localhost:8080/api/v4/user" >/dev/null; then
  echo "API connection verified" >&2
else
  echo "ERROR: API connection failed" >&2
  exit 1
fi

# Output token to stdout (only non-error output)
echo "$TOKEN"
