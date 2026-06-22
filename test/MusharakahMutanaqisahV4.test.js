const { expect } = require("chai");
const { ethers } = require("hardhat");
const { time } = require("@nomicfoundation/hardhat-network-helpers");

describe("MusharakahMutanaqisahV4 - lawful rescission (khiyar + iqalah)", function () {
  let bank, client, valuer, oracle, c;
  const V0 = 100000000n;
  const KHIYAR = 3600; // 1 hour window

  beforeEach(async function () {
    [bank, client, valuer] = await ethers.getSigners();
    const O = await ethers.getContractFactory("MockValuationOracle");
    oracle = await O.connect(valuer).deploy(V0);
    await oracle.waitForDeployment();
    const F = await ethers.getContractFactory("MusharakahMutanaqisahV4");
    c = await F.connect(bank).deploy(client.address, await oracle.getAddress(), 8000n, 1n, KHIYAR);
    await c.waitForDeployment();
  });
  async function fund() {
    await c.connect(bank).fundBank({ value: (V0 * 8000n) / 10000n });
    await c.connect(client).fundClient({ value: (V0 * 2000n) / 10000n });
  }

  it("khiyar al-shart: either partner may rescind within the window; capital is refunded", async function () {
    await fund();
    await expect(c.connect(client).rescindKhiyar())
      .to.emit(c, "Unwound").withArgs((V0 * 8000n) / 10000n, (V0 * 2000n) / 10000n);
    expect(await c.rescinded()).to.equal(true);
    expect(await c.active()).to.equal(false);
  });

  it("khiyar window closes: rescission after the deadline reverts", async function () {
    await fund();
    await time.increase(KHIYAR + 1);
    await expect(c.connect(bank).rescindKhiyar()).to.be.revertedWith("khiyar window closed");
  });

  it("iqalah: completes only with the OTHER partner's consent", async function () {
    await fund();
    await c.connect(bank).proposeIqalah();
    await expect(c.connect(bank).acceptIqalah()).to.be.revertedWith("needs the other partner");
    await expect(c.connect(client).acceptIqalah()).to.emit(c, "IqalahCompleted").withArgs(client.address);
    expect(await c.rescinded()).to.equal(true);
  });

  it("iqalah requires a prior proposal", async function () {
    await fund();
    await expect(c.connect(client).acceptIqalah()).to.be.revertedWith("needs the other partner");
  });

  it("rescission is barred once performance has begun (a buyout occurred)", async function () {
    await fund();
    await c.connect(client).buyShare(1000n, { value: (V0 * 1000n) / 10000n });
    await expect(c.connect(bank).rescindKhiyar()).to.be.revertedWith("performance begun");
    await c.connect(bank).proposeIqalah();
    await expect(c.connect(client).acceptIqalah()).to.be.revertedWith("performance begun");
  });
});
