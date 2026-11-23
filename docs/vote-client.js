// Tiny client for the Vote contract. Works in the browser (global
// `VoteClient`, with the StellarSdk CDN build) and in Node (require).
// `signXdr(xdr) -> Promise<signedXdr>` is provided by Freighter in the browser
// or by a local Keypair in tests.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory;
  else root.VoteClient = factory;
})(typeof self !== "undefined" ? self : this, function (StellarSdk, cfg) {
  const { rpc, Contract, TransactionBuilder, BASE_FEE, nativeToScVal, scValToNative, Address } = StellarSdk;
  const server = new rpc.Server(cfg.rpcUrl);
  const contract = new Contract(cfg.contractId);

  async function build(source, method, args) {
    const account = await server.getAccount(source);
    return new TransactionBuilder(account, { fee: BASE_FEE, networkPassphrase: cfg.networkPassphrase })
      .addOperation(contract.call(method, ...args))
      .setTimeout(60)
      .build();
  }

  // Read-only call: simulate, never submit. Any funded account works as source.
  async function read(method, args = []) {
    const tx = await build(cfg.readAccount, method, args);
    const sim = await server.simulateTransaction(tx);
    if (rpc.Api.isSimulationError(sim)) throw new Error(sim.error);
    return scValToNative(sim.result.retval);
  }

  // Ranking, most votes first: [{ project, votes }].
  async function ranking() {
    const rows = (await read("results")) || [];
    return rows
      .map((r) => ({ project: String(r.project), votes: Number(r.votes) }))
      .sort((a, b) => b.votes - a.votes || a.project.localeCompare(b.project));
  }

  // Project name this wallet voted for, or null.
  async function votedFor(voter) {
    const v = await read("voted_for", [new Address(voter).toScVal()]);
    return v == null ? null : String(v);
  }

  // Write: build -> simulate/prepare -> sign -> send -> wait.
  async function vote(voter, project, signXdr) {
    const tx = await build(voter, "vote", [
      new Address(voter).toScVal(),
      nativeToScVal(project, { type: "string" }),
    ]);
    let prepared;
    try {
      prepared = await server.prepareTransaction(tx);
    } catch (e) {
      const m = String(e.message || e);
      if (/Error\(Contract, #1\)/.test(m)) throw new Error("Esta wallet ya votó.");
      if (/Error\(Contract, #2\)/.test(m)) throw new Error("El nombre del proyecto debe tener 1-64 caracteres.");
      if (/Error\(Contract, #3\)/.test(m)) throw new Error("Ya hay demasiados proyectos; votá uno del ranking.");
      throw e;
    }
    const signed = TransactionBuilder.fromXDR(await signXdr(prepared.toXDR()), cfg.networkPassphrase);
    const sent = await server.sendTransaction(signed);
    if (sent.status === "ERROR") throw new Error("send failed: " + JSON.stringify(sent.errorResult));
    let res = await server.getTransaction(sent.hash);
    for (let i = 0; i < 30 && res.status === "NOT_FOUND"; i++) {
      await new Promise((r) => setTimeout(r, 1000));
      res = await server.getTransaction(sent.hash);
    }
    if (res.status !== "SUCCESS") throw new Error("tx " + res.status);
    return { hash: sent.hash, votes: scValToNative(res.returnValue) };
  }

  return { ranking, votedFor, vote };
});
