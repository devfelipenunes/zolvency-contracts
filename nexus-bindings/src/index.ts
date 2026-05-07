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





export interface Scope {
  contract_allowlist: Option<Array<string>>;
  function_allowlist: Option<Array<string>>;
  renewal_period: Option<u64>;
  scope_commitment: Option<Buffer>;
  transfer_limit: Option<i128>;
  ttl: u64;
}

export type DataKey = {tag: "SoulLocks", values: readonly [u32]} | {tag: "SoulBlacklist", values: readonly [u32]} | {tag: "Admin", values: void} | {tag: "PendingAdmin", values: void} | {tag: "Signer", values: void} | {tag: "TokenCount", values: void} | {tag: "TokenId", values: readonly [u32]} | {tag: "TokenExists", values: readonly [string]} | {tag: "AxelarConfig", values: void} | {tag: "InteropConfig", values: void} | {tag: "FeeConfig", values: void} | {tag: "Treasury", values: void} | {tag: "WillContract", values: void} | {tag: "Mandate", values: readonly [u64]} | {tag: "MandateState", values: readonly [u64]} | {tag: "MandateChildren", values: readonly [u64]} | {tag: "GlobalEpoch", values: readonly [string]} | {tag: "VerificationCacheKey", values: readonly [u64, u64]} | {tag: "ConsumedNonce", values: readonly [string, u64, Buffer]} | {tag: "NextMandateId", values: void} | {tag: "AgentMandate", values: readonly [string]};


export interface Mandate {
  agent: string;
  delegation_policy: DelegationPolicy;
  depth: u32;
  id: u64;
  issued_at_epoch: u64;
  issuer: string;
  parent_mandate_id: Option<u64>;
  root_anchor: string;
  scope: Scope;
}

export type ScopeTag = {tag: "TransferLimit", values: void} | {tag: "ContractAllowlist", values: void} | {tag: "FunctionAllowlist", values: void} | {tag: "ScopeCommitment", values: void};

export type Ecosystem = {tag: "Evm", values: void} | {tag: "Cosmos", values: void} | {tag: "Solana", values: void};


export interface FeeConfig {
  amount: i128;
  token: string;
}


export interface AxelarConfig {
  gas_service: string;
  gas_token: string;
  gateway: string;
}

export const MandateError = {
  1: {message:"AlreadyInitialized"},
  2: {message:"NotAdmin"},
  3: {message:"NotPendingAdmin"},
  4: {message:"NotInitialized"},
  5: {message:"SoulBlocked"},
  6: {message:"Unauthorized"},
  7: {message:"MandateNotFound"},
  8: {message:"MandateRevoked"},
  9: {message:"MandateExpired"},
  10: {message:"EpochMismatch"},
  11: {message:"BudgetExceeded"},
  12: {message:"ContractNotAllowed"},
  13: {message:"FunctionNotAllowed"},
  14: {message:"DepthExceeded"},
  15: {message:"DelegationNotAllowed"},
  16: {message:"ScopeViolation"},
  17: {message:"BudgetFractionViolated"},
  18: {message:"NonceAlreadyConsumed"},
  19: {message:"InvalidSep45Signature"},
  20: {message:"MandateAlreadyExists"},
  21: {message:"SoulIDRequired"}
}


export interface MandateState {
  allocated_to_children: i128;
  current_period_start: u64;
  is_revoked: boolean;
  mandate_id: u64;
  spent_budget: i128;
}


export interface ActionContext {
  function_name: string;
  target_contract: string;
}


export interface InteropConfig {
  active_protocol: InteropProtocol;
  adapter_address: string;
}


export interface TokenMetadata {
  data_source: string;
  name: string;
  symbol: string;
  version: string;
}


export interface MandateRequest {
  agent: string;
  delegation_policy: DelegationPolicy;
  epoch: u64;
  nonce: Buffer;
  root_anchor: string;
  scope: Scope;
  sep45_signature: Buffer;
}


export interface DelegationRules {
  allowed_scope_tags: Option<Array<ScopeTag>>;
  budget_fraction: Option<u32>;
  max_subdepth: u32;
}

export type InteropProtocol = {tag: "None", values: void} | {tag: "Axelar", values: void} | {tag: "Wormhole", values: void};


export interface CrossChainParams {
  destination_address: string;
  destination_chain: string;
  ecosystem: Ecosystem;
  user_destination_address: Buffer;
}

