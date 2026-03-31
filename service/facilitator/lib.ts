import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  keccak256,
  isAddress,
  isHex,
  toHex,
  type Address,
  type Hex,
} from "viem";
import { privateKeyToAccount } from "viem/accounts";

export type PartnerPolicy = {
  key: string;
  name: string;
  routeTag: string;
  qualityScoreBps: number;
  ttlSeconds: number;
  description?: string;
};

type PartnerPolicyFile = {
  partners: PartnerPolicy[];
};

export type FacilitatorConfig = {
  chainId: number;
  routerAddress: Address;
  partnerPolicies: PartnerPolicy[];
  facilitatorPrivateKey: Hex;
};

export type ProductivePaymentRequest = {
  payer: Address;
  recipient: Address;
  agcAmountIn: bigint;
  paymentId: Hex;
  partnerKey: string;
};

export type ProductivePaymentResponse = {
  facilitator: Address;
  attestation: {
    payer: Address;
    recipient: Address;
    agcAmountIn: string;
    paymentId: Hex;
    qualityScoreBps: number;
    deadline: number;
    routeHash: Hex;
  };
  signature: Hex;
  domain: {
    name: string;
    version: string;
    chainId: number;
    verifyingContract: Address;
  };
};

export class FacilitatorRequestError extends Error {
  statusCode: number;

  constructor(message: string, statusCode = 400) {
    super(message);
    this.name = "FacilitatorRequestError";
    this.statusCode = statusCode;
  }
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const defaultPartnerPolicyPath = path.resolve(
  __dirname,
  "../../configs/facilitator/partners.json",
);

export async function loadFacilitatorConfig(
  overrides: Partial<FacilitatorConfig> = {},
): Promise<FacilitatorConfig> {
  const partnerPolicyPath =
    process.env.FACILITATOR_PARTNER_CONFIG_PATH ?? defaultPartnerPolicyPath;
  const file = await readFile(partnerPolicyPath, "utf8");
  const parsed = JSON.parse(file) as PartnerPolicyFile;

  const facilitatorPrivateKey = (overrides.facilitatorPrivateKey ??
    process.env.FACILITATOR_PRIVATE_KEY) as Hex | undefined;
  const routerAddress = (overrides.routerAddress ??
    process.env.FACILITATOR_ROUTER_ADDRESS) as Address | undefined;
  const chainIdValue = overrides.chainId ?? Number(process.env.FACILITATOR_CHAIN_ID ?? "31337");

  if (!facilitatorPrivateKey || !isHex(facilitatorPrivateKey, { strict: true })) {
    throw new Error("FACILITATOR_PRIVATE_KEY must be set to a 32-byte hex string.");
  }
  if (!routerAddress || !isAddress(routerAddress)) {
    throw new Error("FACILITATOR_ROUTER_ADDRESS must be set to a valid address.");
  }

  return {
    chainId: chainIdValue,
    routerAddress,
    partnerPolicies: overrides.partnerPolicies ?? parsed.partners,
    facilitatorPrivateKey,
  };
}

export function publicFacilitatorConfig(config: FacilitatorConfig) {
  const account = privateKeyToAccount(config.facilitatorPrivateKey);
  return {
    facilitator: account.address,
    routerAddress: config.routerAddress,
    chainId: config.chainId,
    partners: config.partnerPolicies.map((partner) => ({
      key: partner.key,
      name: partner.name,
      description: partner.description ?? "",
      qualityScoreBps: partner.qualityScoreBps,
      ttlSeconds: partner.ttlSeconds,
      routeHash: keccak256(toHex(partner.routeTag)),
    })),
  };
}

export async function attestProductivePayment(
  config: FacilitatorConfig,
  request: ProductivePaymentRequest,
  now: number = Math.floor(Date.now() / 1000),
): Promise<ProductivePaymentResponse> {
  if (!isAddress(request.payer)) {
    throw new FacilitatorRequestError("Invalid payer address.");
  }
  if (!isAddress(request.recipient)) {
    throw new FacilitatorRequestError("Invalid recipient address.");
  }
  if (!isHex(request.paymentId, { strict: true }) || request.paymentId.length !== 66) {
    throw new FacilitatorRequestError("paymentId must be a 32-byte hex string.");
  }
  if (request.agcAmountIn <= 0n) {
    throw new FacilitatorRequestError("agcAmountIn must be greater than zero.");
  }

  const partner = config.partnerPolicies.find((entry) => entry.key === request.partnerKey);
  if (!partner) {
    throw new FacilitatorRequestError("Unknown partnerKey.", 404);
  }

  const account = privateKeyToAccount(config.facilitatorPrivateKey);
  const attestation = {
    payer: request.payer,
    recipient: request.recipient,
    agcAmountIn: request.agcAmountIn,
    paymentId: request.paymentId,
    qualityScoreBps: partner.qualityScoreBps,
    deadline: now + partner.ttlSeconds,
    routeHash: keccak256(toHex(partner.routeTag)),
  };

  const domain = {
    name: "AgentCreditSettlementRouter",
    version: "1",
    chainId: config.chainId,
    verifyingContract: config.routerAddress,
  } as const;

  const signature = await account.signTypedData({
    domain,
    primaryType: "ProductivePaymentAttestation",
    types: {
      ProductivePaymentAttestation: [
        { name: "payer", type: "address" },
        { name: "recipient", type: "address" },
        { name: "agcAmountIn", type: "uint256" },
        { name: "paymentId", type: "bytes32" },
        { name: "qualityScoreBps", type: "uint16" },
        { name: "deadline", type: "uint64" },
        { name: "routeHash", type: "bytes32" },
      ],
    },
    message: attestation,
  });

  return {
    facilitator: account.address,
    attestation: {
      ...attestation,
      agcAmountIn: attestation.agcAmountIn.toString(),
    },
    signature,
    domain,
  };
}
