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
  LP_TOKEN_LOCK_SEED,
  POOL_LP_MINT_SEED,
  POOL_SEED,
  WSOL_MINT,
  admin,
  feeCollector,
  mint,
} from './00_setup_tests';

describe('Finalize lp lock burn', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let vault: PublicKey;
  let wsolVaultAta: PublicKey;
  let lpMint: PublicKey;
  let poolState: PublicKey;
  let creatorLpTokenAddress: PublicKey;
  let lpTokenLockPda: PublicKey;
  let lpTokenLockAta: PublicKey;

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

    [lpMint] = PublicKey.findProgramAddressSync(
      [Buffer.from(POOL_LP_MINT_SEED), poolState.toBuffer()],
      CP_SWAP_PROGRAM,
    );

    [creatorLpTokenAddress] = PublicKey.findProgramAddressSync(
      [admin.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), lpMint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    [lpTokenLockPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(LP_TOKEN_LOCK_SEED), presale.toBuffer()],
      program.programId,
    );

    lpTokenLockAta = await getAssociatedTokenAddress(lpMint, lpTokenLockPda, true);

    wsolVaultAta = await getAssociatedTokenAddress(WSOL_MINT, presale, true, TOKEN_PROGRAM_ID);
  });
  it('should fail if signed not by owner or admin', async () => {
    let error: anchor.AnchorError;

    try {
      await program.methods
        .finalizeLpLockBurn()
        .accountsPartial({
          presale: presale,
          signer: feeCollector.publicKey,
          lpTokenLock: lpTokenLockPda,
          lpTokenLockAta: lpTokenLockAta,
          tokenMint: mint,
          lpMint,
          creatorLpToken: creatorLpTokenAddress,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([feeCollector])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'Unauthorized');
  });
  it('should burn the lp', async () => {
    const lpBalanceBefore = await provider.connection.getTokenAccountBalance(creatorLpTokenAddress);

    await program.methods
      .finalizeLpLockBurn()
      .accountsPartial({
        presale: presale,
        signer: admin.publicKey,
        lpTokenLock: lpTokenLockPda,
        lpTokenLockAta: lpTokenLockAta,
        tokenMint: mint,
        lpMint,
        creatorLpToken: creatorLpTokenAddress,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    const lpBalanceAfter = await provider.connection.getTokenAccountBalance(creatorLpTokenAddress);
    const lpLockBalance = await provider.connection.getTokenAccountBalance(lpTokenLockAta);

    assert.isTrue(lpBalanceBefore.value.amount > '0');
    assert.isTrue(lpBalanceAfter.value.amount === '0');
    assert.isTrue(lpLockBalance.value.amount === '0');
  });
});
