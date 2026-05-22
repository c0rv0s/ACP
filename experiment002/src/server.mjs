import fs from "node:fs";
import http from "node:http";
import path from "node:path";

import { IDL_PATH, ROOT_DIR, SOLANA_DIR, deploymentExists, loadDeployment } from "./chain.mjs";
import { createTxService } from "./tx-service.mjs";

const port = Number.parseInt(process.env.PORT ?? "8082", 10);
const web3BundlePath = path.join(SOLANA_DIR, "node_modules", "@solana", "web3.js", "lib", "index.iife.min.js");

const server = http.createServer(async (request, response) => {
  try {
    const url = new URL(request.url ?? "/", `http://${request.headers.host ?? "localhost"}`);
    const method = request.method ?? "GET";

    if (method === "OPTIONS") {
      return writeJson(response, 204, {});
    }

    if (method === "GET" && url.pathname === "/health") {
      if (!deploymentExists()) {
        return writeJson(response, 200, {
          ok: false,
          service: "agc-experiment-002",
          message: "Run npm run setup:local before using onchain mode.",
        });
      }
      return writeJson(response, 200, await createTxService().health());
    }

    if (method === "GET" && url.pathname === "/v1/deployment") {
      return writeJson(response, 200, publicDeployment(loadDeployment()));
    }

    if (method === "GET" && url.pathname === "/v1/idl") {
      return sendFile(response, IDL_PATH, "application/json");
    }

    if (method === "GET" && url.pathname === "/v1/state") {
      return writeJson(response, 200, await createTxService().state());
    }

    if (method === "POST" && url.pathname.startsWith("/v1/transactions/")) {
      const kind = url.pathname.split("/").pop();
      return writeJson(response, 200, await createTxService().buildTransaction(kind, await readJson(request)));
    }

    if (method === "POST" && url.pathname === "/v1/airdrop") {
      return writeJson(response, 200, await createTxService().airdrop(await readJson(request)));
    }

    if (method === "GET" && url.pathname === "/vendor/solana-web3.js") {
      return sendFile(response, web3BundlePath, "text/javascript");
    }

    return serveStatic(response, url.pathname);
  } catch (error) {
    const status = error.statusCode ?? 500;
    writeJson(response, status, {
      error: error.code ?? "REQUEST_FAILED",
      message: error instanceof Error ? error.message : "Unknown error",
    });
  }
});

server.listen(port, "127.0.0.1", () => {
  console.log(`AGC Experiment 002 dashboard listening on http://127.0.0.1:${port}`);
});

function publicDeployment(deployment) {
  return {
    generatedAt: deployment.generatedAt,
    rpcUrl: deployment.rpcUrl,
    programId: deployment.programId,
    publicKeys: deployment.publicKeys,
    addresses: deployment.addresses,
  };
}

function serveStatic(response, pathname) {
  const normalized = pathname === "/" ? "/index.html" : pathname;
  const requested = path.normalize(decodeURIComponent(normalized)).replace(/^(\.\.[/\\])+/, "");
  const file = path.join(ROOT_DIR, requested);
  if (!file.startsWith(ROOT_DIR)) {
    return writeJson(response, 403, { error: "FORBIDDEN" });
  }
  const type =
    file.endsWith(".html")
      ? "text/html"
      : file.endsWith(".css")
        ? "text/css"
        : file.endsWith(".js") || file.endsWith(".mjs")
          ? "text/javascript"
          : file.endsWith(".svg")
            ? "image/svg+xml"
            : "application/octet-stream";
  return sendFile(response, file, type);
}

function sendFile(response, file, contentType) {
  if (!fs.existsSync(file) || !fs.statSync(file).isFile()) {
    return writeJson(response, 404, { error: "NOT_FOUND", message: `${file} does not exist` });
  }
  response.writeHead(200, { "content-type": contentType });
  fs.createReadStream(file).pipe(response);
}

function readJson(request) {
  return new Promise((resolve, reject) => {
    let raw = "";
    request.on("data", (chunk) => {
      raw += chunk;
      if (raw.length > 1_000_000) {
        reject(Object.assign(new Error("Request body too large"), { statusCode: 413, code: "BODY_TOO_LARGE" }));
      }
    });
    request.on("end", () => {
      if (!raw) return resolve({});
      try {
        resolve(JSON.parse(raw));
      } catch {
        reject(Object.assign(new Error("Invalid JSON"), { statusCode: 400, code: "INVALID_JSON" }));
      }
    });
  });
}

function writeJson(response, status, body) {
  response.writeHead(status, {
    "content-type": "application/json",
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-headers": "content-type",
  });
  response.end(JSON.stringify(body, null, 2));
}