export type DelegationPolicy = {tag: "None", values: void} | {tag: "Full", values: void} | {tag: "Restricted", values: readonly [DelegationRules]};


export interface VerificationCache {
  cached_at_ledger: u32;
  epoch_at_cache: u64;
  is_valid: boolean;
  mandate_id: u64;
}

export interface Client {
  /**
   * Construct and simulate a get_epoch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_epoch: ({root_anchor}: {root_anchor: string}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_signer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_signer: (options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_zenith transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_zenith: ({soul_id}: {soul_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Map<string, u64>>>

  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, signer}: {admin: string, signer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_treasury: ({admin, treasury}: {admin: string, treasury: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a issue_mandate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_mandate: ({issuer, agent, scope, delegation_policy, parent_mandate_id}: {issuer: string, agent: string, scope: Scope, delegation_policy: DelegationPolicy, parent_mandate_id: Option<u64>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a update_signer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_signer: ({admin, new_signer}: {admin: string, new_signer: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a is_soul_locked transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_soul_locked: ({soul_id}: {soul_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a register_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  register_token: ({admin, token_contract}: {admin: string, token_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a revoke_mandate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revoke_mandate: ({caller, mandate_id}: {caller: string, mandate_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_fee_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_fee_config: ({admin, config}: {admin: string, config: FeeConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a transfer_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  transfer_admin: ({admin, new_admin}: {admin: string, new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a increment_epoch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  increment_epoch: ({root_anchor}: {root_anchor: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a verify_authority transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  verify_authority: ({mandate_id, agent, contract, function, transfer_amount}: {mandate_id: u64, agent: string, contract: string, function: string, transfer_amount: Option<i128>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<boolean>>>

  /**
   * Construct and simulate a export_reputation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  export_reputation: ({caller, soul_id, token_address, external_id, tier, nonce, cross_chain}: {caller: string, soul_id: u32, token_address: string, external_id: string, tier: u32, nonce: u64, cross_chain: Option<CrossChainParams>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_axelar_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_axelar_config: ({admin, config}: {admin: string, config: AxelarConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_will_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_will_contract: ({admin, will_contract}: {admin: string, will_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_token_metadata transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_token_metadata: ({token_contract}: {token_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<TokenMetadata>>

  /**
   * Construct and simulate a set_interop_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_interop_config: ({admin, config}: {admin: string, config: InteropConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_soul_slashing transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_soul_slashing: ({admin, soul_id}: {admin: string, soul_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_soul_reputation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_soul_reputation: ({soul_id, tokens}: {soul_id: u32, tokens: Option<Array<string>>}, options?: MethodOptions) => Promise<AssembledTransaction<Map<string, u64>>>

  /**
   * Construct and simulate a is_soul_blacklisted transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_soul_blacklisted: ({soul_id}: {soul_id: u32}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a issue_mandate_remote transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_mandate_remote: ({request}: {request: MandateRequest}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u64>>>

  /**
   * Construct and simulate a lock_soul_reputation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  lock_soul_reputation: ({admin, soul_id, unlock_timestamp}: {admin: string, soul_id: u32, unlock_timestamp: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a export_will_authority transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  export_will_authority: ({caller, agent, cc}: {caller: string, agent: string, cc: CrossChainParams}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

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
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAABVNjb3BlAAAAAAAABgAAAAAAAAASY29udHJhY3RfYWxsb3dsaXN0AAAAAAPoAAAD6gAAABMAAAAAAAAAEmZ1bmN0aW9uX2FsbG93bGlzdAAAAAAD6AAAA+oAAAARAAAAAAAAAA5yZW5ld2FsX3BlcmlvZAAAAAAD6AAAAAYAAAAAAAAAEHNjb3BlX2NvbW1pdG1lbnQAAAPoAAAD7gAAACAAAAAAAAAADnRyYW5zZmVyX2xpbWl0AAAAAAPoAAAACwAAAAAAAAADdHRsAAAAAAY=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAFQAAAAEAAAAAAAAACVNvdWxMb2NrcwAAAAAAAAEAAAAEAAAAAQAAAAAAAAANU291bEJsYWNrbGlzdAAAAAAAAAEAAAAEAAAAAAAAAAAAAAAFQWRtaW4AAAAAAAAAAAAAAAAAAAxQZW5kaW5nQWRtaW4AAAAAAAAAAAAAAAZTaWduZXIAAAAAAAAAAAAAAAAAClRva2VuQ291bnQAAAAAAAEAAAAAAAAAB1Rva2VuSWQAAAAAAQAAAAQAAAABAAAAAAAAAAtUb2tlbkV4aXN0cwAAAAABAAAAEwAAAAAAAAAAAAAADEF4ZWxhckNvbmZpZwAAAAAAAAAAAAAADUludGVyb3BDb25maWcAAAAAAAAAAAAAAAAAAAlGZWVDb25maWcAAAAAAAAAAAAAAAAAAAhUcmVhc3VyeQAAAAAAAAAAAAAADFdpbGxDb250cmFjdAAAAAEAAAAAAAAAB01hbmRhdGUAAAAAAQAAAAYAAAABAAAAAAAAAAxNYW5kYXRlU3RhdGUAAAABAAAABgAAAAEAAAAAAAAAD01hbmRhdGVDaGlsZHJlbgAAAAABAAAABgAAAAEAAAAAAAAAC0dsb2JhbEVwb2NoAAAAAAEAAAATAAAAAQAAAAAAAAAUVmVyaWZpY2F0aW9uQ2FjaGVLZXkAAAACAAAABgAAAAYAAAABAAAAAAAAAA1Db25zdW1lZE5vbmNlAAAAAAAAAwAAABMAAAAGAAAD7gAAACAAAAAAAAAAAAAAAA1OZXh0TWFuZGF0ZUlkAAAAAAAAAQAAAAAAAAAMQWdlbnRNYW5kYXRlAAAAAQAAABM=",
        "AAAAAQAAAAAAAAAAAAAAB01hbmRhdGUAAAAACQAAAAAAAAAFYWdlbnQAAAAAAAATAAAAAAAAABFkZWxlZ2F0aW9uX3BvbGljeQAAAAAAB9AAAAAQRGVsZWdhdGlvblBvbGljeQAAAAAAAAAFZGVwdGgAAAAAAAAEAAAAAAAAAAJpZAAAAAAABgAAAAAAAAAPaXNzdWVkX2F0X2Vwb2NoAAAAAAYAAAAAAAAABmlzc3VlcgAAAAAAEwAAAAAAAAARcGFyZW50X21hbmRhdGVfaWQAAAAAAAPoAAAABgAAAAAAAAALcm9vdF9hbmNob3IAAAAAEwAAAAAAAAAFc2NvcGUAAAAAAAfQAAAABVNjb3BlAAAA",
        "AAAAAgAAAAAAAAAAAAAACFNjb3BlVGFnAAAABAAAAAAAAAAAAAAADVRyYW5zZmVyTGltaXQAAAAAAAAAAAAAAAAAABFDb250cmFjdEFsbG93bGlzdAAAAAAAAAAAAAAAAAAAEUZ1bmN0aW9uQWxsb3dsaXN0AAAAAAAAAAAAAAAAAAAPU2NvcGVDb21taXRtZW50AA==",
        "AAAAAgAAAAAAAAAAAAAACUVjb3N5c3RlbQAAAAAAAAMAAAAAAAAAAAAAAANFdm0AAAAAAAAAAAAAAAAGQ29zbW9zAAAAAAAAAAAAAAAAAAZTb2xhbmEAAA==",
        "AAAAAQAAAAAAAAAAAAAACUZlZUNvbmZpZwAAAAAAAAIAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAFdG9rZW4AAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAADEF4ZWxhckNvbmZpZwAAAAMAAAAAAAAAC2dhc19zZXJ2aWNlAAAAABMAAAAAAAAACWdhc190b2tlbgAAAAAAABMAAAAAAAAAB2dhdGV3YXkAAAAAEw==",
        "AAAABAAAAAAAAAAAAAAADE1hbmRhdGVFcnJvcgAAABUAAAAAAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAAQAAAAAAAAAITm90QWRtaW4AAAACAAAAAAAAAA9Ob3RQZW5kaW5nQWRtaW4AAAAAAwAAAAAAAAAOTm90SW5pdGlhbGl6ZWQAAAAAAAQAAAAAAAAAC1NvdWxCbG9ja2VkAAAAAAUAAAAAAAAADFVuYXV0aG9yaXplZAAAAAYAAAAAAAAAD01hbmRhdGVOb3RGb3VuZAAAAAAHAAAAAAAAAA5NYW5kYXRlUmV2b2tlZAAAAAAACAAAAAAAAAAOTWFuZGF0ZUV4cGlyZWQAAAAAAAkAAAAAAAAADUVwb2NoTWlzbWF0Y2gAAAAAAAAKAAAAAAAAAA5CdWRnZXRFeGNlZWRlZAAAAAAACwAAAAAAAAASQ29udHJhY3ROb3RBbGxvd2VkAAAAAAAMAAAAAAAAABJGdW5jdGlvbk5vdEFsbG93ZWQAAAAAAA0AAAAAAAAADURlcHRoRXhjZWVkZWQAAAAAAAAOAAAAAAAAABREZWxlZ2F0aW9uTm90QWxsb3dlZAAAAA8AAAAAAAAADlNjb3BlVmlvbGF0aW9uAAAAAAAQAAAAAAAAABZCdWRnZXRGcmFjdGlvblZpb2xhdGVkAAAAAAARAAAAAAAAABROb25jZUFscmVhZHlDb25zdW1lZAAAABIAAAAAAAAAFUludmFsaWRTZXA0NVNpZ25hdHVyZQAAAAAAABMAAAAAAAAAFE1hbmRhdGVBbHJlYWR5RXhpc3RzAAAAFAAAAAAAAAAOU291bElEUmVxdWlyZWQAAAAAABU=",
        "AAAAAQAAAAAAAAAAAAAADE1hbmRhdGVTdGF0ZQAAAAUAAAAAAAAAFWFsbG9jYXRlZF90b19jaGlsZHJlbgAAAAAAAAsAAAAAAAAAFGN1cnJlbnRfcGVyaW9kX3N0YXJ0AAAABgAAAAAAAAAKaXNfcmV2b2tlZAAAAAAAAQAAAAAAAAAKbWFuZGF0ZV9pZAAAAAAABgAAAAAAAAAMc3BlbnRfYnVkZ2V0AAAACw==",
        "AAAAAQAAAAAAAAAAAAAADUFjdGlvbkNvbnRleHQAAAAAAAACAAAAAAAAAA1mdW5jdGlvbl9uYW1lAAAAAAAAEAAAAAAAAAAPdGFyZ2V0X2NvbnRyYWN0AAAAABM=",
        "AAAAAQAAAAAAAAAAAAAADUludGVyb3BDb25maWcAAAAAAAACAAAAAAAAAA9hY3RpdmVfcHJvdG9jb2wAAAAH0AAAAA9JbnRlcm9wUHJvdG9jb2wAAAAAAAAAAA9hZGFwdGVyX2FkZHJlc3MAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAADVRva2VuTWV0YWRhdGEAAAAAAAAEAAAAAAAAAAtkYXRhX3NvdXJjZQAAAAAQAAAAAAAAAARuYW1lAAAAEAAAAAAAAAAGc3ltYm9sAAAAAAAQAAAAAAAAAAd2ZXJzaW9uAAAAABA=",
        "AAAAAQAAAAAAAAAAAAAADk1hbmRhdGVSZXF1ZXN0AAAAAAAHAAAAAAAAAAVhZ2VudAAAAAAAABMAAAAAAAAAEWRlbGVnYXRpb25fcG9saWN5AAAAAAAH0AAAABBEZWxlZ2F0aW9uUG9saWN5AAAAAAAAAAVlcG9jaAAAAAAAAAYAAAAAAAAABW5vbmNlAAAAAAAD7gAAACAAAAAAAAAAC3Jvb3RfYW5jaG9yAAAAABMAAAAAAAAABXNjb3BlAAAAAAAH0AAAAAVTY29wZQAAAAAAAAAAAAAPc2VwNDVfc2lnbmF0dXJlAAAAA+4AAABA",
        "AAAAAQAAAAAAAAAAAAAAD0RlbGVnYXRpb25SdWxlcwAAAAADAAAAAAAAABJhbGxvd2VkX3Njb3BlX3RhZ3MAAAAAA+gAAAPqAAAH0AAAAAhTY29wZVRhZwAAAAAAAAAPYnVkZ2V0X2ZyYWN0aW9uAAAAA+gAAAAEAAAAAAAAAAxtYXhfc3ViZGVwdGgAAAAE",
        "AAAAAgAAAAAAAAAAAAAAD0ludGVyb3BQcm90b2NvbAAAAAADAAAAAAAAAAAAAAAETm9uZQAAAAAAAAAAAAAABkF4ZWxhcgAAAAAAAAAAAAAAAAAIV29ybWhvbGU=",
        "AAAAAQAAAAAAAAAAAAAAEENyb3NzQ2hhaW5QYXJhbXMAAAAEAAAAAAAAABNkZXN0aW5hdGlvbl9hZGRyZXNzAAAAABAAAAAAAAAAEWRlc3RpbmF0aW9uX2NoYWluAAAAAAAAEAAAAAAAAAAJZWNvc3lzdGVtAAAAAAAH0AAAAAlFY29zeXN0ZW0AAAAAAAAAAAAAGHVzZXJfZGVzdGluYXRpb25fYWRkcmVzcwAAAA4=",
        "AAAAAgAAAAAAAAAAAAAAEERlbGVnYXRpb25Qb2xpY3kAAAADAAAAAAAAAAAAAAAETm9uZQAAAAAAAAAAAAAABEZ1bGwAAAABAAAAAAAAAApSZXN0cmljdGVkAAAAAAABAAAH0AAAAA9EZWxlZ2F0aW9uUnVsZXMA",
        "AAAAAQAAAAAAAAAAAAAAEVZlcmlmaWNhdGlvbkNhY2hlAAAAAAAABAAAAAAAAAAQY2FjaGVkX2F0X2xlZGdlcgAAAAQAAAAAAAAADmVwb2NoX2F0X2NhY2hlAAAAAAAGAAAAAAAAAAhpc192YWxpZAAAAAEAAAAAAAAACm1hbmRhdGVfaWQAAAAAAAY=",
        "AAAAAAAAAAAAAAAJZ2V0X2Vwb2NoAAAAAAAAAQAAAAAAAAALcm9vdF9hbmNob3IAAAAAEwAAAAEAAAAG",
        "AAAAAAAAAAAAAAAKZ2V0X3NpZ25lcgAAAAAAAAAAAAEAAAPpAAAAEwAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAKZ2V0X3plbml0aAAAAAAAAQAAAAAAAAAHc291bF9pZAAAAAAEAAAAAQAAA+wAAAARAAAABg==",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZzaWduZXIAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAMc2V0X3RyZWFzdXJ5AAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAh0cmVhc3VyeQAAABMAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAANaXNzdWVfbWFuZGF0ZQAAAAAAAAUAAAAAAAAABmlzc3VlcgAAAAAAEwAAAAAAAAAFYWdlbnQAAAAAAAATAAAAAAAAAAVzY29wZQAAAAAAB9AAAAAFU2NvcGUAAAAAAAAAAAAAEWRlbGVnYXRpb25fcG9saWN5AAAAAAAH0AAAABBEZWxlZ2F0aW9uUG9saWN5AAAAAAAAABFwYXJlbnRfbWFuZGF0ZV9pZAAAAAAAA+gAAAAGAAAAAQAAA+kAAAAGAAAH0AAAAAxNYW5kYXRlRXJyb3I=",
        "AAAAAAAAAAAAAAANdXBkYXRlX3NpZ25lcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKbmV3X3NpZ25lcgAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAOaXNfc291bF9sb2NrZWQAAAAAAAEAAAAAAAAAB3NvdWxfaWQAAAAABAAAAAEAAAAB",
        "AAAAAAAAAAAAAAAOcmVnaXN0ZXJfdG9rZW4AAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOdG9rZW5fY29udHJhY3QAAAAAABMAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAAOcmV2b2tlX21hbmRhdGUAAAAAAAIAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAKbWFuZGF0ZV9pZAAAAAAABgAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAOc2V0X2ZlZV9jb25maWcAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGY29uZmlnAAAAAAfQAAAACUZlZUNvbmZpZwAAAAAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAOdHJhbnNmZXJfYWRtaW4AAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAPaW5jcmVtZW50X2Vwb2NoAAAAAAEAAAAAAAAAC3Jvb3RfYW5jaG9yAAAAABMAAAABAAAD6QAAAAYAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAAQdmVyaWZ5X2F1dGhvcml0eQAAAAUAAAAAAAAACm1hbmRhdGVfaWQAAAAAAAYAAAAAAAAABWFnZW50AAAAAAAAEwAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAhmdW5jdGlvbgAAABEAAAAAAAAAD3RyYW5zZmVyX2Ftb3VudAAAAAPoAAAACwAAAAEAAAPpAAAAAQAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAARZXhwb3J0X3JlcHV0YXRpb24AAAAAAAAHAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAB3NvdWxfaWQAAAAABAAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAAC2V4dGVybmFsX2lkAAAAABAAAAAAAAAABHRpZXIAAAAEAAAAAAAAAAVub25jZQAAAAAAAAYAAAAAAAAAC2Nyb3NzX2NoYWluAAAAA+gAAAfQAAAAEENyb3NzQ2hhaW5QYXJhbXMAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAARc2V0X2F4ZWxhcl9jb25maWcAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABmNvbmZpZwAAAAAH0AAAAAxBeGVsYXJDb25maWcAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAARc2V0X3dpbGxfY29udHJhY3QAAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADXdpbGxfY29udHJhY3QAAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAAxNYW5kYXRlRXJyb3I=",
        "AAAAAAAAAAAAAAASZ2V0X3Rva2VuX21ldGFkYXRhAAAAAAABAAAAAAAAAA50b2tlbl9jb250cmFjdAAAAAAAEwAAAAEAAAfQAAAADVRva2VuTWV0YWRhdGEAAAA=",
        "AAAAAAAAAAAAAAASc2V0X2ludGVyb3BfY29uZmlnAAAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABmNvbmZpZwAAAAAH0AAAAA1JbnRlcm9wQ29uZmlnAAAAAAAAAQAAA+kAAAACAAAH0AAAAAxNYW5kYXRlRXJyb3I=",
        "AAAAAAAAAAAAAAATYXBwbHlfc291bF9zbGFzaGluZwAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB3NvdWxfaWQAAAAABAAAAAEAAAPpAAAAAgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAATZ2V0X3NvdWxfcmVwdXRhdGlvbgAAAAACAAAAAAAAAAdzb3VsX2lkAAAAAAQAAAAAAAAABnRva2VucwAAAAAD6AAAA+oAAAATAAAAAQAAA+wAAAARAAAABg==",
        "AAAAAAAAAAAAAAATaXNfc291bF9ibGFja2xpc3RlZAAAAAABAAAAAAAAAAdzb3VsX2lkAAAAAAQAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAUaXNzdWVfbWFuZGF0ZV9yZW1vdGUAAAABAAAAAAAAAAdyZXF1ZXN0AAAAB9AAAAAOTWFuZGF0ZVJlcXVlc3QAAAAAAAEAAAPpAAAABgAAB9AAAAAMTWFuZGF0ZUVycm9y",
        "AAAAAAAAAAAAAAAUbG9ja19zb3VsX3JlcHV0YXRpb24AAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAB3NvdWxfaWQAAAAABAAAAAAAAAAQdW5sb2NrX3RpbWVzdGFtcAAAAAYAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==",
        "AAAAAAAAAAAAAAAVZXhwb3J0X3dpbGxfYXV0aG9yaXR5AAAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAVhZ2VudAAAAAAAABMAAAAAAAAAAmNjAAAAAAfQAAAAEENyb3NzQ2hhaW5QYXJhbXMAAAABAAAD6QAAAAIAAAfQAAAADE1hbmRhdGVFcnJvcg==" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_epoch: this.txFromJSON<u64>,
        get_signer: this.txFromJSON<Result<string>>,
        get_zenith: this.txFromJSON<Map<string, u64>>,
        initialize: this.txFromJSON<Result<void>>,
        accept_admin: this.txFromJSON<Result<void>>,
        set_treasury: this.txFromJSON<Result<void>>,
        issue_mandate: this.txFromJSON<Result<u64>>,
        update_signer: this.txFromJSON<Result<void>>,
        is_soul_locked: this.txFromJSON<boolean>,
        register_token: this.txFromJSON<Result<void>>,
        revoke_mandate: this.txFromJSON<Result<void>>,
        set_fee_config: this.txFromJSON<Result<void>>,
        transfer_admin: this.txFromJSON<Result<void>>,
        increment_epoch: this.txFromJSON<Result<u64>>,
        verify_authority: this.txFromJSON<Result<boolean>>,
        export_reputation: this.txFromJSON<Result<void>>,
        set_axelar_config: this.txFromJSON<Result<void>>,
        set_will_contract: this.txFromJSON<Result<void>>,
        get_token_metadata: this.txFromJSON<TokenMetadata>,
        set_interop_config: this.txFromJSON<Result<void>>,
        apply_soul_slashing: this.txFromJSON<Result<void>>,
        get_soul_reputation: this.txFromJSON<Map<string, u64>>,
        is_soul_blacklisted: this.txFromJSON<boolean>,
        issue_mandate_remote: this.txFromJSON<Result<u64>>,
        lock_soul_reputation: this.txFromJSON<Result<void>>,
        export_will_authority: this.txFromJSON<Result<void>>
  }
}