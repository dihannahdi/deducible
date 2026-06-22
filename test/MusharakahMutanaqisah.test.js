const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("MusharakahMutanaqisah - compliance by construction", function () {
  let bank, client, other, valuer, oracle, c;

  beforeEach(async function () {
    [bank, client, other, valuer] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(1000000n); // independent valuer attests 1,000,000
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("MusharakahMutanaqisah");
    // bank (deployer) owns 8000 bps (80%); client 2000 bps; rent = 1 wei per bps
    c = await F.deploy(client.address, await oracle.getAddress(), 8000n, 1n);
    await c.waitForDeployment();
  });

  it("INV1: rent is charged only on the bank's living share and falls as ownership transfers", async function () {
    expect(await c.rentDue()).to.equal(8000n);
    await c.connect(client).buyShare(2000n, { value: 200000n }); // 1,000,000 * 2000/10000
    expect(await c.bankShareBps()).to.equal(6000n);
    expect(await c.rentDue()).to.equal(6000n);
  });

  it("INV2: the buyer cannot name their own price - it tracks the independent oracle", async function () {
    await expect(c.connect(client).buyShare(2000n, { value: 199999n }))
      .to.be.revertedWith("value must equal fair price of bps bought");
    // when the independent valuer re-attests, the required price moves with it
    await oracle.connect(valuer).attest(1200000n);
    await expect(c.connect(client).buyShare(2000n, { value: 200000n }))
      .to.be.revertedWith("value must equal fair price of bps bought");
    await c.connect(client).buyShare(2000n, { value: 240000n }); // 1,200,000 * 2000/10000
    expect(await c.bankShareBps()).to.equal(6000n);
  });

  it("INV3: a fall in attested value is shared by ownership ratio - the bank cannot exit whole", async function () {
    await oracle.connect(valuer).attest(900000n); // value falls 1,000,000 -> 900,000
    await expect(c.connect(bank).syncValuation())
      .to.emit(c, "LossShared")
      .withArgs(900000n, 80000n, 20000n); // loss 100,000 split 80/20
    expect(await c.lossRecorded()).to.equal(true);
  });

  it("INV4: roles enforced - only a partner syncs, only the valuer attests, lessee pays", async function () {
    await expect(c.connect(other).payRent({ value: 8000n })).to.be.revertedWith("only client");
    await expect(c.connect(other).syncValuation()).to.be.revertedWith("only a partner");
    await expect(oracle.connect(other).attest(123n)).to.be.revertedWith("only valuer");
    await expect(c.connect(client).payRent({ value: 8000n }))
      .to.emit(c, "RentPaid").withArgs(8000n, 8000n);
  });

  it("terminal state: when the client acquires the bank's whole share, rent goes to zero", async function () {
    await c.connect(client).buyShare(8000n, { value: 800000n });
    expect(await c.fullyAcquired()).to.equal(true);
    expect(await c.rentDue()).to.equal(0n);
  });
});
