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




export const Errors = {
  1: {message:"AlreadyInitialized"},
  2: {message:"NotAuthorized"},
  3: {message:"SoulAlreadyExists"},
  4: {message:"NotInitialized"},
  5: {message:"CounterOverflow"},
  6: {message:"SoulNotFound"},
  7: {message:"InvalidRecoverySignature"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "PendingAdmin", values: void} | {tag: "Relayer", values: void} | {tag: "TotalSouls", values: void} | {tag: "SoulById", values: readonly [u32]} | {tag: "SoulByPasskey", values: readonly [Buffer]} | {tag: "SoulByAddress", values: readonly [string]};


export interface SoulData {
  id: u32;
  minted_at: u64;
  owner: string;
  passkey: Buffer;
  recovery_pubkey: Buffer;
}

export type Ecosystem = {tag: "Evm", values: void} | {tag: "Cosmos", values: void} | {tag: "Solana", values: void};


export interface CrossChainParams {
  destination_address: string;
  destination_chain: string;
  ecosystem: Ecosystem;
  user_destination_address: Buffer;
}

export interface Client {
  /**
   * Construct and simulate a mint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  mint: ({relayer, owner, passkey, recovery_pubkey}: {relayer: string, owner: string, passkey: Buffer, recovery_pubkey: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a relayer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  relayer: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({admin, new_wasm_hash}: {admin: string, new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_soul transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul: ({id}: {id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SoulData>>>

  /**
   * Construct and simulate a has_soul transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  has_soul: ({owner}: {owner: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, relayer}: {admin: string, relayer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a total_souls transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  total_souls: (options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a recover_soul transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  recover_soul: ({relayer, old_passkey, new_passkey, recovery_signature}: {relayer: string, old_passkey: Buffer, new_passkey: Buffer, recovery_signature: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a transfer_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_admin: ({admin, new_admin}: {admin: string, new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_relayer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_relayer: ({admin, new_relayer}: {admin: string, new_relayer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_soul_by_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_by_address: ({owner}: {owner: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SoulData>>>

  /**
   * Construct and simulate a get_soul_by_passkey transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_by_passkey: ({passkey}: {passkey: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SoulData>>>

  /**
   * Construct and simulate a get_soul_id_by_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_id_by_address: ({owner}: {owner: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<u32>>>

  /**
   * Construct and simulate a get_soul_id_by_passkey transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_id_by_passkey: ({passkey}: {passkey: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<u32>>>

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
      new ContractSpec([ "AAAAAAAAAAAAAAAEbWludAAAAAQAAAAAAAAAB3JlbGF5ZXIAAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAdwYXNza2V5AAAAA+4AAABBAAAAAAAAAA9yZWNvdmVyeV9wdWJrZXkAAAAD7gAAAEEAAAABAAAD6QAAAAQAAAAD",
        "AAAAAAAAAAAAAAAFYWRtaW4AAAAAAAAAAAAAAQAAA+kAAAATAAAAAw==",
        "AAAAAAAAAAAAAAAHcmVsYXllcgAAAAAAAAAAAQAAA+kAAAATAAAAAw==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADW5ld193YXNtX2hhc2gAAAAAAAPuAAAAIAAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAIZ2V0X3NvdWwAAAABAAAAAAAAAAJpZAAAAAAABAAAAAEAAAPoAAAH0AAAAAhTb3VsRGF0YQ==",
        "AAAAAAAAAAAAAAAIaGFzX3NvdWwAAAABAAAAAAAAAAVvd25lcgAAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdyZWxheWVyAAAAABMAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAALdG90YWxfc291bHMAAAAAAAAAAAEAAAPpAAAABAAAAAM=",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAMcmVjb3Zlcl9zb3VsAAAABAAAAAAAAAAHcmVsYXllcgAAAAATAAAAAAAAAAtvbGRfcGFzc2tleQAAAAPuAAAAQQAAAAAAAAALbmV3X3Bhc3NrZXkAAAAD7gAAAEEAAAAAAAAAEnJlY292ZXJ5X3NpZ25hdHVyZQAAAAAD7gAAAEAAAAABAAAD6QAAAAIAAAAD",
        "AAAAAAAAAAAAAAAOdHJhbnNmZXJfYWRtaW4AAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAAOdXBkYXRlX3JlbGF5ZXIAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAALbmV3X3JlbGF5ZXIAAAAAEwAAAAEAAAPpAAAAAgAAAAM=",
        "AAAAAAAAAAAAAAATZ2V0X3NvdWxfYnlfYWRkcmVzcwAAAAABAAAAAAAAAAVvd25lcgAAAAAAABMAAAABAAAD6AAAB9AAAAAIU291bERhdGE=",
        "AAAAAAAAAAAAAAATZ2V0X3NvdWxfYnlfcGFzc2tleQAAAAABAAAAAAAAAAdwYXNza2V5AAAAA+4AAABBAAAAAQAAA+gAAAfQAAAACFNvdWxEYXRh",
        "AAAAAAAAAAAAAAAWZ2V0X3NvdWxfaWRfYnlfYWRkcmVzcwAAAAAAAQAAAAAAAAAFb3duZXIAAAAAAAATAAAAAQAAA+gAAAAE",
        "AAAAAAAAAAAAAAAWZ2V0X3NvdWxfaWRfYnlfcGFzc2tleQAAAAAAAQAAAAAAAAAHcGFzc2tleQAAAAPuAAAAQQAAAAEAAAPoAAAABA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAABwAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAAABAAAAAAAAAA1Ob3RBdXRob3JpemVkAAAAAAAAAgAAAAAAAAARU291bEFscmVhZHlFeGlzdHMAAAAAAAADAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAABAAAAAAAAAAPQ291bnRlck92ZXJmbG93AAAAAAUAAAAAAAAADFNvdWxOb3RGb3VuZAAAAAYAAAAAAAAAGEludmFsaWRSZWNvdmVyeVNpZ25hdHVyZQAAAAc=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABwAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAMUGVuZGluZ0FkbWluAAAAAAAAAAAAAAAHUmVsYXllcgAAAAAAAAAAAAAAAApUb3RhbFNvdWxzAAAAAAABAAAAAAAAAAhTb3VsQnlJZAAAAAEAAAAEAAAAAQAAAAAAAAANU291bEJ5UGFzc2tleQAAAAAAAAEAAAPuAAAAQQAAAAEAAAAAAAAADVNvdWxCeUFkZHJlc3MAAAAAAAABAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAACFNvdWxEYXRhAAAABQAAAAAAAAACaWQAAAAAAAQAAAAAAAAACW1pbnRlZF9hdAAAAAAAAAYAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAHcGFzc2tleQAAAAPuAAAAQQAAAAAAAAAPcmVjb3ZlcnlfcHVia2V5AAAAA+4AAABB",
        "AAAAAgAAAAAAAAAAAAAACUVjb3N5c3RlbQAAAAAAAAMAAAAAAAAAAAAAAANFdm0AAAAAAAAAAAAAAAAGQ29zbW9zAAAAAAAAAAAAAAAAAAZTb2xhbmEAAA==",
        "AAAAAQAAAAAAAAAAAAAAEENyb3NzQ2hhaW5QYXJhbXMAAAAEAAAAAAAAABNkZXN0aW5hdGlvbl9hZGRyZXNzAAAAABAAAAAAAAAAEWRlc3RpbmF0aW9uX2NoYWluAAAAAAAAEAAAAAAAAAAJZWNvc3lzdGVtAAAAAAAH0AAAAAlFY29zeXN0ZW0AAAAAAAAAAAAAGHVzZXJfZGVzdGluYXRpb25fYWRkcmVzcwAAAA4=" ]),
      options
    )
  }
  public readonly fromJSON = {
    mint: this.txFromJSON<Result<u32>>,
        admin: this.txFromJSON<Result<string>>,
        relayer: this.txFromJSON<Result<string>>,
        upgrade: this.txFromJSON<Result<void>>,
        get_soul: this.txFromJSON<Option<SoulData>>,
        has_soul: this.txFromJSON<boolean>,
        initialize: this.txFromJSON<Result<void>>,
        total_souls: this.txFromJSON<Result<u32>>,
        accept_admin: this.txFromJSON<Result<void>>,
        recover_soul: this.txFromJSON<Result<void>>,
        transfer_admin: this.txFromJSON<Result<void>>,
        update_relayer: this.txFromJSON<Result<void>>,
        get_soul_by_address: this.txFromJSON<Option<SoulData>>,
        get_soul_by_passkey: this.txFromJSON<Option<SoulData>>,
        get_soul_id_by_address: this.txFromJSON<Option<u32>>,
        get_soul_id_by_passkey: this.txFromJSON<Option<u32>>
  }
}