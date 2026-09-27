import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CCQCVQTYB45FXJG6BPLR4RPBMBOE4VD73MUMTDWT6Q55TXINSEBV3IOQ",
  }
} as const

/**
 * Every failure the escrow can return. Codes are part of the public interface:
 * the backend and frontend map them to user-facing messages, so never renumber.
 */
export const Errors = {
  /**
   * `initialize` was already called.
   */
  1: {message:"AlreadyInitialized"},
  /**
   * The contract has not been initialised yet.
   */
  2: {message:"NotInitialized"},
  /**
   * Amount must be strictly positive.
   */
  3: {message:"InvalidAmount"},
  /**
   * Timeout is outside `[min_timeout_ledgers, max_timeout_ledgers]`, or the
   * timeout bounds themselves are invalid.
   */
  4: {message:"InvalidTimeout"},
  /**
   * An escrow with this `tx_id` already exists (in any status).
   */
  5: {message:"DuplicateTxId"},
  /**
   * No escrow exists for this `tx_id`.
   */
  6: {message:"NotFound"},
  /**
   * The escrow is no longer `Locked` (already claimed or refunded).
   */
  7: {message:"NotLocked"},
  /**
   * The escrow reached `expires_ledger`; the anchor can no longer claim it.
   */
  8: {message:"Expired"},
  /**
   * The escrow has not reached `expires_ledger`; the user cannot refund yet.
   */
  9: {message:"NotExpired"},
  /**
   * New deposits are paused by the admin.
   */
  10: {message:"Paused"}
}


/**
 * Contract configuration, kept in instance storage.
 */
export interface Config {
  /**
 * Can change the anchor, timeouts and pause state, and upgrade the contract.
 */
admin: string;
  /**
 * The only account allowed to `claim` or `cancel` escrows.
 */
anchor: string;
  /**
 * Longest timeout a user may choose for a deposit, in ledgers.
 */
max_timeout_ledgers: u32;
  /**
 * Shortest timeout a user may choose for a deposit, in ledgers.
 */
min_timeout_ledgers: u32;
  /**
 * When true, `deposit` is rejected. Nothing else is affected.
 */
paused: boolean;
  /**
 * The token held in escrow (the USDC Stellar Asset Contract).
 */
token: string;
}


/**
 * One withdrawal escrow, kept in persistent storage under its `tx_id`.
 */
export interface Escrow {
  /**
 * Amount in the token's smallest unit (stroops for USDC: 7 decimals).
 */
amount: i128;
  created_ledger: u32;
  /**
 * From this ledger on, the anchor can no longer claim and anyone can refund.
 */
expires_ledger: u32;
  status: EscrowStatus;
  user: string;
}

export type EscrowStatus = {tag: "Locked", values: void} | {tag: "Claimed", values: void} | {tag: "Refunded", values: void};









