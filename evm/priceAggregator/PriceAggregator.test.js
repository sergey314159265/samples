const { expect } = require("chai");
const { ethers, upgrades } = require("hardhat");
const helpers = require("@nomicfoundation/hardhat-toolbox/network-helpers");

const { ADMIN_ERROR, MANAGER_ERROR } = require("../common/constanst");
const { time } = require("@nomicfoundation/hardhat-toolbox/network-helpers");

describe("PriceAggregator contract", () => {
  let hhPriceAggregator;
  let owner;
  let bot;
  let addr2;
  let addr3;

  before(async () => {
    const PriceAggregator = await ethers.getContractFactory(
      "contracts/helpers/PriceAggregator.sol:PriceAggregator"
    );
    [owner, bot, addr2, addr3, ...addrs] = await ethers.getSigners();

    hhPriceAggregator = await upgrades.deployProxy(
      PriceAggregator,
      [
        1, // _priceUpdateFee,
        [owner.address], // _allowedSigners_
      ],
      {
        initializer: "initialize",
      }
    );
  });

  describe("Deployment", () => {
    it("Should set the right owner address", async () => {
      await expect(await hhPriceAggregator.owner()).to.equal(owner.address);
    });

    it("Should set the right admin address", async () => {
      await expect(await hhPriceAggregator.adminAddress()).to.equal(
        owner.address
      );
    });

    it("Should set the _paused status", async () => {
      await expect(await hhPriceAggregator.paused()).to.equal(false);
    });
  });

  describe("Transactions", () => {
    it("Should revert when set pause", async () => {
      await expect(hhPriceAggregator.connect(bot).pause()).to.be.revertedWith(
        MANAGER_ERROR
      );
    });

    it("Should set pause", async () => {
      await hhPriceAggregator.pause();

      await expect(await hhPriceAggregator.paused()).to.equal(true);
    });

    it("Should revert when set unpause", async () => {
      await expect(hhPriceAggregator.connect(bot).unpause()).to.be.revertedWith(
        MANAGER_ERROR
      );
    });

    it("Should set unpause", async () => {
      await hhPriceAggregator.unpause();

      await expect(await hhPriceAggregator.paused()).to.equal(false);
    });

    it("Should revert when set the admin address", async () => {
      await expect(
        hhPriceAggregator.connect(bot).setAdminAddress(owner.address)
      ).to.be.revertedWith(ADMIN_ERROR);
    });

    it("Should set the admin address", async () => {
      await hhPriceAggregator.setAdminAddress(owner.address);

      await expect(await hhPriceAggregator.adminAddress()).to.equal(
        owner.address
      );
    });

    it("Should revert when addAllowedSigner", async () => {
      await expect(
        hhPriceAggregator.connect(bot).addAllowedSigner(owner.address)
      ).to.be.revertedWith(ADMIN_ERROR);
    });

    it("Should addAllowedSigner", async () => {
      await hhPriceAggregator.addAllowedSigner(
        "0x0000000000000000000000000000000000000000"
      );
    });

    it("Should revert when removeAllowedSigner", async () => {
      await expect(
        hhPriceAggregator.connect(bot).removeAllowedSigner(owner.address)
      ).to.be.revertedWith(ADMIN_ERROR);
    });

    it("Should removeAllowedSigner", async () => {
      await hhPriceAggregator.removeAllowedSigner(
        "0x0000000000000000000000000000000000000000"
      );
    });

    it("Should revert when setPriceUpdateFee", async () => {
      await expect(
        hhPriceAggregator.connect(bot).setPriceUpdateFee(1)
      ).to.be.revertedWith(ADMIN_ERROR);
    });

    it("Should setPriceUpdateFee", async () => {
      const fee = 2;
      await hhPriceAggregator.setPriceUpdateFee(fee);

      await expect(await hhPriceAggregator.priceUpdateFee()).to.equal(fee);
    });

    it("Should revert when setUpdatePriceLifetime", async () => {
      await expect(
        hhPriceAggregator.connect(bot).setUpdatePriceLifetime(1)
      ).to.be.revertedWith(ADMIN_ERROR);
    });

    it("Should setUpdatePriceLifetime", async () => {
      const lifetime = 10;
      await hhPriceAggregator.setUpdatePriceLifetime(lifetime);

      await expect(await hhPriceAggregator.updatePriceLifetime()).to.equal(
        lifetime
      );
    });

    it("Should revert when updatePriceFeeds - invalid data length", async () => {
      const data = ethers.hexlify(ethers.toUtf8Bytes("Example data"));
      await expect(
        hhPriceAggregator.connect(bot).updatePriceFeeds([data], {
          value: 5,
        })
      ).to.be.revertedWith("PriceAggregator: invalid data length");
    });

    it("Should revert when updatePriceFeeds - invalid signature", async () => {
      const id =
        "0x49f6b65cb1de6b10eaf75e7c03ca029c306d0357e91b5311b175084a5ad55688";
      const price = 1000;
      const conf = 500;
      const expo = -2;
      const publishTime = 123456;

      const AbiCoder = new ethers.AbiCoder();
      const updatePriceInfo = AbiCoder.encode(
        ["bytes32", "int64", "uint64", "int32", "uint64"],
        [id, price, conf, expo, publishTime]
      );
      const messageHash = ethers.keccak256(updatePriceInfo);
      const signature = await bot.signMessage(ethers.getBytes(messageHash));
      const signedData = ethers.concat([signature, updatePriceInfo]);
      await expect(
        hhPriceAggregator.connect(bot).updatePriceFeeds([signedData], {
          value: 5,
        })
      ).to.be.revertedWith("PriceAggregator: Invalid signature");
    });

    it("Should updatePriceFeeds", async () => {
      const currentTime = await time.latest();
      const id =
        "0x49f6b65cb1de6b10eaf75e7c03ca029c306d0357e91b5311b175084a5ad55688";
      const price = 1000;
      const conf = 500;
      const expo = -2;
      const publishTime = currentTime;

      const AbiCoder = new ethers.AbiCoder();
      const updatePriceInfo = AbiCoder.encode(
        ["bytes32", "int64", "uint64", "int32", "uint64"],
        [id, price, conf, expo, publishTime]
      );
      const messageHash = ethers.keccak256(updatePriceInfo);
      const signature = await owner.signMessage(ethers.getBytes(messageHash));
      const signedData = ethers.concat([signature, updatePriceInfo]);
      await hhPriceAggregator
        .connect(bot)
        .updatePriceFeeds([signedData], { value: 5 });

      const priceInfo = await hhPriceAggregator.getPrice(id);

      await expect(priceInfo.price).to.equal(price);
      await expect(priceInfo.conf).to.equal(conf);
      await expect(priceInfo.expo).to.equal(expo);
      await expect(priceInfo.publishTime).to.equal(publishTime);
    });

    it("Should revert when updatePriceFeeds - StalePrice", async () => {
      const currentTime = await time.latest();
      const id =
        "0x49f6b65cb1de6b10eaf75e7c03ca029c306d0357e91b5311b175084a5ad55688";
      const price = 1000;
      const conf = 500;
      const expo = -2;
      const publishTime = currentTime - 15;

      const AbiCoder = new ethers.AbiCoder();
      const updatePriceInfo = AbiCoder.encode(
        ["bytes32", "int64", "uint64", "int32", "uint64"],
        [id, price, conf, expo, publishTime]
      );
      const messageHash = ethers.keccak256(updatePriceInfo);
      const signature = await owner.signMessage(ethers.getBytes(messageHash));
      const signedData = ethers.concat([signature, updatePriceInfo]);

      await expect(
        hhPriceAggregator.connect(bot).updatePriceFeeds([signedData], {
          value: 5,
        })
      ).to.be.revertedWithCustomError(hhPriceAggregator, "StalePrice()");
    });
  });
});
