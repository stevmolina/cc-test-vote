// End-to-end check of the web client against the deployed contract, using a
// local Testnet key instead of Freighter. Usage:
//   node scripts/smoke-test.cjs <stellar-key-alias>
// Each wallet can vote only once, so if the alias already voted, the test
// uses a fresh throwaway Testnet key funded by Friendbot.
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const StellarSdk = require("@stellar/stellar-sdk");
const makeClient = require("../docs/vote-client.js");

const alias = process.argv[2] || "me";
const cfgSrc = fs.readFileSync(path.join(__dirname, "../docs/config.js"), "utf8");
const window = {};
new Function("window", cfgSrc)(window);
const cfg = window.CONFIG;
const PROJECT = "Commit Club Vote";

(async () => {
  const client = makeClient(StellarSdk, cfg);
  let kp = StellarSdk.Keypair.fromSecret(execSync(`stellar keys secret ${alias}`).toString().trim());
  if (await client.votedFor(kp.publicKey())) {
    kp = StellarSdk.Keypair.random();
    const r = await fetch(`https://friendbot.stellar.org/?addr=${kp.publicKey()}`);
    if (!r.ok) throw new Error("Friendbot failed: " + r.status);
    console.log("alias already voted, using fresh key " + kp.publicKey());
  }
  const sign = async (xdr) => {
    const tx = StellarSdk.TransactionBuilder.fromXDR(xdr, cfg.networkPassphrase);
    tx.sign(kp);
    return tx.toXDR();
  };

  const before = (await client.ranking()).find((r) => r.project === PROJECT)?.votes || 0;
  const { hash, votes } = await client.vote(kp.publicKey(), PROJECT, sign);
  const after = (await client.ranking()).find((r) => r.project === PROJECT)?.votes || 0;
  const mine = await client.votedFor(kp.publicKey());
  if (votes !== before + 1 || after !== votes || mine !== PROJECT) throw new Error(`unexpected state before=${before} after=${after} votes=${votes} mine=${mine}`);
  let rejected = false;
  try { await client.vote(kp.publicKey(), "Otro", sign); } catch (e) { rejected = /ya votó/.test(e.message); }
  if (!rejected) throw new Error("second vote was not rejected");
  console.log(`OK "${PROJECT}" ${before} -> ${after} votes, second vote rejected`);
  console.log(`Proof: https://stellar.expert/explorer/testnet/tx/${hash}`);
})().catch((e) => { console.error("FAIL", e.message); process.exit(1); });
