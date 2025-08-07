import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  mintTo,
} from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

import { LaunchpadFactory } from '../target/types/launchpad_factory';
import { TokenLaunchpad } from '../target/types/token_launchpad';
import {
  TokenAmounts,
  admin,
  calculateTokensToTransfer,
  createAssociatedTokenAccount,
  creatorFee,
  feeCollector,
  mint,
  serviceFee,
  token2022Mint,
} from './00_setup_tests';

export let tokenAmounts: TokenAmounts;

describe('Create presale', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.LaunchpadFactory as anchor.Program<LaunchpadFactory>;
  const launchpadProgram = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let vault: PublicKey;
  let tokenVaultAccount: PublicKey;
  let token2022VaultAccount: PublicKey;
  let tokenAccount: PublicKey;
  let token2022Account: PublicKey;

  const multiplier = Math.pow(10, 9);
  const args = {
    presaleType: { hardCapped: {} },
    tokensAllocated: new anchor.BN(0),
    tokenPrice: new anchor.BN(0.1 * multiplier),
    hardCap: new anchor.BN(1.5 * multiplier),
    softCap: new anchor.BN(0.75 * multiplier),
    minContribution: new anchor.BN(0.75 * multiplier),
    maxContribution: new anchor.BN(1.5 * multiplier),
    startTime: new anchor.BN(Date.now() / 1000 + 5),
    endTime: new anchor.BN(Date.now() / 1000 + 150),
    listingRate: new anchor.BN(100000000),
    liquidityLockTime: new anchor.BN(0),
    liquidityBp: 2000,
    refundType: { refund: {} },
    listingOpt: { manual: {} },
    liquidityType: { burn: {} },
    listingPlatform: { raydium: {} },
    identifier: 'presale_id',
    affiliateEnabled: false,
    whitelistEnabled: false,
    commRate: 0,
    launchpadType: { pro: {} },
  };

  before(async () => {
    [presale] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('presale'), mint.toBuffer(), Buffer.from('presale_id')],
      launchpadProgram.programId,
    );

    [vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), presale.toBuffer()],
      launchpadProgram.programId,
    );

    tokenVaultAccount = await getAssociatedTokenAddress(mint, presale, true, TOKEN_PROGRAM_ID);
    token2022VaultAccount = await getAssociatedTokenAddress(
      token2022Mint,
      presale,
      true,
      TOKEN_2022_PROGRAM_ID,
    );

    tokenAccount = await getAssociatedTokenAddress(mint, admin.publicKey, false, TOKEN_PROGRAM_ID);
    token2022Account = await getAssociatedTokenAddress(
      token2022Mint,
      admin.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await createAssociatedTokenAccount(
      provider.connection,
      admin,
      mint,
      admin.publicKey,
      false,
      { commitment: 'confirmed' },
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    await createAssociatedTokenAccount(
      provider.connection,
      admin,
      token2022Mint,
      admin.publicKey,
      false,
      { commitment: 'confirmed' },
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
  });
  it('should fail with wrong fee collector', async () => {
    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: PublicKey.unique(),
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'InvalidFeeAccount');
  });
  it('should fail with token_2022 and meteora', async () => {
    const listingPlatform = { meteora: {} };

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: token2022VaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: token2022Mint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'InvalidMint');
  });
  it('should fail if softcap is larger than hardcap', async () => {
    const hardCap = new anchor.BN(1.5 * multiplier);
    const softCap = new anchor.BN(1.6 * multiplier);

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          hardCap,
          softCap,
          args.minContribution,
          args.maxContribution,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if minBuy is larger than maxBuy', async () => {
    const maxBuy = new anchor.BN(1.5 * multiplier);
    const minBuy = new anchor.BN(1.6 * multiplier);

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          minBuy,
          maxBuy,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if start time is after end time', async () => {
    const startTime = new anchor.BN(Date.now() / 1000 + 20);
    const endTime = new anchor.BN(Date.now() / 1000 + 15);

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          startTime,
          endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if start time is in the past', async () => {
    const startTime = new anchor.BN(new Date().getTime() - 20);

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if end time is in the past', async () => {
    const startTime = new anchor.BN(Date.now() / 1000 - 20);
    const endTime = new anchor.BN(Date.now() / 1000 - 15);

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          args.hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          startTime,
          endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          args.launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if hardcap is lower than min hardcap for degen', async () => {
    const hardCap = new anchor.BN((Number(program.idl.constants[2].value) - 0.0001) * multiplier);
    const launchpadType = { degen: {} };

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if hardcap is higher than max hardcap for degen', async () => {
    const hardCap = new anchor.BN((Number(program.idl.constants[1]) + 0.0001) * multiplier);
    const launchpadType = { degen: {} };

    let error: anchor.AnchorError;

    try {
      await program.methods
        .createPresale(
          args.presaleType,
          args.tokensAllocated,
          args.tokenPrice,
          hardCap,
          args.softCap,
          args.minContribution,
          args.maxContribution,
          args.startTime,
          args.endTime,
          args.listingRate,
          args.liquidityLockTime,
          args.liquidityBp,
          args.refundType,
          args.listingOpt,
          args.liquidityType,
          args.listingPlatform,
          args.identifier,
          args.affiliateEnabled,
          args.whitelistEnabled,
          args.commRate,
          launchpadType,
        )
        .accounts({
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          feeCollector: feeCollector.publicKey,
          owner: admin.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          presale: presale,
          presaleProgram: launchpadProgram.programId,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }

    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should init a presale', async () => {
    const feeBalanceBefore = await provider.connection.getBalance(feeCollector.publicKey);

    await mintTo(
      provider.connection,
      admin,
      mint,
      tokenAccount,
      admin.publicKey,
      100 * multiplier,
      [],
      {
        commitment: 'confirmed',
      },
      TOKEN_PROGRAM_ID,
    );

    await program.methods
      .createPresale(
        args.presaleType,
        args.tokensAllocated,
        args.tokenPrice,
        args.hardCap,
        args.softCap,
        args.minContribution,
        args.maxContribution,
        args.startTime,
        args.endTime,
        args.listingRate,
        args.liquidityLockTime,
        args.liquidityBp,
        args.refundType,
        args.listingOpt,
        args.liquidityType,
        args.listingPlatform,
        args.identifier,
        args.affiliateEnabled,
        args.whitelistEnabled,
        args.commRate,
        args.launchpadType,
      )
      .accounts({
        vault: vault,
        tokenVaultAccount: tokenVaultAccount,
        feeCollector: feeCollector.publicKey,
        owner: admin.publicKey,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        presale: presale,
        presaleProgram: launchpadProgram.programId,
      })
      .signers([admin])
      .rpc();

    const presaleData = await launchpadProgram.account.presaleState.fetch(presale);

    const tokenVaultBalance = await provider.connection.getTokenAccountBalance(tokenVaultAccount);
    const feeBalanceAfter = await provider.connection.getBalance(feeCollector.publicKey);
    tokenAmounts = calculateTokensToTransfer(
      args.hardCap.toNumber(),
      serviceFee,
      args.liquidityBp,
      9,
      args.tokenPrice.toNumber(),
      args.listingRate.toNumber(),
    );

    assert.equal(feeBalanceBefore + creatorFee, feeBalanceAfter);
    assert.equal(tokenAmounts.presaleTokens.toString(), tokenVaultBalance.value.amount);

    assert.deepEqual(args.presaleType, presaleData.presaleType as any);
    assert.equal(args.tokensAllocated.toNumber(), presaleData.totalTokensSold.toNumber());
    assert.equal(args.tokenPrice.toNumber(), presaleData.tokenPrice.toNumber());
    assert.equal(args.hardCap.toNumber(), presaleData.hardCap.toNumber());
    assert.equal(args.softCap.toNumber(), presaleData.softCap.toNumber());
    assert.equal(args.minContribution.toNumber(), presaleData.minContribution.toNumber());
    assert.equal(args.maxContribution.toNumber(), presaleData.maxContribution.toNumber());
    assert.equal(args.startTime.toNumber(), presaleData.startTime.toNumber());
    assert.equal(args.endTime.toNumber(), presaleData.endTime.toNumber());
    assert.equal(args.listingRate.toNumber(), presaleData.listingRate.toNumber());
    assert.equal(args.liquidityLockTime.toNumber(), presaleData.liquidityLockTime.toNumber());
    assert.equal(args.liquidityBp, presaleData.liquidityBp);
    assert.deepEqual(args.refundType, presaleData.refundType as any);
    assert.deepEqual(args.listingOpt, presaleData.listingOpt as any);
    assert.deepEqual(args.liquidityType, presaleData.liquidityType as any);
    assert.deepEqual(args.listingPlatform, presaleData.listingPlatform as any);
    assert.equal(args.identifier, presaleData.identifier);
    assert.equal(args.affiliateEnabled, presaleData.affiliateEnabled);
    assert.equal(args.whitelistEnabled, presaleData.whitelistEnabled);
    assert.equal(args.commRate, presaleData.commissionRate);
  });
});
