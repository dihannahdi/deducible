const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("MusharakahMutanaqisahV5 - defect rescission, judicial faskh, maslahah disposition", function () {
  let bank, client, valuer, arbiter, maslahah, oracle, c;
  const V0 = 100000000n;
  const KHIYAR = 3600;

  beforeEach(async function () {
    [bank, client, valuer, arbiter, maslahah] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(V0);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("MusharakahMutanaqisahV5");
    c = await F.connect(bank).deploy(client.address, await oracle.getAddress(), arbiter.address, maslahah.address, 8000n, 1n, KHIYAR);
    await c.waitForDeployment();
  });
  async function fund() {
    await c.connect(bank).fundBank({ value: (V0 * 8000n) / 10000n });
    await c.connect(client).fundClient({ value: (V0 * 2000n) / 10000n });
  }

  it("CORRECTION 2: settle directs the impaired loss residue to the maslahah fund (no wealth stranded)", async function () {
    await fund();
    await oracle.connect(valuer).attest(90000000n);
    const before = await ethers.provider.getBalance(maslahah.address);
    await expect(c.connect(bank).settle()).to.emit(c, "Settled").withArgs(90000000n, 72000000n, 18000000n, 10000000n);
    const after = await ethers.provider.getBalance(maslahah.address);
    expect(after - before).to.equal(10000000n);
  });

  it("CORRECTION 1a: khiyar al-'ayb - a raised defect is adjudicated by the agreed arbiter only", async function () {
    await fund();
    await c.connect(client).raiseDefect("latent structural defect");
    await expect(c.connect(bank).resolveDefect(true)).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).resolveDefect(true)).to.emit(c, "Unwound");
    expect(await c.rescinded()).to.equal(true);
  });

  it("a defect the arbiter does not uphold leaves the partnership live", async function () {
    await fund();
    await c.connect(client).raiseDefect("alleged defect");
    await c.connect(arbiter).resolveDefect(false);
    expect(await c.rescinded()).to.equal(false);
    expect(await c.active()).to.equal(true);
  });

  it("CORRECTION 1b: judicial faskh - only the arbiter may rescind by authority", async function () {
    await fund();
    await expect(c.connect(client).judicialFaskh()).to.be.revertedWith("only arbiter");
    await expect(c.connect(arbiter).judicialFaskh()).to.emit(c, "Unwound");
    expect(await c.rescinded()).to.equal(true);
  });

  it("khiyar al-shart + iqalah still hold in V5", async function () {
    await fund();
    await c.connect(bank).proposeIqalah();
    await expect(c.connect(client).acceptIqalah()).to.emit(c, "IqalahCompleted").withArgs(client.address);
    expect(await c.rescinded()).to.equal(true);
  });
});
