export const addresses = {
  agc: import.meta.env.VITE_AGC_ADDRESS as `0x${string}` | undefined,
  policyController: import.meta.env.VITE_POLICY_CONTROLLER_ADDRESS as `0x${string}` | undefined,
  rewardDistributor: import.meta.env.VITE_REWARD_DISTRIBUTOR_ADDRESS as `0x${string}` | undefined,
  settlementRouter: import.meta.env.VITE_SETTLEMENT_ROUTER_ADDRESS as `0x${string}` | undefined,
};

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
    name: "bandWidthBps",
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
    name: "lastProductiveUsageBps",
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
] as const;

export const rewardDistributorAbi = [
  {
    type: "function",
    name: "claimStream",
    stateMutability: "nonpayable",
    inputs: [{ name: "streamId", type: "uint256" }],
    outputs: [{ name: "claimedAmount", type: "uint256" }],
  },
  {
    type: "function",
    name: "previewClaimable",
    stateMutability: "view",
    inputs: [{ name: "streamId", type: "uint256" }],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

export const settlementRouterAbi = [
  {
    type: "function",
    name: "settlePayment",
    stateMutability: "nonpayable",
    inputs: [
      { name: "agcAmountIn", type: "uint256" },
      { name: "minUsdcOut", type: "uint256" },
      { name: "recipient", type: "address" },
      { name: "paymentId", type: "bytes32" },
      { name: "qualityScoreBps", type: "uint16" },
    ],
    outputs: [{ name: "usdcAmountOut", type: "uint256" }],
  },
] as const;

export const agcAbi = [
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
    name: "balanceOf",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;
