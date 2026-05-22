import { loadKeypair, keyPaths } from "../src/chain.mjs";

const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const keypair = loadKeypair(keyPaths().admin);
const secretKey = Uint8Array.from(keypair.secretKey);

console.log(`Public key: ${keypair.publicKey.toBase58()}`);
console.log(`Secret key JSON: ${JSON.stringify(Array.from(secretKey))}`);
console.log(`Secret key base58: ${base58Encode(secretKey)}`);
console.log("");
console.log("Import this local-only key into Phantom or another Solana wallet for the local validator.");

function base58Encode(bytes) {
  if (bytes.length === 0) return "";

  const digits = [0];
  for (const byte of bytes) {
    let carry = byte;
    for (let index = 0; index < digits.length; index += 1) {
      carry += digits[index] << 8;
      digits[index] = carry % 58;
      carry = Math.floor(carry / 58);
    }
    while (carry > 0) {
      digits.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }

  for (const byte of bytes) {
    if (byte !== 0) break;
    digits.push(0);
  }

  return digits.reverse().map((digit) => alphabet[digit]).join("");
}
