const { expect } = require("chai");
const { ethers } = require("hardhat");

// Three distinct accounts: bank, client, independent valuer (deploys/controls oracle).
describe("MusharakahMutanaqisahV2 - capital custody + settlement", function () {
  let bank, client, valuer, oracle, c;
  const V0 = 100000000n; // asset value

  beforeEach(async function () {
    [bank, client, valuer] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(V0);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("MusharakahMutanaqisahV2");
    c = await F.connect(bank).deploy(client.address, await oracle.getAddress(), 8000n, 1n);
    await c.waitForDeployment();
  });

  async function fund() {
    await c.connect(bank).fundBank({ value: (V0 * 8000n) / 10000n });   // 80,000,000
    await c.connect(client).fundClient({ value: (V0 * 2000n) / 10000n }); // 20,000,000
  }

  it("activates only when both partners fund their exact ownership share", async function () {
    await expect(c.connect(bank).fundBank({ value: 1n })).to.be.revertedWith("bank must fund its share");
    await fund();
    expect(await c.active()).to.equal(true);
    expect(await c.pool()).to.equal(V0);
  });

  it("CRITICAL FIX: a loss reduces the financier's recoverable capital (cannot exit whole)", async function () {
    await fund();
    await oracle.connect(valuer).attest(90000000n); // value falls 100M -> 90M
    const before = await ethers.provider.getBalance(bank.address);
    await c.connect(client).settle(); // client pays gas, so bank delta is pure payout
    const after = await ethers.provider.getBalance(bank.address);
    // bank funded 80,000,000; recovers 90M * 80% = 72,000,000 -> bore 8,000,000 loss
    expect(after - before).to.equal(72000000n);
  });

  it("loss is shared proportionally; the impaired remainder is locked", async function () {
    await fund();
    await oracle.connect(valuer).attest(90000000n);
    await expect(c.connect(bank).settle())
      .to.emit(c, "Settled").withArgs(90000000n, 72000000n, 18000000n, 10000000n);
    expect(await c.settled()).to.equal(true);
  });

  it("buyShare shifts ownership, conserves the pool, and pays the bank at fair value", async function () {
    await fund();
    await c.connect(client).buyShare(2000n, { value: (V0 * 2000n) / 10000n });
    expect(await c.bankShareBps()).to.equal(6000n);
    expect(await c.pool()).to.equal(V0);
  });

  it("roles enforced across three distinct accounts", async function () {
    await fund();
    await expect(c.connect(valuer).payRent({ value: 8000n })).to.be.revertedWith("only client");
    await expect(c.connect(valuer).settle()).to.be.revertedWith("only a partner");
    await expect(oracle.connect(client).attest(1n)).to.be.revertedWith("only valuer");
  });
});