export interface Client {
  /**
   * Construct and simulate a claim transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Anchor takes the funds after paying out fiat. Only while `Locked` and
   * strictly before `expires_ledger`.
   */
  claim: ({tx_id}: {tx_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Blocks new deposits. Claims, cancels and refunds keep working.
   */
  pause: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Anchor gives up on the withdrawal (e.g. the fiat payout failed) and
   * returns the funds to the user. Allowed any time while `Locked`.
   */
  cancel: ({tx_id}: {tx_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the funds to the user once the escrow has expired unclaimed.
   * Anyone may call it (the funds can only go to the original user), and it
   * works while the contract is paused.
   */
  refund: ({tx_id}: {tx_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Locks `amount` of the configured token from `user` for withdrawal `tx_id`.
   * 
   * `tx_id` is `sha256(zephyr transaction uuid)`. The escrow expires
   * `timeout_ledgers` ledgers from now.
   */
  deposit: ({user, tx_id, amount, timeout_ledgers}: {user: string, tx_id: Buffer, amount: i128, timeout_ledgers: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unpause: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Replaces the contract code. Storage (config and escrows) is kept.
   */
  upgrade: ({wasm_hash}: {wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: (options?: MethodOptions) => Promise<AssembledTransaction<Result<Config>>>

  /**
   * Construct and simulate a get_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_escrow: ({tx_id}: {tx_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Escrow>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Stores the configuration. Can only be called once.
   * 
   * `min_timeout`/`max_timeout` bound the `timeout_ledgers` a user may pick
   * in `deposit`. `token` is the USDC Stellar Asset Contract address.
   */
  initialize: ({admin, anchor, token, min_timeout, max_timeout}: {admin: string, anchor: string, token: string, min_timeout: u32, max_timeout: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_anchor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Changes the anchor. Existing `Locked` escrows become claimable/cancellable
   * by the new anchor only.
   */
  set_anchor: ({anchor}: {anchor: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_timeouts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Changes the timeout bounds for new deposits. Existing escrows keep their
   * `expires_ledger`.
   */
  set_timeouts: ({min_timeout, max_timeout}: {min_timeout: u32, max_timeout: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAGdBbmNob3IgdGFrZXMgdGhlIGZ1bmRzIGFmdGVyIHBheWluZyBvdXQgZmlhdC4gT25seSB3aGlsZSBgTG9ja2VkYCBhbmQKc3RyaWN0bHkgYmVmb3JlIGBleHBpcmVzX2xlZGdlcmAuAAAAAAVjbGFpbQAAAAAAAAEAAAAAAAAABXR4X2lkAAAAAAAD7gAAACAAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAD5CbG9ja3MgbmV3IGRlcG9zaXRzLiBDbGFpbXMsIGNhbmNlbHMgYW5kIHJlZnVuZHMga2VlcCB3b3JraW5nLgAAAAAABXBhdXNlAAAAAAAAAAAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAINBbmNob3IgZ2l2ZXMgdXAgb24gdGhlIHdpdGhkcmF3YWwgKGUuZy4gdGhlIGZpYXQgcGF5b3V0IGZhaWxlZCkgYW5kCnJldHVybnMgdGhlIGZ1bmRzIHRvIHRoZSB1c2VyLiBBbGxvd2VkIGFueSB0aW1lIHdoaWxlIGBMb2NrZWRgLgAAAAAGY2FuY2VsAAAAAAABAAAAAAAAAAV0eF9pZAAAAAAAA+4AAAAgAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAALBSZXR1cm5zIHRoZSBmdW5kcyB0byB0aGUgdXNlciBvbmNlIHRoZSBlc2Nyb3cgaGFzIGV4cGlyZWQgdW5jbGFpbWVkLgpBbnlvbmUgbWF5IGNhbGwgaXQgKHRoZSBmdW5kcyBjYW4gb25seSBnbyB0byB0aGUgb3JpZ2luYWwgdXNlciksIGFuZCBpdAp3b3JrcyB3aGlsZSB0aGUgY29udHJhY3QgaXMgcGF1c2VkLgAAAAZyZWZ1bmQAAAAAAAEAAAAAAAAABXR4X2lkAAAAAAAD7gAAACAAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAALBMb2NrcyBgYW1vdW50YCBvZiB0aGUgY29uZmlndXJlZCB0b2tlbiBmcm9tIGB1c2VyYCBmb3Igd2l0aGRyYXdhbCBgdHhfaWRgLgoKYHR4X2lkYCBpcyBgc2hhMjU2KHplcGh5ciB0cmFuc2FjdGlvbiB1dWlkKWAuIFRoZSBlc2Nyb3cgZXhwaXJlcwpgdGltZW91dF9sZWRnZXJzYCBsZWRnZXJzIGZyb20gbm93LgAAAAdkZXBvc2l0AAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAV0eF9pZAAAAAAAA+4AAAAgAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAD3RpbWVvdXRfbGVkZ2VycwAAAAAEAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAHdW5wYXVzZQAAAAAAAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAEFSZXBsYWNlcyB0aGUgY29udHJhY3QgY29kZS4gU3RvcmFnZSAoY29uZmlnIGFuZCBlc2Nyb3dzKSBpcyBrZXB0LgAAAAAAAAd1cGdyYWRlAAAAAAEAAAAAAAAACXdhc21faGFzaAAAAAAAA+4AAAAgAAAAAQAAA+kAAAACAAAAAw==",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAAAAAAEAAAPpAAAH0AAAAAZDb25maWcAAAAAAAM=",
        "AAAAAAAAAAAAAAAKZ2V0X2VzY3JvdwAAAAAAAQAAAAAAAAAFdHhfaWQAAAAAAAPuAAAAIAAAAAEAAAPpAAAH0AAAAAZFc2Nyb3cAAAAAAAM=",
        "AAAAAAAAAL1TdG9yZXMgdGhlIGNvbmZpZ3VyYXRpb24uIENhbiBvbmx5IGJlIGNhbGxlZCBvbmNlLgoKYG1pbl90aW1lb3V0YC9gbWF4X3RpbWVvdXRgIGJvdW5kIHRoZSBgdGltZW91dF9sZWRnZXJzYCBhIHVzZXIgbWF5IHBpY2sKaW4gYGRlcG9zaXRgLiBgdG9rZW5gIGlzIHRoZSBVU0RDIFN0ZWxsYXIgQXNzZXQgQ29udHJhY3QgYWRkcmVzcy4AAAAAAAAKaW5pdGlhbGl6ZQAAAAAABQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZhbmNob3IAAAAAABMAAAAAAAAABXRva2VuAAAAAAAAEwAAAAAAAAALbWluX3RpbWVvdXQAAAAABAAAAAAAAAALbWF4X3RpbWVvdXQAAAAABAAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAGJDaGFuZ2VzIHRoZSBhbmNob3IuIEV4aXN0aW5nIGBMb2NrZWRgIGVzY3Jvd3MgYmVjb21lIGNsYWltYWJsZS9jYW5jZWxsYWJsZQpieSB0aGUgbmV3IGFuY2hvciBvbmx5LgAAAAAACnNldF9hbmNob3IAAAAAAAEAAAAAAAAABmFuY2hvcgAAAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAFpDaGFuZ2VzIHRoZSB0aW1lb3V0IGJvdW5kcyBmb3IgbmV3IGRlcG9zaXRzLiBFeGlzdGluZyBlc2Nyb3dzIGtlZXAgdGhlaXIKYGV4cGlyZXNfbGVkZ2VyYC4AAAAAAAxzZXRfdGltZW91dHMAAAACAAAAAAAAAAttaW5fdGltZW91dAAAAAAEAAAAAAAAAAttYXhfdGltZW91dAAAAAAEAAAAAQAAA+kAAAACAAAAAw==",
        "AAAABAAAAJpFdmVyeSBmYWlsdXJlIHRoZSBlc2Nyb3cgY2FuIHJldHVybi4gQ29kZXMgYXJlIHBhcnQgb2YgdGhlIHB1YmxpYyBpbnRlcmZhY2U6CnRoZSBiYWNrZW5kIGFuZCBmcm9udGVuZCBtYXAgdGhlbSB0byB1c2VyLWZhY2luZyBtZXNzYWdlcywgc28gbmV2ZXIgcmVudW1iZXIuAAAAAAAAAAAABUVycm9yAAAAAAAACgAAACBgaW5pdGlhbGl6ZWAgd2FzIGFscmVhZHkgY2FsbGVkLgAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAEAAAAqVGhlIGNvbnRyYWN0IGhhcyBub3QgYmVlbiBpbml0aWFsaXNlZCB5ZXQuAAAAAAAOTm90SW5pdGlhbGl6ZWQAAAAAAAIAAAAhQW1vdW50IG11c3QgYmUgc3RyaWN0bHkgcG9zaXRpdmUuAAAAAAAADUludmFsaWRBbW91bnQAAAAAAAADAAAAblRpbWVvdXQgaXMgb3V0c2lkZSBgW21pbl90aW1lb3V0X2xlZGdlcnMsIG1heF90aW1lb3V0X2xlZGdlcnNdYCwgb3IgdGhlCnRpbWVvdXQgYm91bmRzIHRoZW1zZWx2ZXMgYXJlIGludmFsaWQuAAAAAAAOSW52YWxpZFRpbWVvdXQAAAAAAAQAAAA7QW4gZXNjcm93IHdpdGggdGhpcyBgdHhfaWRgIGFscmVhZHkgZXhpc3RzIChpbiBhbnkgc3RhdHVzKS4AAAAADUR1cGxpY2F0ZVR4SWQAAAAAAAAFAAAAIk5vIGVzY3JvdyBleGlzdHMgZm9yIHRoaXMgYHR4X2lkYC4AAAAAAAhOb3RGb3VuZAAAAAYAAAA/VGhlIGVzY3JvdyBpcyBubyBsb25nZXIgYExvY2tlZGAgKGFscmVhZHkgY2xhaW1lZCBvciByZWZ1bmRlZCkuAAAAAAlOb3RMb2NrZWQAAAAAAAAHAAAAR1RoZSBlc2Nyb3cgcmVhY2hlZCBgZXhwaXJlc19sZWRnZXJgOyB0aGUgYW5jaG9yIGNhbiBubyBsb25nZXIgY2xhaW0gaXQuAAAAAAdFeHBpcmVkAAAAAAgAAABIVGhlIGVzY3JvdyBoYXMgbm90IHJlYWNoZWQgYGV4cGlyZXNfbGVkZ2VyYDsgdGhlIHVzZXIgY2Fubm90IHJlZnVuZCB5ZXQuAAAACk5vdEV4cGlyZWQAAAAAAAkAAAAlTmV3IGRlcG9zaXRzIGFyZSBwYXVzZWQgYnkgdGhlIGFkbWluLgAAAAAAAAZQYXVzZWQAAAAAAAo=",
        "AAAAAQAAADFDb250cmFjdCBjb25maWd1cmF0aW9uLCBrZXB0IGluIGluc3RhbmNlIHN0b3JhZ2UuAAAAAAAAAAAAAAZDb25maWcAAAAAAAYAAABKQ2FuIGNoYW5nZSB0aGUgYW5jaG9yLCB0aW1lb3V0cyBhbmQgcGF1c2Ugc3RhdGUsIGFuZCB1cGdyYWRlIHRoZSBjb250cmFjdC4AAAAAAAVhZG1pbgAAAAAAABMAAAA4VGhlIG9ubHkgYWNjb3VudCBhbGxvd2VkIHRvIGBjbGFpbWAgb3IgYGNhbmNlbGAgZXNjcm93cy4AAAAGYW5jaG9yAAAAAAATAAAAPExvbmdlc3QgdGltZW91dCBhIHVzZXIgbWF5IGNob29zZSBmb3IgYSBkZXBvc2l0LCBpbiBsZWRnZXJzLgAAABNtYXhfdGltZW91dF9sZWRnZXJzAAAAAAQAAAA9U2hvcnRlc3QgdGltZW91dCBhIHVzZXIgbWF5IGNob29zZSBmb3IgYSBkZXBvc2l0LCBpbiBsZWRnZXJzLgAAAAAAABNtaW5fdGltZW91dF9sZWRnZXJzAAAAAAQAAAA7V2hlbiB0cnVlLCBgZGVwb3NpdGAgaXMgcmVqZWN0ZWQuIE5vdGhpbmcgZWxzZSBpcyBhZmZlY3RlZC4AAAAABnBhdXNlZAAAAAAAAQAAADtUaGUgdG9rZW4gaGVsZCBpbiBlc2Nyb3cgKHRoZSBVU0RDIFN0ZWxsYXIgQXNzZXQgQ29udHJhY3QpLgAAAAAFdG9rZW4AAAAAAAAT",
        "AAAAAQAAAERPbmUgd2l0aGRyYXdhbCBlc2Nyb3csIGtlcHQgaW4gcGVyc2lzdGVudCBzdG9yYWdlIHVuZGVyIGl0cyBgdHhfaWRgLgAAAAAAAAAGRXNjcm93AAAAAAAFAAAAQ0Ftb3VudCBpbiB0aGUgdG9rZW4ncyBzbWFsbGVzdCB1bml0IChzdHJvb3BzIGZvciBVU0RDOiA3IGRlY2ltYWxzKS4AAAAABmFtb3VudAAAAAAACwAAAAAAAAAOY3JlYXRlZF9sZWRnZXIAAAAAAAQAAABKRnJvbSB0aGlzIGxlZGdlciBvbiwgdGhlIGFuY2hvciBjYW4gbm8gbG9uZ2VyIGNsYWltIGFuZCBhbnlvbmUgY2FuIHJlZnVuZC4AAAAAAA5leHBpcmVzX2xlZGdlcgAAAAAABAAAAAAAAAAGc3RhdHVzAAAAAAfQAAAADEVzY3Jvd1N0YXR1cwAAAAAAAAAEdXNlcgAAABM=",
        "AAAAAgAAAAAAAAAAAAAADEVzY3Jvd1N0YXR1cwAAAAMAAAAAAAAAH0Z1bmRzIGFyZSBoZWxkIGJ5IHRoZSBjb250cmFjdC4AAAAABkxvY2tlZAAAAAAAAAAAACxUaGUgYW5jaG9yIHBhaWQgb3V0IGZpYXQgYW5kIHRvb2sgdGhlIGZ1bmRzLgAAAAdDbGFpbWVkAAAAAAAAAAA/RnVuZHMgd2VudCBiYWNrIHRvIHRoZSB1c2VyIChhbmNob3IgYGNhbmNlbGAgb3IgdXNlciBgcmVmdW5kYCkuAAAAAAhSZWZ1bmRlZA==",
        "AAAABQAAADFBIHVzZXIgbG9ja2VkIGZ1bmRzLiBUb3BpY3M6IGBbImxvY2tlZCIsIHR4X2lkXWAuAAAAAAAAAAAAAAZMb2NrZWQAAAAAAAEAAAAGbG9ja2VkAAAAAAAEAAAAAAAAAAV0eF9pZAAAAAAAA+4AAAAgAAAAAQAAAAAAAAAEdXNlcgAAABMAAAAAAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAAAAAA5leHBpcmVzX2xlZGdlcgAAAAAABAAAAAAAAAAC",
        "AAAABQAAAFFUaGUgYW5jaG9yIGNsYWltZWQgdGhlIGZ1bmRzIGFmdGVyIHBheWluZyBvdXQgZmlhdC4gVG9waWNzOiBgWyJjbGFpbWVkIiwgdHhfaWRdYC4AAAAAAAAAAAAAB0NsYWltZWQAAAAAAQAAAAdjbGFpbWVkAAAAAAMAAAAAAAAABXR4X2lkAAAAAAAD7gAAACAAAAABAAAAAAAAAAZhbmNob3IAAAAAABMAAAAAAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAHFGdW5kcyB3ZW50IGJhY2sgdG8gdGhlIHVzZXIuIGBieV9hbmNob3JgIGlzIHRydWUgZm9yIGBjYW5jZWxgLCBmYWxzZSBmb3IKYHJlZnVuZGAuIFRvcGljczogYFsicmVmdW5kZWQiLCB0eF9pZF1gLgAAAAAAAAAAAAAIUmVmdW5kZWQAAAABAAAACHJlZnVuZGVkAAAABAAAAAAAAAAFdHhfaWQAAAAAAAPuAAAAIAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAAAAAAJYnlfYW5jaG9yAAAAAAAAAQAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAACFVwZ3JhZGVkAAAAAQAAAAh1cGdyYWRlZAAAAAEAAAAAAAAACXdhc21faGFzaAAAAAAAA+4AAAAgAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAC0luaXRpYWxpemVkAAAAAAEAAAALaW5pdGlhbGl6ZWQAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAAAAAAGYW5jaG9yAAAAAAATAAAAAAAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADUFuY2hvclVwZGF0ZWQAAAAAAAABAAAADmFuY2hvcl91cGRhdGVkAAAAAAABAAAAAAAAAAZhbmNob3IAAAAAABMAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADVBhdXNlZENoYW5nZWQAAAAAAAABAAAADnBhdXNlZF9jaGFuZ2VkAAAAAAABAAAAAAAAAAZwYXVzZWQAAAAAAAEAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAD1RpbWVvdXRzVXBkYXRlZAAAAAABAAAAEHRpbWVvdXRzX3VwZGF0ZWQAAAACAAAAAAAAABNtaW5fdGltZW91dF9sZWRnZXJzAAAAAAQAAAAAAAAAAAAAABNtYXhfdGltZW91dF9sZWRnZXJzAAAAAAQAAAAAAAAAAg==" ]),
      options
    )
  }
  public readonly fromJSON = {
    claim: this.txFromJSON<Result<void>>,
        pause: this.txFromJSON<Result<void>>,
        cancel: this.txFromJSON<Result<void>>,
        refund: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        unpause: this.txFromJSON<Result<void>>,
        upgrade: this.txFromJSON<Result<void>>,
        get_config: this.txFromJSON<Result<Config>>,
        get_escrow: this.txFromJSON<Result<Escrow>>,
        initialize: this.txFromJSON<Result<void>>,
        set_anchor: this.txFromJSON<Result<void>>,
        set_timeouts: this.txFromJSON<Result<void>>
  }
}