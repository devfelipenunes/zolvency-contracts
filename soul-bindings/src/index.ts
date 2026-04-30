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

export type DataKey = {tag: "Admin", values: void} | {tag: "Relayer", values: void} | {tag: "TotalSouls", values: void} | {tag: "SoulById", values: readonly [u32]} | {tag: "SoulByPasskey", values: readonly [Buffer]};


export interface SoulData {
  id: u32;
  minted_at: u64;
  passkey: Buffer;
  recovery_pubkey: Buffer;
}

export interface Client {
  /**
   * Construct and simulate a mint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  mint: ({relayer, passkey, recovery_pubkey}: {relayer: string, passkey: Buffer, recovery_pubkey: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a relayer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  relayer: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_soul transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul: ({id}: {id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SoulData>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, relayer}: {admin: string, relayer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a total_souls transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  total_souls: (options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a recover_soul transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  recover_soul: ({relayer, old_passkey, new_passkey, recovery_signature}: {relayer: string, old_passkey: Buffer, new_passkey: Buffer, recovery_signature: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_relayer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_relayer: ({admin, new_relayer}: {admin: string, new_relayer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_soul_by_passkey transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_by_passkey: ({passkey}: {passkey: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Option<SoulData>>>

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
      new ContractSpec([ "AAAAAAAAAAAAAAAEbWludAAAAAMAAAAAAAAAB3JlbGF5ZXIAAAAAEwAAAAAAAAAHcGFzc2tleQAAAAPuAAAAQQAAAAAAAAAPcmVjb3ZlcnlfcHVia2V5AAAAA+4AAABBAAAAAQAAA+kAAAAEAAAAAw==",
        "AAAAAAAAAAAAAAAFYWRtaW4AAAAAAAAAAAAAAQAAA+kAAAATAAAAAw==",
        "AAAAAAAAAAAAAAAHcmVsYXllcgAAAAAAAAAAAQAAA+kAAAATAAAAAw==",
        "AAAAAAAAAAAAAAAIZ2V0X3NvdWwAAAABAAAAAAAAAAJpZAAAAAAABAAAAAEAAAPoAAAH0AAAAAhTb3VsRGF0YQ==",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAdyZWxheWVyAAAAABMAAAABAAAD6QAAA+0AAAAAAAAAAw==",
        "AAAAAAAAAAAAAAALdG90YWxfc291bHMAAAAAAAAAAAEAAAPpAAAABAAAAAM=",
        "AAAAAAAAAAAAAAAMcmVjb3Zlcl9zb3VsAAAABAAAAAAAAAAHcmVsYXllcgAAAAATAAAAAAAAAAtvbGRfcGFzc2tleQAAAAPuAAAAQQAAAAAAAAALbmV3X3Bhc3NrZXkAAAAD7gAAAEEAAAAAAAAAEnJlY292ZXJ5X3NpZ25hdHVyZQAAAAAD7gAAAEAAAAABAAAD6QAAA+0AAAAAAAAAAw==",
        "AAAAAAAAAAAAAAAOdXBkYXRlX3JlbGF5ZXIAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAALbmV3X3JlbGF5ZXIAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAAD",
        "AAAAAAAAAAAAAAATZ2V0X3NvdWxfYnlfcGFzc2tleQAAAAABAAAAAAAAAAdwYXNza2V5AAAAA+4AAABBAAAAAQAAA+gAAAfQAAAACFNvdWxEYXRh",
        "AAAAAAAAAAAAAAAWZ2V0X3NvdWxfaWRfYnlfcGFzc2tleQAAAAAAAQAAAAAAAAAHcGFzc2tleQAAAAPuAAAAQQAAAAEAAAPoAAAABA==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAABwAAAAAAAAASQWxyZWFkeUluaXRpYWxpemVkAAAAAAABAAAAAAAAAA1Ob3RBdXRob3JpemVkAAAAAAAAAgAAAAAAAAARU291bEFscmVhZHlFeGlzdHMAAAAAAAADAAAAAAAAAA5Ob3RJbml0aWFsaXplZAAAAAAABAAAAAAAAAAPQ291bnRlck92ZXJmbG93AAAAAAUAAAAAAAAADFNvdWxOb3RGb3VuZAAAAAYAAAAAAAAAGEludmFsaWRSZWNvdmVyeVNpZ25hdHVyZQAAAAc=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAHUmVsYXllcgAAAAAAAAAAAAAAAApUb3RhbFNvdWxzAAAAAAABAAAAAAAAAAhTb3VsQnlJZAAAAAEAAAAEAAAAAQAAAAAAAAANU291bEJ5UGFzc2tleQAAAAAAAAEAAAPuAAAAQQ==",
        "AAAAAQAAAAAAAAAAAAAACFNvdWxEYXRhAAAABAAAAAAAAAACaWQAAAAAAAQAAAAAAAAACW1pbnRlZF9hdAAAAAAAAAYAAAAAAAAAB3Bhc3NrZXkAAAAD7gAAAEEAAAAAAAAAD3JlY292ZXJ5X3B1YmtleQAAAAPuAAAAQQ==" ]),
      options
    )
  }
  public readonly fromJSON = {
    mint: this.txFromJSON<Result<u32>>,
        admin: this.txFromJSON<Result<string>>,
        relayer: this.txFromJSON<Result<string>>,
        get_soul: this.txFromJSON<Option<SoulData>>,
        initialize: this.txFromJSON<Result<void>>,
        total_souls: this.txFromJSON<Result<u32>>,
        recover_soul: this.txFromJSON<Result<void>>,
        update_relayer: this.txFromJSON<Result<void>>,
        get_soul_by_passkey: this.txFromJSON<Option<SoulData>>,
        get_soul_id_by_passkey: this.txFromJSON<Option<u32>>
  }
}