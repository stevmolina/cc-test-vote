# Commit Club Vote 🗳️

Vote for the best **Commit Club** project on **Stellar Testnet**. One wallet, one vote; the page shows the live ranking. No real money.

Built at BAF × Stellar from the [Proof of Thanks starter](https://github.com/stevmolina/proof-of-thanks-starter).

**Live page:** https://stevmolina.github.io/cc-test-vote/

## What's inside

- `contracts/vote`: a Soroban contract with
  - `vote(voter, project)`: the wallet signs and casts its single vote (a second vote is rejected). Returns the project's new total.
  - `results()`: every project with its vote count.
  - `voted_for(voter)`: what a wallet voted for, if anything.
- `docs/`: a static page that connects [Freighter](https://www.freighter.app), shows the ranking and lets you vote. Published with GitHub Pages from `/docs`.
- `scripts/`: `setup.sh` (tools + funded Testnet account), `deploy.sh` (test, build, deploy), `smoke-test.cjs` (end-to-end check).

## Quick start

```bash
bash scripts/setup.sh         # once
bash scripts/deploy.sh vote   # deploy the contract to Testnet
node scripts/smoke-test.cjs me
npx --yes serve docs          # open the page locally
```

## Wallet setup

1. Install [Freighter](https://www.freighter.app).
2. Settings → Network → **Testnet**.
3. Fund your account for free with [Friendbot](https://lab.stellar.org/account/fund).
