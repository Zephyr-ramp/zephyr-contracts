#!/usr/bin/env bash
# Build, deploy and initialise the Zephyr escrow on Stellar testnet.
#
# Usage:  ./scripts/deploy-testnet.sh
#
# Environment (all optional):
#   ADMIN_IDENTITY       stellar-cli identity that deploys and administers   (default: zephyr-admin)
#   ANCHOR_IDENTITY      identity whose address becomes the anchor           (default: zephyr-anchor)
#   ANCHOR_ADDRESS       use this G... address as the anchor instead
#   USDC_ASSET           classic asset wrapped by the SAC                    (default: Circle testnet USDC)
#   MIN_TIMEOUT_LEDGERS  shortest escrow timeout                             (default: 720, ~1 hour)
#   MAX_TIMEOUT_LEDGERS  longest escrow timeout                              (default: 120960, ~7 days)
#
# Writes the result to deployments/testnet.json.
set -euo pipefail

NETWORK="testnet"
ADMIN_IDENTITY="${ADMIN_IDENTITY:-zephyr-admin}"
ANCHOR_IDENTITY="${ANCHOR_IDENTITY:-zephyr-anchor}"
USDC_ASSET="${USDC_ASSET:-USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5}"
MIN_TIMEOUT_LEDGERS="${MIN_TIMEOUT_LEDGERS:-720}"
MAX_TIMEOUT_LEDGERS="${MAX_TIMEOUT_LEDGERS:-120960}"
WASM="target/wasm32v1-none/release/zephyr_escrow.wasm"

cd "$(dirname "$0")/.."

command -v stellar >/dev/null || {
  echo "stellar CLI not found. Install: https://developers.stellar.org/docs/tools/cli" >&2
  exit 1
}

ensure_identity() {
  if ! stellar keys address "$1" >/dev/null 2>&1; then
    echo "==> Creating and funding identity '$1' with friendbot"
    stellar keys generate "$1" --network "$NETWORK" --fund
  fi
}

ensure_identity "$ADMIN_IDENTITY"
ADMIN_ADDRESS="$(stellar keys address "$ADMIN_IDENTITY")"

if [[ -z "${ANCHOR_ADDRESS:-}" ]]; then
  ensure_identity "$ANCHOR_IDENTITY"
  ANCHOR_ADDRESS="$(stellar keys address "$ANCHOR_IDENTITY")"
fi

if [[ "$ADMIN_ADDRESS" == "$ANCHOR_ADDRESS" ]]; then
  echo "Admin and anchor must be different accounts." >&2
  exit 1
fi

echo "==> Building (optimised)"
stellar contract build

echo "==> Resolving the USDC Stellar Asset Contract"
# Deploying the SAC fails harmlessly if it already exists (it does for Circle USDC).
stellar contract asset deploy --asset "$USDC_ASSET" --source "$ADMIN_IDENTITY" --network "$NETWORK" >/dev/null 2>&1 || true
TOKEN_ID="$(stellar contract id asset --asset "$USDC_ASSET" --network "$NETWORK")"

echo "==> Deploying escrow"
CONTRACT_ID="$(stellar contract deploy \
  --wasm "$WASM" \
  --source "$ADMIN_IDENTITY" \
  --network "$NETWORK" \
  --alias zephyr-escrow)"

echo "==> Initialising $CONTRACT_ID"
# `initialize` is a separate transaction from `deploy`, so someone could call it
# first. We check the stored config afterwards and abort if it isn't ours.
stellar contract invoke --id "$CONTRACT_ID" --source "$ADMIN_IDENTITY" --network "$NETWORK" -- \
  initialize \
  --admin "$ADMIN_ADDRESS" \
  --anchor "$ANCHOR_ADDRESS" \
  --token "$TOKEN_ID" \
  --min_timeout "$MIN_TIMEOUT_LEDGERS" \
  --max_timeout "$MAX_TIMEOUT_LEDGERS"

CONFIG="$(stellar contract invoke --id "$CONTRACT_ID" --source "$ADMIN_IDENTITY" --network "$NETWORK" --send=no -- get_config)"
if [[ "$CONFIG" != *"$ADMIN_ADDRESS"* || "$CONFIG" != *"$ANCHOR_ADDRESS"* ]]; then
  echo "Stored config does not match what we initialised. Do not use this contract; redeploy." >&2
  echo "$CONFIG" >&2
  exit 1
fi

WASM_HASH="$(sha256sum "$WASM" | cut -d' ' -f1)"
mkdir -p deployments
cat > deployments/testnet.json <<EOF
{
  "network": "$NETWORK",
  "contractId": "$CONTRACT_ID",
  "wasmHash": "$WASM_HASH",
  "admin": "$ADMIN_ADDRESS",
  "anchor": "$ANCHOR_ADDRESS",
  "token": "$TOKEN_ID",
  "usdcAsset": "$USDC_ASSET",
  "minTimeoutLedgers": $MIN_TIMEOUT_LEDGERS,
  "maxTimeoutLedgers": $MAX_TIMEOUT_LEDGERS,
  "deployedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo
echo "Escrow deployed: $CONTRACT_ID"
echo "Config:          $CONFIG"
echo "Saved to deployments/testnet.json. Next: ./scripts/bindings.sh"
