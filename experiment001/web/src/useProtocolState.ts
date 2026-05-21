import { BorshAccountsCoder } from "@coral-xyz/anchor";
import type { Idl } from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  unpackAccount,
  unpackMint,
} from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import { useEffect, useState } from "react";

import idl from "./agc/agc_solana.json";
import { solanaAddresses, solanaRpcUrl } from "./contracts";

// Subset of the on-chain ProtocolState we surface in the dashboard.
export interface ProtocolStateView {
  regime: "neutral" | "expansion" | "defense" | "recovery";
  anchorPriceX18: bigint;
  lastSettledEpoch: bigint;
  lastReserveCoverageBps: bigint;
  lastStableCashCoverageBps: bigint;
  lastPremiumBps: bigint;
  lastLockedShareBps: bigint;
  lastExitPressureBps: bigint;
  lastVolatilityBps: bigint;
  creditPrincipalOutstandingAgc: bigint;
  creditDrawnAgc: bigint;
  creditRepaidAgc: bigint;
  creditInterestPaidAgc: bigint;
  creditDefaultedAgc: bigint;
}

export interface TokenBalances {
  treasuryAgc: bigint;
  treasuryUsdc: bigint;
  xagcVaultAgc: bigint;
  underwriterVaultAgc: bigint;
  agcSupply: bigint;
  xagcSupply: bigint;
}

export interface UserBalances {
  agc: bigint;
  xagc: bigint;
  btc: bigint;
}

export interface ProtocolSnapshot {
  state: ProtocolStateView | null;
  balances: TokenBalances | null;
  userBalances: UserBalances | null;
  error: string | null;
  lastFetchedAt: number | null;
}

function regimeFromVariant(variant: Record<string, unknown>): ProtocolStateView["regime"] {
  // Anchor's IDL decoder uses the Rust enum's PascalCase variant names.
  if (variant.Expansion || variant.expansion) return "expansion";
  if (variant.Defense || variant.defense) return "defense";
  if (variant.Recovery || variant.recovery) return "recovery";
  return "neutral";
}

function toBigInt(value: unknown): bigint {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") return BigInt(value);
  if (typeof value === "string") return BigInt(value);
  if (value && typeof (value as { toString: () => string }).toString === "function") {
    return BigInt((value as { toString: () => string }).toString());
  }
  return 0n;
}

