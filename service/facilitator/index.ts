import { createServer } from "node:http";
import {
  attestProductivePayment,
  FacilitatorRequestError,
  loadFacilitatorConfig,
  publicFacilitatorConfig,
  type ProductivePaymentRequest,
} from "./lib.js";

const port = Number(process.env.PORT ?? "8787");

function sendJson(
  response: Parameters<Parameters<typeof createServer>[0]>[1],
  statusCode: number,
  payload: unknown,
) {
  response.writeHead(statusCode, {
    "content-type": "application/json; charset=utf-8",
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-headers": "content-type",
  });
  response.end(`${JSON.stringify(payload)}\n`);
}

async function readJsonBody<T>(request: Parameters<Parameters<typeof createServer>[0]>[0]) {
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }

  if (chunks.length === 0) {
    throw new FacilitatorRequestError("Request body is required.");
  }

  try {
    return JSON.parse(Buffer.concat(chunks).toString("utf8")) as T;
  } catch (error) {
    throw new FacilitatorRequestError(
      error instanceof Error ? `Invalid JSON body: ${error.message}` : "Invalid JSON body.",
    );
  }
}

async function main() {
  const config = await loadFacilitatorConfig();

  const server = createServer(async (request, response) => {
    if (!request.url) {
      sendJson(response, 404, { error: "Not found." });
      return;
    }

    if (request.method === "OPTIONS") {
      sendJson(response, 204, {});
      return;
    }

    try {
      if (request.method === "GET" && request.url === "/health") {
        sendJson(response, 200, { ok: true, timestamp: Math.floor(Date.now() / 1000) });
        return;
      }

      if (request.method === "GET" && request.url === "/config/public") {
        sendJson(response, 200, publicFacilitatorConfig(config));
        return;
      }

      if (request.method === "POST" && request.url === "/attest/productive-payment") {
        const body = await readJsonBody<ProductivePaymentRequest & Record<string, unknown>>(request);
        const payload = await attestProductivePayment(config, {
          payer: body.payer,
          recipient: body.recipient,
          agcAmountIn: BigInt(body.agcAmountIn),
          paymentId: body.paymentId,
          partnerKey: body.partnerKey,
        });
        sendJson(response, 200, payload);
        return;
      }

      sendJson(response, 404, { error: "Not found." });
    } catch (error) {
      const statusCode =
        error instanceof FacilitatorRequestError ? error.statusCode : 500;
      sendJson(response, statusCode, {
        error: error instanceof Error ? error.message : "Internal server error.",
      });
    }
  });

  server.listen(port, "127.0.0.1", () => {
    console.log(`Facilitator listening on http://127.0.0.1:${port}`);
  });
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack ?? error.message : error);
  process.exitCode = 1;
});
