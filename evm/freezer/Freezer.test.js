const { expect } = require("chai");
const { ethers, upgrades } = require("hardhat");
const helpers = require("@nomicfoundation/hardhat-toolbox/network-helpers");
const { ADMIN_ERROR, MANAGER_ERROR } = require("./common/constanst");
const {
    deployTokenFixture,
    deployInfinityPassFixture,
    deployTermsAndConditionsFixture,
} = require("./common/mocks");
const { time, mine } = require("@nomicfoundation/hardhat-network-helpers");

describe("Freezer contract", () => {
    let hhFreezer;
    let owner;
    let addr1;
    let addr2;
    let addr3;
    let bot;
    let vesting;
    let rewardsDistributor;
    let erc20Token;
    let infinityPass;
    let termsAndConditions;

    before(async () => {
        const Freezer = await ethers.getContractFactory("contracts/Freezer.sol:Freezer");
        [owner, addr1, addr2, addr3, bot, vesting, rewardsDistributor, ...addrs] =
            await ethers.getSigners();
        const nonZeroAddress = ethers.Wallet.createRandom().address;
        erc20Token = await helpers.loadFixture(deployTokenFixture);
        infinityPass = await helpers.loadFixture(deployInfinityPassFixture);
        termsAndConditions = await helpers.loadFixture(deployTermsAndConditionsFixture);
        const infinityPassPercent = 5;

        hhFreezer = await upgrades.deployProxy(
            Freezer,
            [
                vesting.address,
                infinityPassPercent,
                infinityPass.target,
                nonZeroAddress,
                termsAndConditions.target,
            ],

            {
                initializer: "initialize",
            },
        );
    });

    describe("Deployment", () => {
        it("Should set the right owner address", async () => {
            await expect(await hhFreezer.owner()).to.equal(owner.address);
        });

        it("Should set the right admin address", async () => {
            await expect(await hhFreezer.adminAddress()).to.equal(owner.address);
        });

        it("Should set the right vesting address", async () => {
            await expect(await hhFreezer.vestingAddress()).to.equal(vesting.address);
        });

        it("Should set the _paused status", async () => {
            await expect(await hhFreezer.paused()).to.equal(false);
        });
    });

    describe("Transactions", () => {
        it("Should revert when set pause", async () => {
            await expect(hhFreezer.connect(addr1).pause()).to.be.revertedWith(MANAGER_ERROR);
        });

        it("Should set pause", async () => {
            await hhFreezer.pause();

            await expect(await hhFreezer.paused()).to.equal(true);
        });

        it("Should revert when set unpause", async () => {
            await expect(hhFreezer.connect(addr1).unpause()).to.be.revertedWith(MANAGER_ERROR);
        });

        it("Should set unpause", async () => {
            await hhFreezer.unpause();

            await expect(await hhFreezer.paused()).to.equal(false);
        });

        it("Should revert when set the admin address", async () => {
            await expect(
                hhFreezer.connect(addr1).setAdminAddress(owner.address),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should set the admin address", async () => {
            await hhFreezer.setAdminAddress(owner.address);

            await expect(await hhFreezer.adminAddress()).to.equal(owner.address);
        });

        it("Should revert when setVestingAddress", async () => {
            await expect(
                hhFreezer.connect(addr1).setVestingAddress(owner.address),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should setVestingAddress", async () => {
            await hhFreezer.setVestingAddress(vesting.address);

            await expect(await hhFreezer.vestingAddress()).to.equal(vesting.address);
        });

        it("Should revert when setRewardsDistributorAddress", async () => {
            await expect(
                hhFreezer.connect(addr1).setRewardsDistributorAddress(owner.address),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should setRewardsDistributorAddress", async () => {
            await hhFreezer.setRewardsDistributorAddress(rewardsDistributor.address);

            await expect(await hhFreezer.rewardsDistributorAddress()).to.equal(
                rewardsDistributor.address,
            );
        });

        it("Should revert when setAverageBlockTime", async () => {
            await expect(hhFreezer.connect(addr1).setAverageBlockTime(8)).to.be.revertedWith(
                ADMIN_ERROR,
            );
        });

        it("Should setAverageBlockTime", async () => {
            await hhFreezer.setAverageBlockTime(8);

            await expect(await hhFreezer.averageBlockTime()).to.equal(8);
        });

        it("Should revert when addPool", async () => {
            const poolFee = {
                depositFee: 0.5 * 1e4,
                withdrawFee: 0.5 * 1e4,
                claimFee: 0.5 * 1e4,
            };
            await expect(
                hhFreezer
                    .connect(addr1)
                    .addPool(erc20Token.target, erc20Token.target, 1, 1, poolFee),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should addPool", async () => {
            const fee = {
                depositFee: 1 * 1e4,
                withdrawFee: 1 * 1e4,
                claimFee: 1 * 1e4,
            };
            const baseToken = erc20Token.target;
            const lastRewardBlock = await ethers.provider.getBlockNumber();
            const accRewardPerShare = ethers.parseEther("0.01");

            await hhFreezer.addPool(
                baseToken,
                baseToken,
                lastRewardBlock,
                accRewardPerShare,
                fee,
            );

            await expect(await hhFreezer.getPoolLength()).to.be.equal(1);

            const poolInfo = await hhFreezer.poolInfo(0);
            const poolFee = await hhFreezer.poolFee(0);

            await expect(poolInfo[0]).to.be.equal(baseToken);
            await expect(poolInfo[1]).to.be.equal(baseToken);
            await expect(poolInfo[2]).to.be.equal(0);
            await expect(poolInfo[3]).to.be.equal(lastRewardBlock);
            await expect(poolInfo[4]).to.be.equal(accRewardPerShare);
            await expect(poolFee.depositFee).to.be.equal(1 * 1e4);
            await expect(poolFee.withdrawFee).to.be.equal(1 * 1e4);
            await expect(poolFee.claimFee).to.be.equal(1 * 1e4);
        });

        it("Should addPool with lastRewardBlock > blockNumber", async () => {
            const fee = {
                depositFee: 1 * 1e4,
                withdrawFee: 1 * 1e4,
                claimFee: 1 * 1e4,
            };
            const baseToken = erc20Token.target;
            const lastRewardBlock = (await ethers.provider.getBlockNumber()) + 500;
            const accRewardPerShare = ethers.parseEther("0.01");

            await hhFreezer.addPool(
                baseToken,
                baseToken,
                lastRewardBlock,
                accRewardPerShare,
                fee,
            );

            await expect(await hhFreezer.getPoolLength()).to.be.equal(2);

            const poolInfo = await hhFreezer.poolInfo(1);
            const poolFee = await hhFreezer.poolFee(0);

            await expect(poolInfo[0]).to.be.equal(baseToken);
            await expect(poolInfo[1]).to.be.equal(baseToken);
            await expect(poolInfo[2]).to.be.equal(0);
            await expect(poolInfo[3]).to.be.equal(lastRewardBlock);
            await expect(poolInfo[4]).to.be.equal(accRewardPerShare);
            await expect(poolFee.depositFee).to.be.equal(1 * 1e4);
            await expect(poolFee.withdrawFee).to.be.equal(1 * 1e4);
            await expect(poolFee.claimFee).to.be.equal(1 * 1e4);
        });

        it("Should revert when setRewardConfiguration", async () => {
            await expect(
                hhFreezer.connect(addr1).setRewardConfiguration(0, 1, 1),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should setRewardConfiguration", async () => {
            const rewardPerBlock = ethers.parseEther("0.5");
            const updateBlocksInterval = 12345;

            await hhFreezer.setRewardConfiguration(0, rewardPerBlock, updateBlocksInterval);

            const lastBlock = await ethers.provider.getBlockNumber();
            const rewardsConfiguration = await hhFreezer.getRewardsConfiguration(0);

            await expect(rewardsConfiguration.rewardPerBlock).to.be.equal(rewardPerBlock);
            await expect(rewardsConfiguration.lastUpdateBlockNum).to.be.equal(lastBlock);
            await expect(rewardsConfiguration.updateBlocksInterval).to.be.equal(
                updateBlocksInterval,
            );
        });

        it("Should revert when setPoolInfo - admin error", async () => {
            await expect(hhFreezer.connect(addr1).setPoolInfo(0, 1, 1)).to.be.revertedWith(
                ADMIN_ERROR,
            );
        });

        it("Should revert when setPoolInfo - WrongPool", async () => {
            await expect(hhFreezer.setPoolInfo(5, 1, 1)).to.be.revertedWithCustomError(
                hhFreezer,
                "WrongPool",
            );
        });

        it("Should setPoolInfo", async () => {
            const _pid = 0;
            const lastRewardBlock = await ethers.provider.getBlockNumber();
            const accRewardPerShare = ethers.parseEther("0.02");

            await hhFreezer.setPoolInfo(_pid, lastRewardBlock, accRewardPerShare);

            const poolInfo = await hhFreezer.poolInfo(_pid);

            await expect(poolInfo[3]).to.be.equal(lastRewardBlock);
            await expect(poolInfo[4]).to.be.equal(accRewardPerShare);
        });

        it("Should revert when setPoolFee - admin error", async () => {
            const poolFee = {
                depositFee: 0.5 * 1e4,
                withdrawFee: 0.5 * 1e4,
                claimFee: 0.5 * 1e4,
            };
            await expect(hhFreezer.connect(addr1).setPoolFee(0, poolFee)).to.be.revertedWith(
                ADMIN_ERROR,
            );
        });

        it("Should revert when setPoolFee - WrongPool", async () => {
            const poolFee = {
                depositFee: 0.5 * 1e4,
                withdrawFee: 0.5 * 1e4,
                claimFee: 0.5 * 1e4,
            };
            await expect(hhFreezer.setPoolFee(5, poolFee)).to.be.revertedWithCustomError(
                hhFreezer,
                "WrongPool",
            );
        });

        it("Should setPoolFee", async () => {
            const _pid = 0;
            const poolFee = {
                depositFee: 5,
                withdrawFee: 5,
                claimFee: 5,
            };

            await hhFreezer.setPoolFee(_pid, poolFee);

            const poolFeeInfo = await hhFreezer.poolFee(_pid);

            await expect(poolFeeInfo[0]).to.be.equal(5);
            await expect(poolFeeInfo[1]).to.be.equal(5);
            await expect(poolFeeInfo[2]).to.be.equal(5);
        });

        it("Should revert when setLockPeriod - admin error", async () => {
            await expect(hhFreezer.connect(addr1).setLockPeriod(1, 1)).to.be.revertedWith(
                ADMIN_ERROR,
            );
        });

        it("Should setLockPeriod ", async () => {
            const id_0 = 0;
            const id_1 = 1;
            const id_2 = 2;
            const durations_0 = 60; //1 min
            const durations_1 = 120; //2 min
            const durations_2 = 600; //10 min

            await hhFreezer.setLockPeriod(id_0, durations_0);
            await hhFreezer.setLockPeriod(id_1, durations_1);
            await hhFreezer.setLockPeriod(id_2, durations_2);

            await expect(await hhFreezer.lockPeriod(id_0)).to.be.equal(durations_0);
            await expect(await hhFreezer.lockPeriod(id_1)).to.be.equal(durations_1);
            await expect(await hhFreezer.lockPeriod(id_2)).to.be.equal(durations_2);
        });

        it("Should revert when setLockPeriodMultiplier - admin error", async () => {
            await expect(
                hhFreezer.connect(addr1).setLockPeriodMultiplier(1, 1),
            ).to.be.revertedWith(ADMIN_ERROR);
        });

        it("Should setLockPeriodMultiplier ", async () => {
            const id = 0;
            const multiplier = 1e5; //1.00000

            await hhFreezer.setLockPeriodMultiplier(id, multiplier);

            await expect(await hhFreezer.lockPeriodMultiplier(id)).to.be.equal(multiplier);
        });

        it("Should revert when deposit - pause", async () => {
            await hhFreezer.pause();

            await expect(
                hhFreezer.connect(addr1).deposit(1, 1, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "EnforcedPause");
            await hhFreezer.unpause();
        });

        it("Should revert when deposit - WrongPool", async () => {
            await expect(
                hhFreezer.connect(addr1).deposit(2, 1, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "WrongPool");
        });

        it("Should revert when deposit - invalid lock id", async () => {
            await expect(
                hhFreezer.connect(addr1).deposit(0, 3, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "WrongLockPeriod");
        });

        it("Should revert when deposit - invalid lock id", async () => {
            await expect(
                hhFreezer.connect(addr1).deposit(0, 5, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "WrongLockPeriod");
        });

        it("Should revert when deposit - OnlyAgreedToTerms", async () => {
            await expect(
                hhFreezer.connect(addr1).deposit(0, 0, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "OnlyAgreedToTerms");

            await termsAndConditions.connect(addr1).agreeToTerms();
        });

        it("Should revert when deposit - invalid balance", async () => {
            await expect(
                hhFreezer.connect(addr1).deposit(0, 0, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "InvalidAmount");
        });

        it("Should revert when depositVesting - pause", async () => {
            await hhFreezer.pause();

            await expect(
                hhFreezer.connect(addr1).depositVesting(addr1.address, 1, 1, 1, 1, 5),
            ).to.be.revertedWithCustomError(hhFreezer, "EnforcedPause");
            await hhFreezer.unpause();
        });

        it("Should revert when depositVesting - WrongPool", async () => {
            await expect(
                hhFreezer.connect(addr1).depositVesting(addr1.address, 2, 1, 1, 1, 5),
            ).to.be.revertedWithCustomError(hhFreezer, "WrongPool");
        });

        it("Should revert when depositVesting - only vesting", async () => {
            await expect(
                hhFreezer.connect(addr1).depositVesting(addr1.address, 0, 2, 1, 1, 5),
            ).to.be.revertedWithCustomError(hhFreezer, "NotAllowed");
        });

        it("Should depositVesting", async () => {
            const pid = 0;
            const amount = ethers.parseEther("10");
            const lockId = 5;
            const depositTimestamp = await time.latest();
            const withdrawalTimestamp = depositTimestamp + 100;

            await erc20Token.mint(hhFreezer.target, amount);

            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);

            await hhFreezer
                .connect(vesting)
                .depositVesting(
                    addr1.address,
                    pid,
                    amount,
                    depositTimestamp,
                    withdrawalTimestamp,
                    lockId,
                );

            const poolInfo = await hhFreezer.poolInfo(pid);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, 0);
            const blockNumber = await ethers.provider.getBlockNumber();

            const rewardDebt = (amount * poolInfo[4]) / ethers.parseEther("1");

            await expect(poolInfo[0]).to.be.equal(poolInfoBefore[0]);
            await expect(poolInfo[1]).to.be.equal(poolInfoBefore[1]);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] + amount);
            await expect(poolInfo[3]).to.be.equal(blockNumber);
            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] + amount);
            await expect(userInfo[1]).to.be.equal(userInfoBefore[1] + BigInt(1));
            await expect(userInfo[2]).to.be.equal(userInfoBefore[2]);
            await expect(userDeposit[0]).to.be.equal(amount);
            await expect(userDeposit[1]).to.be.equal(lockId);
            await expect(userDeposit[2]).to.be.equal(depositTimestamp);
            await expect(userDeposit[3]).to.be.equal(withdrawalTimestamp);
            await expect(userDeposit[4]).to.be.equal(0);
            await expect(userDeposit[5]).to.be.equal(rewardDebt);
            await expect(userDeposit[6]).to.be.equal(false);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(tvlBefore + amount);
        });

        it("Should deposit", async () => {
            const pid = 0;
            const lockId = 0;
            const amount = ethers.parseEther("1");
            const lockPeriod = await hhFreezer.lockPeriod(lockId);
            const poolFee = await hhFreezer.poolFee(pid);

            await erc20Token.mint(addr1.address, amount);
            await erc20Token.connect(addr1).approve(hhFreezer.target, amount);

            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);
            const burnAmount = (amount * poolFee.depositFee) / BigInt(1e4);

            await hhFreezer.connect(addr1).deposit(pid, lockId, amount);

            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfo = await hhFreezer.poolInfo(pid);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, 1);
            const blockNumber = await ethers.provider.getBlockNumber();
            const blockTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

            const rewardDebt = ((amount - burnAmount) * poolInfo[4]) / ethers.parseEther("1");

            await expect(contractBalance).to.be.equal(contractBalanceBefore + amount - burnAmount);
            await expect(poolInfo[0]).to.be.equal(poolInfoBefore[0]);
            await expect(poolInfo[1]).to.be.equal(poolInfoBefore[1]);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] + amount - burnAmount);
            await expect(poolInfo[3]).to.be.equal(blockNumber);
            // await expect(poolInfo[4]).to.be.equal(poolInfoBefore[4]);
            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] + amount - burnAmount);
            await expect(userInfo[1]).to.be.equal(userInfoBefore[1] + BigInt(1));
            await expect(userInfo[2]).to.be.equal(userInfoBefore[2]);
            await expect(userDeposit[0]).to.be.equal(amount - burnAmount);
            await expect(userDeposit[1]).to.be.equal(lockId);
            await expect(userDeposit[2]).to.be.equal(blockTimestamp);
            await expect(userDeposit[3]).to.be.equal(BigInt(blockTimestamp) + lockPeriod);
            await expect(userDeposit[4]).to.be.equal(0);
            await expect(userDeposit[5]).to.be.equal(rewardDebt);
            await expect(userDeposit[6]).to.be.equal(false);
            await expect(await hhFreezer.pendingReward(pid, 0, addr1.address)).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore + amount - burnAmount,
            );
        });

        it("Should deposit - 2", async () => {
            const pid = 0;
            const lockId = 0;
            const amount = ethers.parseEther("5");
            const lockPeriod = await hhFreezer.lockPeriod(lockId);
            const poolFee = await hhFreezer.poolFee(pid);

            await erc20Token.mint(addr1.address, amount);
            await erc20Token.connect(addr1).approve(hhFreezer.target, amount);

            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);
            const burnAmount = (amount * poolFee.depositFee) / BigInt(1e4);

            await hhFreezer.connect(addr1).deposit(pid, lockId, amount);

            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfo = await hhFreezer.poolInfo(pid);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, 2);
            const blockNumber = await ethers.provider.getBlockNumber();
            const blockTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

            const rewardDebt = ((amount - burnAmount) * poolInfo[4]) / ethers.parseEther("1");
            const accRewardPerShare = await expect(contractBalance).to.be.equal(
                contractBalanceBefore + amount - burnAmount,
            );
            await expect(poolInfo[0]).to.be.equal(poolInfoBefore[0]);
            await expect(poolInfo[1]).to.be.equal(poolInfoBefore[1]);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] + amount - burnAmount);
            await expect(poolInfo[3]).to.be.equal(blockNumber);
            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] + amount - burnAmount);
            await expect(userInfo[1]).to.be.equal(userInfoBefore[1] + BigInt(1));
            await expect(userInfo[2]).to.be.equal(userInfoBefore[2]);
            await expect(userDeposit[0]).to.be.equal(amount - burnAmount);
            await expect(userDeposit[1]).to.be.equal(lockId);
            await expect(userDeposit[2]).to.be.equal(blockTimestamp);
            await expect(userDeposit[3]).to.be.equal(BigInt(blockTimestamp) + lockPeriod);
            await expect(userDeposit[4]).to.be.equal(0);
            await expect(userDeposit[5]).to.be.equal(rewardDebt);
            await expect(userDeposit[6]).to.be.equal(false);
            await expect(await hhFreezer.pendingReward(pid, 2, addr1.address)).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore + amount - burnAmount,
            );
        });

        it("Should deposit - 3, with diff lockId", async () => {
            const pid = 0;
            const lockId = 1;
            const amount = ethers.parseEther("5");
            const lockPeriod = await hhFreezer.lockPeriod(lockId);
            const poolFee = await hhFreezer.poolFee(pid);

            await erc20Token.mint(addr1.address, amount);
            await erc20Token.connect(addr1).approve(hhFreezer.target, amount);

            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);
            const burnAmount = (amount * poolFee.depositFee) / BigInt(1e4);

            await hhFreezer.connect(addr1).deposit(pid, lockId, amount);

            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const poolInfo = await hhFreezer.poolInfo(pid);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, 3);
            const blockNumber = await ethers.provider.getBlockNumber();
            const blockTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

            const rewardDebt = ((amount - burnAmount) * poolInfo[4]) / ethers.parseEther("1");
            const accRewardPerShare = await expect(contractBalance).to.be.equal(
                contractBalanceBefore + amount - burnAmount,
            );
            await expect(poolInfo[0]).to.be.equal(poolInfoBefore[0]);
            await expect(poolInfo[1]).to.be.equal(poolInfoBefore[1]);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] + amount - burnAmount);
            await expect(poolInfo[3]).to.be.equal(blockNumber);
            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] + amount - burnAmount);
            await expect(userInfo[1]).to.be.equal(userInfoBefore[1] + BigInt(1));
            await expect(userInfo[2]).to.be.equal(userInfoBefore[2]);
            await expect(userDeposit[0]).to.be.equal(amount - burnAmount);
            await expect(userDeposit[1]).to.be.equal(lockId);
            await expect(userDeposit[2]).to.be.equal(blockTimestamp);
            await expect(userDeposit[3]).to.be.equal(BigInt(blockTimestamp) + lockPeriod);
            await expect(userDeposit[4]).to.be.equal(0);
            await expect(userDeposit[5]).to.be.equal(rewardDebt);
            await expect(userDeposit[6]).to.be.equal(false);
            await expect(await hhFreezer.pendingReward(pid, 3, addr1.address)).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore + amount - burnAmount,
            );
        });

        it("Should get pendingReward = 0, block = lastRewardBlock", async () => {
            const pid = 0;
            const depositId = 2;

            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);

            const pendingReward =
                (userDeposit[0] * poolInfo[4]) / ethers.parseEther("1") - userDeposit[5];

            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(pendingReward);
        });

        it("Should get pendingReward, block > lastRewardBlock", async () => {
            const pid = 0;
            const depositId = 1;
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);

            await helpers.mine(10);
            const blockNumber = await ethers.provider.getBlockNumber();

            const multiplier = BigInt(blockNumber) - poolInfo[3];
            const rewardPerBlock = await hhFreezer.getRewardPerBlock(0);
            const reward = multiplier * rewardPerBlock;
            const accRewardPerShare = poolInfo[4] + (reward * ethers.parseEther("1")) / poolInfo[2];
            const pendingReward =
                (userDeposit[0] * accRewardPerShare) / ethers.parseEther("1") - userDeposit[5];

            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(pendingReward);
        });

        it("Should get pendingReward, block > lastRewardBlock with lock period multiplier", async () => {
            const pid = 0;
            const depositId = 2;
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);
            const lockMultiplier = BigInt(100005); //1.00005
            await hhFreezer.setLockPeriodMultiplier(0, lockMultiplier);

            await helpers.mine(10);
            const blockNumber = await ethers.provider.getBlockNumber();

            const multiplier = BigInt(blockNumber) - poolInfo[3];
            const rewardPerBlock = await hhFreezer.getRewardPerBlock(0);
            const reward = multiplier * rewardPerBlock;
            const accRewardPerShare = poolInfo[4] + (reward * ethers.parseEther("1")) / poolInfo[2];
            let pendingReward =
                (userDeposit[0] * accRewardPerShare) / ethers.parseEther("1") - userDeposit[5];

            pendingReward = (pendingReward * lockMultiplier) / BigInt(1e5);

            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(pendingReward);

            await hhFreezer.setLockPeriodMultiplier(0, 1e5);
        });

        it("Should get pendingRewardTotal - 4 deposit", async () => {
            await mine(5);
            const rewards1 = await hhFreezer.pendingReward(0, 0, addr1.address);
            const rewards2 = await hhFreezer.pendingReward(0, 1, addr1.address);
            const rewards3 = await hhFreezer.pendingReward(0, 2, addr1.address);
            const rewards4 = await hhFreezer.pendingReward(0, 3, addr1.address);
            await expect(await hhFreezer.pendingRewardTotal(0, addr1.address)).to.be.equal(
                rewards1 + rewards2 + rewards3 + rewards4,
            );
        });

        it("Should get getUserLastDepositId", async () => {
            await expect(await hhFreezer.getUserLastDepositId(0, addr1.address)).to.be.equal(3);
        });

        it("Should revert when claim - WrongPool", async () => {
            await expect(hhFreezer.connect(addr1).claim(2, 1)).to.be.revertedWithCustomError(
                hhFreezer,
                "WrongPool",
            );
        });

        it("Should claim", async () => {
            const pid = 0;
            const depositId = 0;
            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );

            await hhFreezer.connect(addr1).claim(pid, depositId);
            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const pendingRewards = userInfo[2] - userInfoBefore[2];

            await expect(userInfo[2]).to.be.equal(userInfoBefore[2] + pendingRewards);
            await expect(userDeposit[4]).to.be.equal(userDepositBefore[4] + pendingRewards);
            await expect(userBalance).to.be.equal(userBalanceBefore + pendingRewards);
            await expect(contractBalance).to.be.equal(contractBalanceBefore - pendingRewards);
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
        });

        it("Should claimAll", async () => {
            const pid = 0;

            await erc20Token.mint(hhFreezer.target, ethers.parseEther("10"));
            await helpers.mine(10);

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);

            await hhFreezer.connect(addr1).claimAll(pid);

            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);

            const pendingRewards = userInfo[2] - userInfoBefore[2];
            const claimFee =
                contractBalanceBefore -
                pendingRewards -
                (await erc20Token.balanceOf(hhFreezer.target));

            await expect(userInfo[2]).to.be.equal(userInfoBefore[2] + pendingRewards);
            await expect(userBalance).to.be.equal(userBalanceBefore + pendingRewards);
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - pendingRewards - claimFee,
            );
            await expect(await hhFreezer.pendingRewardTotal(pid, addr1.address)).to.be.equal(0);
        });

        it("Should revert when withdraw - WrongPool", async () => {
            await expect(hhFreezer.connect(addr1).withdraw(2, 0)).to.be.revertedWithCustomError(
                hhFreezer,
                "WrongPool",
            );
        });

        it("Should revert when withdraw - withdrawalTimestamp > block.timestamp", async () => {
            await expect(hhFreezer.connect(addr1).withdraw(0, 2)).to.be.revertedWithCustomError(
                hhFreezer,
                "PeriodNotEnded",
            );
        });

        it("Should withdraw ", async () => {
            const pid = 0;
            const depositId = 1;
            const lockId = 0;
            const poolFee = await hhFreezer.poolFee(pid);
            await erc20Token.mint(hhFreezer.target, ethers.parseEther("10"));
            await helpers.mine(50);

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const depositTokens = userDepositBefore[0];
            const tvlBefore = await hhFreezer.tvl(pid, lockId);

            await hhFreezer.connect(addr1).withdraw(pid, depositId);
            const burnAmount = (depositTokens * poolFee.withdrawFee) / BigInt(1e4);

            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);

            const claimAmount = userDeposit[4] - userDepositBefore[4];

            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] - depositTokens);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] - depositTokens);
            await expect(userDeposit[6]).to.be.equal(true);
            await expect(userBalance).to.be.equal(
                userBalanceBefore + depositTokens + claimAmount - burnAmount,
            );
            await expect(contractBalance).to.be.below(
                contractBalanceBefore - depositTokens - claimAmount - burnAmount,
            );
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore - depositTokens,
            );
        });

        it("Should revert when withdrawVesting - WrongPool", async () => {
            await expect(
                hhFreezer
                    .connect(vesting)
                    .withdrawVesting(addr1.address, 2, 0, ethers.parseEther("1")),
            ).to.be.revertedWithCustomError(hhFreezer, "WrongPool");
        });

        it("Should revert when withdrawVesting - invalid withdraw amount", async () => {
            await expect(
                hhFreezer
                    .connect(vesting)
                    .withdrawVesting(addr1.address, 0, 1, ethers.parseEther("100")),
            ).to.be.revertedWithCustomError(hhFreezer, "InvalidAmount");
        });

        it("Should withdrawVesting ", async () => {
            const pid = 0;
            const depositId = 0;
            const lockId = 5;
            await erc20Token.mint(hhFreezer.target, ethers.parseEther("10"));
            await helpers.mine(50);
            const withdrawAmount = ethers.parseEther("5");

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);

            await hhFreezer
                .connect(addr1)
                .connect(vesting)
                .withdrawVesting(addr1.address, pid, depositId, withdrawAmount);

            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);

            const claimAmount = userDeposit[4] - userDepositBefore[4];

            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] - withdrawAmount);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] - withdrawAmount);
            await expect(userDeposit[6]).to.be.equal(false);
            await expect(userBalance).to.be.equal(userBalanceBefore + withdrawAmount + claimAmount);
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - withdrawAmount - claimAmount,
            );
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore - withdrawAmount,
            );
        });

        it("Should withdrawVesting full amount", async () => {
            const pid = 0;
            const depositId = 0;
            const lockId = 5;
            await erc20Token.mint(hhFreezer.target, ethers.parseEther("10"));
            await helpers.mine(50);
            const withdrawAmount = ethers.parseEther("5");

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );
            const poolInfoBefore = await hhFreezer.poolInfo(pid);
            const tvlBefore = await hhFreezer.tvl(pid, lockId);

            await hhFreezer
                .connect(addr1)
                .connect(vesting)
                .withdrawVesting(addr1.address, pid, depositId, withdrawAmount);

            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const poolInfo = await hhFreezer.poolInfo(pid);

            const claimAmount = userDeposit[4] - userDepositBefore[4];

            await expect(userInfo[0]).to.be.equal(userInfoBefore[0] - withdrawAmount);
            await expect(poolInfo[2]).to.be.equal(poolInfoBefore[2] - withdrawAmount);
            await expect(userDeposit[6]).to.be.equal(true);
            await expect(userBalance).to.be.equal(userBalanceBefore + withdrawAmount + claimAmount);
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - withdrawAmount - claimAmount,
            );
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(
                tvlBefore - withdrawAmount,
            );
        });

        it("Should get pendingReward with block.timestamp <= pool.lastRewardBlock", async () => {
            const pid = 0;
            const amount = ethers.parseEther("10");
            const lockId = 5;
            const depositTimestamp = await time.latest();
            const withdrawalTimestamp = depositTimestamp + 50;
            const accRewardPerShare = (await hhFreezer.poolInfo(pid))[4];

            await erc20Token.mint(hhFreezer.target, amount);

            const block = (await ethers.provider.getBlockNumber()) + 500;
            const tvlBefore = await hhFreezer.tvl(pid, lockId);

            await hhFreezer
                .connect(vesting)
                .depositVesting(
                    addr1.address,
                    pid,
                    amount,
                    depositTimestamp,
                    withdrawalTimestamp,
                    lockId,
                );

            await hhFreezer.setPoolInfo(0, block, 0);

            await expect(await hhFreezer.pendingRewardTotal(0, addr1.address)).to.be.equal(0);

            await hhFreezer.setPoolInfo(
                0,
                await ethers.provider.getBlockNumber(),
                accRewardPerShare,
            );
            await expect(await hhFreezer.tvl(pid, lockId)).to.be.equal(tvlBefore + amount);
        });

        it("Should get pendingReward after claim and some blocks, auto-withdraw", async () => {
            const pid = 0;
            const depositId = 2;

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const fee = await hhFreezer.poolFee(pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );

            await hhFreezer.connect(addr1).claim(pid, depositId);
            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const pendingRewards = userInfo[2] - userInfoBefore[2];
            const burnFee = contractBalanceBefore - contractBalance - pendingRewards;
            const burnDepositFee = (userDeposit.depositTokens * fee.withdrawFee) / BigInt(1e4);

            await expect(userInfo[2]).to.be.equal(userInfoBefore[2] + pendingRewards);
            await expect(userDeposit[4]).to.be.equal(userDepositBefore[4] + pendingRewards);
            await expect(userBalance).to.be.equal(
                userBalanceBefore + pendingRewards + userDeposit.depositTokens - burnDepositFee,
            );
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - pendingRewards - burnFee,
            );
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
        });

        it("Should revert when addRewards", async () => {
            await expect(
                hhFreezer.connect(addr1).addRewards(1, 1),
            ).to.be.revertedWithCustomError(hhFreezer, "NotAllowed");
        });

        it("Should addRewards", async () => {
            const pid = 0;
            const poolInfo = await hhFreezer.poolInfo(pid);
            const totalShares = poolInfo[2];
            const amount = totalShares * BigInt(2);

            await erc20Token.mint(hhFreezer.target, amount);

            await hhFreezer.connect(rewardsDistributor).addRewards(pid, amount);

            const productsRewardsInfo = await hhFreezer.productsRewardsInfo(pid);

            await expect(productsRewardsInfo[0]).to.be.equal(amount);
            await expect(productsRewardsInfo[1]).to.be.equal(
                (amount * ethers.parseEther("1")) / totalShares,
            );
        });

        it("Should get pendingReward after addRewards and claim, skip some blocks", async () => {
            const pid = 0;
            const depositId = 2;

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);
            const userDepositBefore = await hhFreezer.userDeposits(
                addr1.address,
                pid,
                depositId,
            );

            await hhFreezer.connect(addr1).claim(pid, depositId);
            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);
            const pendingRewards = userInfo[2] - userInfoBefore[2];
            const burnFee = contractBalanceBefore - contractBalance - pendingRewards;

            await expect(userInfo[2]).to.be.equal(userInfoBefore[2] + pendingRewards);
            await expect(userDeposit[4]).to.be.equal(userDepositBefore[4] + pendingRewards);
            await expect(userBalance).to.be.equal(userBalanceBefore + pendingRewards);
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - pendingRewards - burnFee,
            );
            await expect(
                await hhFreezer.pendingReward(pid, depositId, addr1.address),
            ).to.be.equal(0);
        });

        it("Should get pendingRewardByLockId - 3 deposit", async () => {
            const rewards1 = await hhFreezer.pendingReward(0, 0, addr1.address);
            const rewards2 = await hhFreezer.pendingReward(0, 1, addr1.address);
            const rewards3 = await hhFreezer.pendingReward(0, 2, addr1.address);
            const rewards4 = await hhFreezer.pendingReward(0, 3, addr1.address);
            await expect(await hhFreezer.pendingRewardByLockId(0, 0, addr1.address)).to.be.equal(
                rewards1 + rewards2 + rewards3,
            );
            await expect(await hhFreezer.pendingRewardByLockId(0, 1, addr1.address)).to.be.equal(
                rewards4,
            );
        });

        it("Should claimAllByByLockId", async () => {
            const pid = 0;
            const lockId = 0;

            await erc20Token.mint(hhFreezer.target, ethers.parseEther("10"));
            await helpers.mine(10);

            const userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            const contractBalanceBefore = await erc20Token.balanceOf(hhFreezer.target);
            const userInfoBefore = await hhFreezer.userInfo(addr1.address, pid);

            await hhFreezer.connect(addr1).claimAllByByLockId(pid, lockId);

            const userBalance = await erc20Token.balanceOf(addr1.address);
            const contractBalance = await erc20Token.balanceOf(hhFreezer.target);
            const userInfo = await hhFreezer.userInfo(addr1.address, pid);

            const pendingRewards = userInfo[2] - userInfoBefore[2];
            const burnFee = contractBalanceBefore - contractBalance - pendingRewards;

            await expect(userInfo[2]).to.be.equal(userInfoBefore[2] + pendingRewards);
            await expect(userBalance).to.be.equal(userBalanceBefore + pendingRewards);
            await expect(contractBalance).to.be.equal(
                contractBalanceBefore - pendingRewards - burnFee,
            );

            await expect(
                await hhFreezer.pendingRewardByLockId(0, lockId, addr1.address),
            ).to.be.equal(0);

            const rewards4 = await hhFreezer.pendingReward(0, 3, addr1.address);
            const rewards5 = await hhFreezer.pendingReward(0, 4, addr1.address);
            await expect(await hhFreezer.pendingRewardTotal(pid, addr1.address)).to.be.equal(
                rewards4 + rewards5,
            );
        });

        it("Should not calculate rewards after freezer end and auto-withdraw", async () => {
            const pid = 0;
            const lockId = 0;
            const amount = ethers.parseEther("1");

            await erc20Token.mint(addr1.address, amount);
            await erc20Token.connect(addr1).approve(hhFreezer.target, amount);

            await hhFreezer.connect(addr1).deposit(pid, lockId, amount);

            const userInfo = await hhFreezer.userInfo(addr1.address, pid);
            const depositId = userInfo.depositId - BigInt(1);

            let userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            await helpers.time.increase(70);
            await hhFreezer.connect(addr1).claim(pid, depositId);
            let userBalance = await erc20Token.balanceOf(addr1.address);
            await expect(userBalance - userBalanceBefore).to.be.not.equal(0);

            userBalanceBefore = await erc20Token.balanceOf(addr1.address);
            await helpers.time.increase(70);
            await helpers.mine(10);
            await hhFreezer.connect(addr1).claim(pid, depositId);
            userBalance = await erc20Token.balanceOf(addr1.address);

            const userDeposit = await hhFreezer.userDeposits(addr1.address, pid, depositId);

            await expect(userBalance - userBalanceBefore).to.be.equal(0);
            await expect(userDeposit.is_finished).to.be.equal(true);
        });
    });
});