export function useProtocolSnapshot(
  walletAddress: string | null = null,
  refreshMs = 5000,
): ProtocolSnapshot {
  const [snapshot, setSnapshot] = useState<ProtocolSnapshot>({
    state: null,
    balances: null,
    userBalances: null,
    error: null,
    lastFetchedAt: null,
  });

  useEffect(() => {
    let cancelled = false;
    const connection = new Connection(solanaRpcUrl, "confirmed");
    const coder = new BorshAccountsCoder(idl as Idl);

    async function fetchOnce() {
      try {
        const baseKeys = [
          new PublicKey(solanaAddresses.state),
          new PublicKey(solanaAddresses.treasuryAgc),
          new PublicKey(solanaAddresses.treasuryUsdc),
          new PublicKey(solanaAddresses.xagcVaultAgc),
          new PublicKey(solanaAddresses.underwriterVaultAgc),
          new PublicKey(solanaAddresses.agcMint),
          new PublicKey(solanaAddresses.xagcMint),
        ];
        const wallet = walletAddress ? new PublicKey(walletAddress) : null;
        const userAtas = wallet
          ? {
              agc: getAssociatedTokenAddressSync(new PublicKey(solanaAddresses.agcMint), wallet),
              xagc: getAssociatedTokenAddressSync(new PublicKey(solanaAddresses.xagcMint), wallet),
              btc: getAssociatedTokenAddressSync(new PublicKey(solanaAddresses.btcMint), wallet),
            }
          : null;
        const keys = userAtas
          ? [...baseKeys, userAtas.agc, userAtas.xagc, userAtas.btc]
          : baseKeys;
        const infos = await connection.getMultipleAccountsInfo(keys);

        const stateInfo = infos[0];
        const stateView: ProtocolStateView | null = stateInfo
          ? (() => {
              const decoded = coder.decode("ProtocolState", stateInfo.data) as Record<
                string,
                unknown
              >;
              return {
                regime: regimeFromVariant(decoded.regime as Record<string, unknown>),
                anchorPriceX18: toBigInt(decoded.anchor_price_x18),
                lastSettledEpoch: toBigInt(decoded.last_settled_epoch),
                lastReserveCoverageBps: toBigInt(decoded.last_coverage_bps),
                lastStableCashCoverageBps: toBigInt(decoded.last_stable_cash_coverage_bps),
                lastPremiumBps: toBigInt(decoded.last_premium_bps),
                lastLockedShareBps: toBigInt(decoded.last_locked_share_bps),
                lastExitPressureBps: toBigInt(decoded.last_exit_pressure_bps),
                lastVolatilityBps: toBigInt(decoded.last_volatility_bps),
                creditPrincipalOutstandingAgc: toBigInt(
                  decoded.credit_principal_outstanding_agc,
                ),
                creditDrawnAgc: toBigInt(decoded.credit_drawn_agc),
                creditRepaidAgc: toBigInt(decoded.credit_repaid_agc),
                creditInterestPaidAgc: toBigInt(decoded.credit_interest_paid_agc),
                creditDefaultedAgc: toBigInt(decoded.credit_defaulted_agc),
              };
            })()
          : null;

        const tokenAt = (idx: number) => {
          const info = infos[idx];
          if (!info) return 0n;
          const account = unpackAccount(keys[idx], info);
          return account.amount;
        };
        const mintSupply = (idx: number) => {
          const info = infos[idx];
          if (!info) return 0n;
          const mint = unpackMint(keys[idx], info);
          return mint.supply;
        };

        const balances: TokenBalances = {
          treasuryAgc: tokenAt(1),
          treasuryUsdc: tokenAt(2),
          xagcVaultAgc: tokenAt(3),
          underwriterVaultAgc: tokenAt(4),
          agcSupply: mintSupply(5),
          xagcSupply: mintSupply(6),
        };
        const userBalances: UserBalances | null = userAtas
          ? {
              agc: tokenAt(7),
              xagc: tokenAt(8),
              btc: tokenAt(9),
            }
          : null;

        if (!cancelled) {
          setSnapshot({
            state: stateView,
            balances,
            userBalances,
            error: null,
            lastFetchedAt: Date.now(),
          });
        }
      } catch (err) {
        if (!cancelled) {
          setSnapshot((prev) => ({
            ...prev,
            error: err instanceof Error ? err.message : String(err),
          }));
        }
      }
    }

    fetchOnce();
    const interval = setInterval(fetchOnce, refreshMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [refreshMs, walletAddress]);

  return snapshot;
}

const AGC_DECIMALS = 9;
const USDC_DECIMALS = 6;
const PRICE_SCALE = 10n ** 18n;
const BPS_SCALE = 10000n;

export function formatAgc(amount: bigint): string {
  const whole = amount / 10n ** BigInt(AGC_DECIMALS);
  if (whole >= 1_000_000n) return `${(Number(whole) / 1_000_000).toFixed(2)}M`;
  if (whole >= 1_000n) return `${(Number(whole) / 1_000).toFixed(2)}k`;
  return whole.toString();
}

export function formatUsdc(amount: bigint): string {
  const whole = amount / 10n ** BigInt(USDC_DECIMALS);
  if (whole >= 1_000_000n) return `$${(Number(whole) / 1_000_000).toFixed(2)}M`;
  if (whole >= 1_000n) return `$${(Number(whole) / 1_000).toFixed(2)}k`;
  return `$${whole.toString()}`;
}

export function formatPrice(priceX18: bigint): string {
  if (priceX18 === 0n) return "—";
  const cents = Number((priceX18 * 100n) / PRICE_SCALE);
  return `$${(cents / 100).toFixed(4)}`;
}

export function formatBps(bps: bigint): string {
  if (bps === 0n) return "0.00%";
  const pct = Number((bps * 100n) / BPS_SCALE);
  return `${(pct / 100).toFixed(2)}%`;
}

export function regimeLabel(regime: ProtocolStateView["regime"] | undefined): string {
  if (!regime) return "—";
  return regime.charAt(0).toUpperCase() + regime.slice(1);
}
