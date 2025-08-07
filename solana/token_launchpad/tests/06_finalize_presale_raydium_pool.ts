import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import { ComputeBudgetProgram, PublicKey } from '@solana/web3.js';

import { TokenLaunchpad } from '../target/types/token_launchpad';
import {
  AMM_CONFIG,
  AUTH_SEED,
  CP_SWAP_PROGRAM,
  OBSERVATION_SEED,
  POOL_LP_MINT_SEED,
  POOL_SEED,
  POOL_VAULT_SEED,
  RAYDIUM_FEE_COLLECTOR,
  WSOL_MINT,
  admin,
  mint,
} from './00_setup_tests';
import { tokenAmounts } from './02_create_presale';

describe('Finalize create raydium pool', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let vault: PublicKey;
  let wsolVaultAta: PublicKey;
  let lpMint: PublicKey;
  let creatorLpTokenAddress: PublicKey;
  let authority: PublicKey;
  let poolState: PublicKey;
  let token0Vault: PublicKey;
  let token1Vault: PublicKey;
  let observationState: PublicKey;
  let tokenOwnerAccount: PublicKey;
  let wsolOwnerAccount: PublicKey;
  let tokenVaultAccount: PublicKey;
  let wsolVaultAccount: PublicKey;

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

    [authority] = PublicKey.findProgramAddressSync([Buffer.from(AUTH_SEED)], CP_SWAP_PROGRAM);

    [token0Vault] = PublicKey.findProgramAddressSync(
      [Buffer.from(POOL_VAULT_SEED), poolState.toBuffer(), WSOL_MINT.toBuffer()],
      CP_SWAP_PROGRAM,
    );
    [token1Vault] = PublicKey.findProgramAddressSync(
      [Buffer.from(POOL_VAULT_SEED), poolState.toBuffer(), mint.toBuffer()],
      CP_SWAP_PROGRAM,
    );

    [observationState] = PublicKey.findProgramAddressSync(
      [Buffer.from(OBSERVATION_SEED), poolState.toBuffer()],
      CP_SWAP_PROGRAM,
    );

    wsolVaultAta = await getAssociatedTokenAddress(WSOL_MINT, presale, true, TOKEN_PROGRAM_ID);

    tokenOwnerAccount = await getAssociatedTokenAddress(
      mint,
      admin.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );
    wsolOwnerAccount = await getAssociatedTokenAddress(
      WSOL_MINT,
      admin.publicKey,
      false,
      TOKEN_PROGRAM_ID,
    );

    tokenVaultAccount = await getAssociatedTokenAddress(mint, presale, true, TOKEN_PROGRAM_ID);
    wsolVaultAccount = await getAssociatedTokenAddress(WSOL_MINT, presale, true, TOKEN_PROGRAM_ID);
  });
  it('should create raydium pool', async () => {
    await program.methods
      .finalizePresaleRaydiumPool()
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 })])
      .accountsPartial({
        signer: admin.publicKey,
        presale: presale,
        tokenMint: mint,
        ammConfig: AMM_CONFIG,
        authority: authority,
        poolState: poolState,
        token0Mint: WSOL_MINT,
        token1Mint: mint,
        lpMint: lpMint,
        creatorToken0: wsolOwnerAccount,
        creatorToken1: tokenOwnerAccount,
        creatorLpToken: creatorLpTokenAddress,
        token0Vault,
        token1Vault,
        vault: vault,
        tokenVaultAccount: tokenVaultAccount,
        vaultWsolAta: wsolVaultAccount,
        createPoolFee: RAYDIUM_FEE_COLLECTOR,
        observationState: observationState,
        token0Program: TOKEN_PROGRAM_ID,
        token1Program: TOKEN_PROGRAM_ID,
        cpSwapProgram: CP_SWAP_PROGRAM,
      })
      .signers([admin])
      .rpc();

    const token0Balance = await provider.connection.getTokenAccountBalance(token0Vault);
    const token1Balance = await provider.connection.getTokenAccountBalance(token1Vault);

    assert.equal(tokenAmounts.liquiditySols, Number(token0Balance.value.amount));
    assert.equal(tokenAmounts.tokensForLiquidity, Number(token1Balance.value.amount));
  });
});
