use crate::{RuntimeState, read_state};
use canister_api_macros::query;
use ic_canister_sig_creation::signature_map::CanisterSigInputs;
use ic_canister_sig_creation::{DELEGATION_SIG_DOMAIN, delegation_signature_msg};
use identity_canister::get_delegation::{Response::*, *};
use types::{Delegation, SignedDelegation};

#[query(msgpack = true, candid = true)]
fn get_delegation(args: Args) -> Response {
    read_state(|state| get_delegation_impl(args, state))
}

fn get_delegation_impl(args: Args, state: &RuntimeState) -> Response {
    let caller = state.env.caller();
    let auth_principal = state.data.user_principals.unwrap_temp_key_or(caller);

    let Some(user) = state.data.user_principals.get_by_auth_principal(&auth_principal) else {
        panic!("Caller not recognised");
    };

    let seed = state.data.calculate_seed(user.index);

    if let Ok(signature) = state.data.signature_map.get_signature_as_cbor(
        &CanisterSigInputs {
            domain: DELEGATION_SIG_DOMAIN,
            seed: &seed,
            message: &delegation_signature_msg(&args.session_key, args.expiration, None),
        },
        None,
    ) {
        let delegation = Delegation {
            pubkey: args.session_key,
            expiration: args.expiration,
        };

        Success(SignedDelegation { delegation, signature })
    } else {
        NotFound
    }
}
