// V2-A gate: the zero-trust consensus oracle. Median of independent signed attestations,
// outlier rejection, and the gharar boundary as a computed quantity (majhul -> fairValue reverts).
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ConsensusValuationOracle — zero-trust committee valuation", function () {
  let signers, committee, outsider, oracle, addr, chainId;
  const QUORUM = 3n;
  const BOUND = 500n; // 5%

  beforeEach(async function () {
    signers = await ethers.getSigners();
    committee = signers.slice(0, 5);
    outsider = signers[6];
    const F = await ethers.getContractFactory("ConsensusValuationOracle");
    oracle = await F.deploy(committee.map((s) => s.address), QUORUM, BOUND);
    await oracle.waitForDeployment();
    addr = await oracle.getAddress();
    chainId = (await ethers.provider.getNetwork()).chainId;
  });

  async function sign(signer, roundId, value) {
    const digest = ethers.solidityPackedKeccak256(
      ["uint256", "uint256", "address", "uint256"],
      [roundId, value, addr, chainId]
    );
    return await signer.signMessage(ethers.getBytes(digest));
  }

  async function batch(roundId, pairs) {
    const values = pairs.map((p) => p.v);
    const sigs = [];
    for (const p of pairs) sigs.push(await sign(p.s, roundId, p.v));
    return { values, sigs };
  }

  it("resolves the median when independent attestors agree within the gharar band", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1010000n },
      { s: committee[2], v: 990000n },
      { s: committee[3], v: 1005000n },
      { s: committee[4], v: 995000n },
    ]);
    await expect(oracle.submitRound(1n, values, sigs)).to.emit(oracle, "RoundResolved");
    expect(await oracle.fairValue()).to.equal(1000000n);
  });

  it("rejects an outlier attestation and resolves over the inliers", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1010000n },
      { s: committee[2], v: 990000n },
      { s: committee[3], v: 1005000n },
      { s: committee[4], v: 2000000n }, // outlier, outside +/-5% band
    ]);
    await oracle.submitRound(1n, values, sigs);
    const fv = await oracle.fairValue();
    expect(fv).to.be.greaterThan(989000n);
    expect(fv).to.be.lessThan(1011000n); // the 2,000,000 outlier was excluded
  });

  it("treats wide disagreement as gharar: the value is majhul and fairValue() reverts", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1500000n },
      { s: committee[2], v: 2000000n },
      { s: committee[3], v: 800000n },
      { s: committee[4], v: 3000000n },
    ]);
    await expect(oracle.submitRound(1n, values, sigs)).to.emit(oracle, "Undeterminable");
    await expect(oracle.fairValue()).to.be.revertedWith("gharar: latest valuation round is undeterminable");
  });

  it("rejects a signature from outside the committee (zero-trust in the relayer)", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1000000n },
      { s: outsider, v: 1000000n }, // not on the committee
    ]);
    await expect(oracle.submitRound(1n, values, sigs)).to.be.revertedWith("signer not on committee");
  });

  it("rejects a member signing twice in one round", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[0], v: 1000000n }, // same member twice
      { s: committee[1], v: 1000000n },
    ]);
    await expect(oracle.submitRound(1n, values, sigs)).to.be.revertedWith("double-sign by a member");
  });

  it("requires at least a quorum of attestations", async function () {
    const { values, sigs } = await batch(1n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1000000n },
    ]);
    await expect(oracle.submitRound(1n, values, sigs)).to.be.revertedWith("fewer attestations than quorum");
  });

  it("enforces sequential rounds", async function () {
    const { values, sigs } = await batch(2n, [
      { s: committee[0], v: 1000000n },
      { s: committee[1], v: 1000000n },
      { s: committee[2], v: 1000000n },
    ]);
    await expect(oracle.submitRound(2n, values, sigs)).to.be.revertedWith("round must be sequential");
  });
});
