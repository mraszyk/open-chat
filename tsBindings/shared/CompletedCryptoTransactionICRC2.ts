// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CryptoAccountICRC1 } from "./CryptoAccountICRC1";
import type { Cryptocurrency } from "./Cryptocurrency";
import type { TSBytes } from "./TSBytes";
import type { TSPrincipal } from "./TSPrincipal";
import type { UserId } from "./UserId";

export type CompletedCryptoTransactionICRC2 = { ledger: TSPrincipal, token: Cryptocurrency, amount: bigint, spender: UserId, from: CryptoAccountICRC1, to: CryptoAccountICRC1, fee: bigint, memo?: TSBytes, created: bigint, block_index: bigint, };
