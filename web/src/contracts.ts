export const addresses = {
  agc: import.meta.env.VITE_AGC_ADDRESS as `0x${string}` | undefined,
  hook: import.meta.env.VITE_HOOK_ADDRESS as `0x${string}` | undefined,
  policyController: import.meta.env.VITE_POLICY_CONTROLLER_ADDRESS as `0x${string}` | undefined,
  rewardDistributor: import.meta.env.VITE_REWARD_DISTRIBUTOR_ADDRESS as `0x${string}` | undefined,
  settlementRouter: import.meta.env.VITE_SETTLEMENT_ROUTER_ADDRESS as `0x${string}` | undefined,
};

export const facilitatorApiUrl =
  (import.meta.env.VITE_FACILITATOR_API_URL as string | undefined) ??
  "http://127.0.0.1:8787";

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
    name: "claimProductiveReceipt",
    stateMutability: "nonpayable",
    inputs: [{ name: "receiptId", type: "uint256" }],
    outputs: [{ name: "streamId", type: "uint256" }],
  },
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
  {
    type: "event",
    name: "ReceiptClaimed",
    inputs: [
      { indexed: true, name: "receiptId", type: "uint256" },
      { indexed: true, name: "streamId", type: "uint256" },
      { indexed: true, name: "beneficiary", type: "address" },
      { indexed: false, name: "amount", type: "uint256" },
    ],
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
    ],
    outputs: [{ name: "usdcAmountOut", type: "uint256" }],
  },
  {
    type: "function",
    name: "settleProductivePayment",
    stateMutability: "nonpayable",
    inputs: [
      {
        name: "attestation",
        type: "tuple",
        components: [
          { name: "payer", type: "address" },
          { name: "recipient", type: "address" },
          { name: "agcAmountIn", type: "uint256" },
          { name: "paymentId", type: "bytes32" },
          { name: "qualityScoreBps", type: "uint16" },
          { name: "deadline", type: "uint64" },
          { name: "routeHash", type: "bytes32" },
        ],
      },
      { name: "minUsdcOut", type: "uint256" },
      { name: "facilitator", type: "address" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "usdcAmountOut", type: "uint256" }],
  },
] as const;

export const hookAbi = [
  {
    type: "event",
    name: "RewardReceiptCreated",
    inputs: [
      { indexed: true, name: "receiptId", type: "uint256" },
      { indexed: true, name: "epochId", type: "uint64" },
      { indexed: true, name: "beneficiary", type: "address" },
      { indexed: false, name: "intentHash", type: "bytes32" },
      { indexed: false, name: "usdcAmount", type: "uint256" },
    ],
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
