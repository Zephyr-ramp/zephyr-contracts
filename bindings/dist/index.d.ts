import { Buffer } from "buffer";
import { AssembledTransaction, Client as ContractClient, ClientOptions as ContractClientOptions, MethodOptions, Result } from "@stellar/stellar-sdk/contract";
import type { u32, i128 } from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";
export declare const networks: {
    readonly testnet: {
        readonly networkPassphrase: "Test SDF Network ; September 2015";
        readonly contractId: "CCQCVQTYB45FXJG6BPLR4RPBMBOE4VD73MUMTDWT6Q55TXINSEBV3IOQ";
    };
};
/**
 * Every failure the escrow can return. Codes are part of the public interface:
 * the backend and frontend map them to user-facing messages, so never renumber.
 */
export declare const Errors: {
    /**
     * `initialize` was already called.
     */
    1: {
        message: string;
    };
    /**
     * The contract has not been initialised yet.
     */
    2: {
        message: string;
    };
    /**
     * Amount must be strictly positive.
     */
    3: {
        message: string;
    };
    /**
     * Timeout is outside `[min_timeout_ledgers, max_timeout_ledgers]`, or the
     * timeout bounds themselves are invalid.
     */
    4: {
        message: string;
    };
    /**
     * An escrow with this `tx_id` already exists (in any status).
     */
    5: {
        message: string;
    };
    /**
     * No escrow exists for this `tx_id`.
     */
    6: {
        message: string;
    };
    /**
     * The escrow is no longer `Locked` (already claimed or refunded).
     */
    7: {
        message: string;
    };
    /**
     * The escrow reached `expires_ledger`; the anchor can no longer claim it.
     */
    8: {
        message: string;
    };
    /**
     * The escrow has not reached `expires_ledger`; the user cannot refund yet.
     */
    9: {
        message: string;
    };
    /**
     * New deposits are paused by the admin.
     */
    10: {
        message: string;
    };
};
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
export type EscrowStatus = {
    tag: "Locked";
    values: void;
} | {
    tag: "Claimed";
    values: void;
} | {
    tag: "Refunded";
    values: void;
};
export interface Client {
    /**
     * Construct and simulate a claim transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Anchor takes the funds after paying out fiat. Only while `Locked` and
     * strictly before `expires_ledger`.
     */
    claim: ({ tx_id }: {
        tx_id: Buffer;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a pause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Blocks new deposits. Claims, cancels and refunds keep working.
     */
    pause: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a cancel transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Anchor gives up on the withdrawal (e.g. the fiat payout failed) and
     * returns the funds to the user. Allowed any time while `Locked`.
     */
    cancel: ({ tx_id }: {
        tx_id: Buffer;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a refund transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Returns the funds to the user once the escrow has expired unclaimed.
     * Anyone may call it (the funds can only go to the original user), and it
     * works while the contract is paused.
     */
    refund: ({ tx_id }: {
        tx_id: Buffer;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Locks `amount` of the configured token from `user` for withdrawal `tx_id`.
     *
     * `tx_id` is `sha256(zephyr transaction uuid)`. The escrow expires
     * `timeout_ledgers` ledgers from now.
     */
    deposit: ({ user, tx_id, amount, timeout_ledgers }: {
        user: string;
        tx_id: Buffer;
        amount: i128;
        timeout_ledgers: u32;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a unpause transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     */
    unpause: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Replaces the contract code. Storage (config and escrows) is kept.
     */
    upgrade: ({ wasm_hash }: {
        wasm_hash: Buffer;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     */
    get_config: (options?: MethodOptions) => Promise<AssembledTransaction<Result<Config>>>;
    /**
     * Construct and simulate a get_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     */
    get_escrow: ({ tx_id }: {
        tx_id: Buffer;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<Escrow>>>;
    /**
     * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Stores the configuration. Can only be called once.
     *
     * `min_timeout`/`max_timeout` bound the `timeout_ledgers` a user may pick
     * in `deposit`. `token` is the USDC Stellar Asset Contract address.
     */
    initialize: ({ admin, anchor, token, min_timeout, max_timeout }: {
        admin: string;
        anchor: string;
        token: string;
        min_timeout: u32;
        max_timeout: u32;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a set_anchor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Changes the anchor. Existing `Locked` escrows become claimable/cancellable
     * by the new anchor only.
     */
    set_anchor: ({ anchor }: {
        anchor: string;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
    /**
     * Construct and simulate a set_timeouts transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
     * Changes the timeout bounds for new deposits. Existing escrows keep their
     * `expires_ledger`.
     */
    set_timeouts: ({ min_timeout, max_timeout }: {
        min_timeout: u32;
        max_timeout: u32;
    }, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>;
}
export declare class Client extends ContractClient {
    readonly options: ContractClientOptions;
    static deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions & Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
    }): Promise<AssembledTransaction<T>>;
    constructor(options: ContractClientOptions);
    readonly fromJSON: {
        claim: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        pause: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        cancel: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        refund: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        deposit: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        unpause: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        upgrade: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        get_config: (json: string) => AssembledTransaction<Result<Config, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        get_escrow: (json: string) => AssembledTransaction<Result<Escrow, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        initialize: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        set_anchor: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
        set_timeouts: (json: string) => AssembledTransaction<Result<void, import("@stellar/stellar-sdk/contract").ErrorMessage>>;
    };
}
