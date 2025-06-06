#!/bin/bash

# Pass in network name, IC url, identity name, Governance canisterId, Ledger canisterId, CMC canisterId, and test mode (true/false)
# eg './deploy.sh ic https://ic0.app/ openchat rrkah-fqaaa-aaaaa-aaaaq-cai ryjl3-tyaaa-aaaaa-aaaba-cai rkp4c-7iaaa-aaaaa-aaaca-cai false'

NETWORK=$1
IC_URL=$2
IDENTITY=$3
WASM_SRC=$4 # WASM_SRC is either empty, "build", "latest", "local", "prod" the commit Id or the release version
NNS_ROOT_CANISTER_ID=$5
NNS_GOVERNANCE_CANISTER_ID=$6
NNS_INTERNET_IDENTITY_CANISTER_ID=$7
NNS_LEDGER_CANISTER_ID=$8
NNS_CMC_CANISTER_ID=$9
NNS_SNS_WASM_CANISTER_ID=${10}
NNS_INDEX_CANISTER_ID=${11}
TEST_MODE=${12}

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR/..

if [ $WASM_SRC = "build" ]
then
    ./scripts/generate-all-canister-wasms.sh
elif [ $WASM_SRC != "local" ]
then
    ./scripts/download-all-canister-wasms.sh $WASM_SRC || exit 1
fi

if [ ! -e ./wasms/event_store.wasm.gz ]
then
  ./scripts/download-canister-wasm-dfx.sh event_store || exit 1
  ./scripts/download-canister-wasm-dfx.sh sign_in_with_email || exit 1
  ./scripts/download-canister-wasm-dfx.sh sign_in_with_ethereum || exit 1
  ./scripts/download-canister-wasm-dfx.sh sign_in_with_solana || exit 1
fi

OPENCHAT_INSTALLER_CANISTER_ID=$(dfx canister --network $NETWORK id openchat_installer)
USER_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id user_index)
GROUP_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id group_index)
NOTIFICATIONS_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id notifications_index)
LOCAL_USER_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id local_user_index)
LOCAL_GROUP_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id local_group_index)
NOTIFICATIONS_CANISTER_ID=$(dfx canister --network $NETWORK id notifications)
IDENTITY_CANISTER_ID=$(dfx canister --network $NETWORK id identity)
ONLINE_USERS_CANISTER_ID=$(dfx canister --network $NETWORK id online_users)
PROPOSALS_BOT_CANISTER_ID=$(dfx canister --network $NETWORK id proposals_bot)
AIRDROP_BOT_CANISTER_ID=$(dfx canister --network $NETWORK id airdrop_bot)
STORAGE_INDEX_CANISTER_ID=$(dfx canister --network $NETWORK id storage_index)
CYCLES_DISPENSER_CANISTER_ID=$(dfx canister --network $NETWORK id cycles_dispenser)
REGISTRY_CANISTER_ID=$(dfx canister --network $NETWORK id registry)
MARKET_MAKER_CANISTER_ID=$(dfx canister --network $NETWORK id market_maker)
NEURON_CONTROLLER_CANISTER_ID=$(dfx canister --network $NETWORK id neuron_controller)
ESCROW_CANISTER_ID=$(dfx canister --network $NETWORK id escrow)
TRANSLATIONS_CANISTER_ID=$(dfx canister --network $NETWORK id translations)
EVENT_RELAY_CANISTER_ID=$(dfx canister --network $NETWORK id event_relay)
EVENT_STORE_CANISTER_ID=$(dfx canister --network $NETWORK id event_store)
SIGN_IN_WITH_EMAIL_CANISTER_ID=$(dfx canister --network $NETWORK id sign_in_with_email)
SIGN_IN_WITH_ETHEREUM_CANISTER_ID=$(dfx canister --network $NETWORK id sign_in_with_ethereum)
SIGN_IN_WITH_SOLANA_CANISTER_ID=$(dfx canister --network $NETWORK id sign_in_with_solana)
WEBSITE_CANISTER_ID=$(dfx canister --network $NETWORK id website)

echo "Building canister_installer"
cargo build --package canister_installer
echo "Building complete"

echo "Running canister_installer"
cargo run \
  --package canister_installer -- \
  --url $IC_URL \
  --test-mode $TEST_MODE \
  --controller $IDENTITY \
  --openchat-installer $OPENCHAT_INSTALLER_CANISTER_ID \
  --user-index $USER_INDEX_CANISTER_ID \
  --group-index $GROUP_INDEX_CANISTER_ID \
  --notifications-index $NOTIFICATIONS_INDEX_CANISTER_ID \
  --local-user-index $LOCAL_USER_INDEX_CANISTER_ID \
  --local-group-index $LOCAL_GROUP_INDEX_CANISTER_ID \
  --notifications $NOTIFICATIONS_CANISTER_ID \
  --identity $IDENTITY_CANISTER_ID \
  --online-users $ONLINE_USERS_CANISTER_ID \
  --proposals-bot $PROPOSALS_BOT_CANISTER_ID \
  --airdrop-bot $AIRDROP_BOT_CANISTER_ID \
  --storage-index $STORAGE_INDEX_CANISTER_ID \
  --cycles-dispenser $CYCLES_DISPENSER_CANISTER_ID \
  --registry $REGISTRY_CANISTER_ID \
  --market-maker $MARKET_MAKER_CANISTER_ID \
  --neuron-controller $NEURON_CONTROLLER_CANISTER_ID \
  --escrow $ESCROW_CANISTER_ID \
  --translations $TRANSLATIONS_CANISTER_ID \
  --event-relay $EVENT_RELAY_CANISTER_ID \
  --event-store $EVENT_STORE_CANISTER_ID \
  --sign-in-with-email $SIGN_IN_WITH_EMAIL_CANISTER_ID \
  --sign-in-with-ethereum $SIGN_IN_WITH_ETHEREUM_CANISTER_ID \
  --sign-in-with-solana $SIGN_IN_WITH_SOLANA_CANISTER_ID \
  --nns-root $NNS_ROOT_CANISTER_ID \
  --nns-governance $NNS_GOVERNANCE_CANISTER_ID \
  --nns-internet-identity $NNS_INTERNET_IDENTITY_CANISTER_ID \
  --nns-ledger $NNS_LEDGER_CANISTER_ID \
  --nns-cmc $NNS_CMC_CANISTER_ID \
  --nns-sns-wasm $NNS_SNS_WASM_CANISTER_ID \
  --nns-index $NNS_INDEX_CANISTER_ID \
  --website $WEBSITE_CANISTER_ID \

echo "Canisters installed"
