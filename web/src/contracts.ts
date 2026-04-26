export const solanaAddresses = {
  programId: import.meta.env.VITE_SOLANA_PROGRAM_ID as string | undefined,
  state: import.meta.env.VITE_SOLANA_STATE as string | undefined,
  agcMint: import.meta.env.VITE_SOLANA_AGC_MINT as string | undefined,
  xagcMint: import.meta.env.VITE_SOLANA_XAGC_MINT as string | undefined,
  usdcMint:
    (import.meta.env.VITE_SOLANA_USDC_MINT as string | undefined) ??
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  treasuryAgc: import.meta.env.VITE_SOLANA_TREASURY_AGC as string | undefined,
  treasuryUsdc: import.meta.env.VITE_SOLANA_TREASURY_USDC as string | undefined,
  xagcVaultAgc: import.meta.env.VITE_SOLANA_XAGC_VAULT_AGC as string | undefined,
};

export const hasSolanaDeployment = Boolean(
  solanaAddresses.programId &&
    solanaAddresses.state &&
    solanaAddresses.agcMint &&
    solanaAddresses.xagcMint,
);
