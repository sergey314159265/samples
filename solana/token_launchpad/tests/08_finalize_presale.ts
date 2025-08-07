import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

import { TokenLaunchpad } from '../target/types/token_launchpad';
import {
  AMM_CONFIG,
  CP_SWAP_PROGRAM,
  POOL_SEED,
  WSOL_MINT,
  admin,
  feeCollector,
  mint,
  serviceFee,
} from './00_setup_tests';
import { tokenAmounts } from './02_create_presale';

describe('Finalize presale', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let vault: PublicKey;
  let wsolVaultAta: PublicKey;
  let poolState: PublicKey;
  let tokenVaultAccount: PublicKey;

  before(async () => {
    [presale] = PublicKey.findProgramAddressSync(
      [Buffer.from('presale'), mint.toBuffer(), Buffer.from('presale_id')],
      program.programId,
    );

    [vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), presale.toBuffer()],
      program.programId,
    );

    [poolState] = PublicKey.findProgramAddressSync(
      [Buffer.from(POOL_SEED), AMM_CONFIG.toBuffer(), WSOL_MINT.toBuffer(), mint.toBuffer()],
      CP_SWAP_PROGRAM,
    );

    tokenVaultAccount = await getAssociatedTokenAddress(mint, presale, true, TOKEN_PROGRAM_ID);

    wsolVaultAta = await getAssociatedTokenAddress(WSOL_MINT, presale, true, TOKEN_PROGRAM_ID);
  });
  it('should fail if signed not by owner or admin', async () => {
    let error: anchor.AnchorError;

    try {
      await program.methods
        .finalizePresale()
        .accountsPartial({
          presale: presale,
          signer: feeCollector.publicKey,
          owner: admin.publicKey,
          feeCollector: feeCollector.publicKey,
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          ammConfig: AMM_CONFIG,
          poolProgram: CP_SWAP_PROGRAM,
          poolState: poolState,
        })
        .signers([feeCollector])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'Unauthorized');
  });
  it('should fail if pool has not been created', async () => {
    const pool = PublicKey.unique();
    let error: anchor.AnchorError;

    try {
      await program.methods
        .finalizePresale()
        .accountsPartial({
          presale: presale,
          signer: admin.publicKey,
          owner: admin.publicKey,
          feeCollector: feeCollector.publicKey,
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          ammConfig: AMM_CONFIG,
          poolProgram: CP_SWAP_PROGRAM,
          poolState: pool,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'InvalidRaydiumPoolState');
  });
  it('should finalize presale', async () => {
    const feeBalanceBefore = await provider.connection.getBalance(feeCollector.publicKey);
    const ownerBalanceBefore = await provider.connection.getBalance(admin.publicKey);

    await program.methods
      .finalizePresale()
      .accountsPartial({
        presale: presale,
        signer: admin.publicKey,
        owner: admin.publicKey,
        feeCollector: feeCollector.publicKey,
        vault: vault,
        tokenVaultAccount: tokenVaultAccount,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        ammConfig: AMM_CONFIG,
        poolProgram: CP_SWAP_PROGRAM,
        poolState: poolState,
      })
      .signers([admin])
      .rpc();

    const presaleData = await program.account.presaleState.fetch(presale);

    const fee = (presaleData.totalRaised.toNumber() * serviceFee) / 10000;
    const feeBalanceAfter = await provider.connection.getBalance(feeCollector.publicKey);

    const expectedOwnerReward =
      presaleData.totalRaised.toNumber() - fee - tokenAmounts.liquiditySols;
    const ownerBalanceAfter = await provider.connection.getBalance(admin.publicKey);

    assert.equal(Number(feeBalanceBefore) + fee, Number(feeBalanceAfter));
    assert.equal(Number(ownerBalanceBefore) + expectedOwnerReward, Number(ownerBalanceAfter));
  });
  it('should fail if presale has been already finalized', async () => {
    let error: anchor.AnchorError;

    try {
      await program.methods
        .finalizePresale()
        .accountsPartial({
          presale: presale,
          signer: admin.publicKey,
          owner: admin.publicKey,
          feeCollector: feeCollector.publicKey,
          vault: vault,
          tokenVaultAccount: tokenVaultAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          ammConfig: AMM_CONFIG,
          poolProgram: CP_SWAP_PROGRAM,
          poolState: poolState,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'PresaleFinalizationPreconditionsNotMet');
  });
});
