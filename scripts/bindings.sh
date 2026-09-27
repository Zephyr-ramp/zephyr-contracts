#!/usr/bin/env bash
# Generate the TypeScript client for the escrow into bindings/ and name the
# package @zephyr-ramp/escrow-client.
#
# Usage:  ./scripts/bindings.sh
#
# If deployments/testnet.json exists, the bindings are generated from that
# deployed contract and its ID is baked into the `networks.testnet` export.
# Otherwise (or with LOCAL_ONLY=1) they come from the local wasm, and callers
# pass `contractId` themselves.
set -euo pipefail

cd "$(dirname "$0")/.."

WASM="target/wasm32v1-none/release/zephyr_escrow.wasm"
OUT="bindings"
PACKAGE_NAME="@zephyr-ramp/escrow-client"

stellar contract build

ARGS=(--output-dir "$OUT" --overwrite)
if [[ -f deployments/testnet.json && -z "${LOCAL_ONLY:-}" ]]; then
  # Read the interface from the deployed contract so the bindings match what is
  # on chain. Set LOCAL_ONLY=1 to generate from the local build instead.
  CONTRACT_ID="$(sed -n 's/.*"contractId": *"\([^"]*\)".*/\1/p' deployments/testnet.json)"
  ARGS+=(--contract-id "$CONTRACT_ID" --network testnet)
else
  ARGS+=(--wasm "$WASM")
fi

stellar contract bindings typescript "${ARGS[@]}"

# Give the generated package its published name and metadata.
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
node - "$OUT/package.json" "$PACKAGE_NAME" "$VERSION" <<'EOF'
const fs = require("fs");
const [file, name, version] = process.argv.slice(2);
const pkg = JSON.parse(fs.readFileSync(file, "utf8"));
pkg.name = name;
pkg.version = version;
pkg.description = "TypeScript client for the Zephyr withdrawal escrow Soroban contract";
pkg.license = "Apache-2.0";
pkg.repository = { type: "git", url: "https://github.com/zephyr-ramp/zephyr-contracts.git", directory: "bindings" };
pkg.publishConfig = { access: "public" };
fs.writeFileSync(file, JSON.stringify(pkg, null, 2) + "\n");
EOF

(cd "$OUT" && npm install --no-audit --no-fund && npm run build)
echo "Bindings written to $OUT/ as $PACKAGE_NAME"
