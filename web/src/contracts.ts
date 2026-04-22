export const addresses = {
  agc: import.meta.env.VITE_AGC_ADDRESS as `0x${string}` | undefined,
  usdc: import.meta.env.VITE_USDC_ADDRESS as `0x${string}` | undefined,
  hook: import.meta.env.VITE_HOOK_ADDRESS as `0x${string}` | undefined,
  policyController: import.meta.env.VITE_POLICY_CONTROLLER_ADDRESS as `0x${string}` | undefined,
  settlementRouter: import.meta.env.VITE_SETTLEMENT_ROUTER_ADDRESS as `0x${string}` | undefined,
  treasuryVault: import.meta.env.VITE_TREASURY_VAULT_ADDRESS as `0x${string}` | undefined,
  xagcVault: import.meta.env.VITE_XAGC_VAULT_ADDRESS as `0x${string}` | undefined,
};

export const erc20Abi = [
  {
    type: "function",
    name: "approve",
    stateMutability: "nonpayable",
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
  {
    type: "function",
    name: "allowance",
    stateMutability: "view",
    inputs: [
      { name: "owner", type: "address" },
      { name: "spender", type: "address" },
    ],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "balanceOf",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

export const agcAbi = erc20Abi;

export const policyControllerAbi = [
  {
    type: "function",
    name: "anchorPriceX18",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "regime",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint8" }],
  },
  {
    type: "function",
    name: "lastPremiumBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "lastCoverageBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "lastExitPressureBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "lastVolatilityBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "lastLockedShareBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "lastLockFlowBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "pendingTreasuryBuybackUsdc",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

export const settlementRouterAbi = [
  {
    type: "function",
    name: "buyAGC",
    stateMutability: "nonpayable",
    inputs: [
      { name: "usdcAmountIn", type: "uint256" },
      { name: "minAgcOut", type: "uint256" },
      { name: "recipient", type: "address" },
      { name: "refId", type: "bytes32" },
    ],
    outputs: [{ name: "agcAmountOut", type: "uint256" }],
  },
  {
    type: "function",
    name: "sellAGC",
    stateMutability: "nonpayable",
    inputs: [
      { name: "agcAmountIn", type: "uint256" },
      { name: "minUsdcOut", type: "uint256" },
      { name: "recipient", type: "address" },
      { name: "refId", type: "bytes32" },
    ],
    outputs: [{ name: "usdcAmountOut", type: "uint256" }],
  },
] as const;

export const hookAbi = [
  {
    type: "function",
    name: "currentEpochId",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint64" }],
  },
  {
    type: "function",
    name: "currentAccumulator",
    stateMutability: "view",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "tuple",
        components: [
          { name: "epochId", type: "uint64" },
          { name: "startedAt", type: "uint64" },
          { name: "updatedAt", type: "uint64" },
          { name: "lastObservedAt", type: "uint64" },
          { name: "observationCount", type: "uint64" },
          { name: "grossBuyVolumeQuoteX18", type: "uint256" },
          { name: "grossSellVolumeQuoteX18", type: "uint256" },
          { name: "totalVolumeQuoteX18", type: "uint256" },
          { name: "lastMidPriceX18", type: "uint256" },
          { name: "cumulativeMidPriceTimeX18", type: "uint256" },
          { name: "cumulativeAbsMidPriceChangeBps", type: "uint256" },
          { name: "totalHookFeesQuoteX18", type: "uint256" },
          { name: "totalHookFeesAgc", type: "uint256" },
        ],
      },
    ],
  },
] as const;

export const stabilityVaultAbi = [
  {
    type: "function",
    name: "availableUsdc",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "availableAGC",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

export const xagcVaultAbi = [
  {
    type: "function",
    name: "balanceOf",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "totalAssets",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "totalSupply",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    type: "function",
    name: "exitFeeBps",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint16" }],
  },
  {
    type: "function",
    name: "previewDeposit",
    stateMutability: "view",
    inputs: [{ name: "assets", type: "uint256" }],
    outputs: [{ name: "shares", type: "uint256" }],
  },
  {
    type: "function",
    name: "previewRedeem",
    stateMutability: "view",
    inputs: [{ name: "shares", type: "uint256" }],
    outputs: [
      { name: "netAssets", type: "uint256" },
      { name: "feeAssets", type: "uint256" },
    ],
  },
  {
    type: "function",
    name: "deposit",
    stateMutability: "nonpayable",
    inputs: [
      { name: "assets", type: "uint256" },
      { name: "receiver", type: "address" },
    ],
    outputs: [{ name: "shares", type: "uint256" }],
  },
  {
    type: "function",
    name: "redeem",
    stateMutability: "nonpayable",
    inputs: [
      { name: "shares", type: "uint256" },
      { name: "receiver", type: "address" },
      { name: "owner_", type: "address" },
    ],
    outputs: [{ name: "netAssets", type: "uint256" }],
  },
] as const;
