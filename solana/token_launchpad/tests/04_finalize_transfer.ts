import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getMinimumBalanceForRentExemptAccount,
} from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

import { TokenLaunchpad } from '../target/types/token_launchpad';
import { WSOL_MINT, admin, feeCollector, mint } from './00_setup_tests';
import { tokenAmounts } from './02_create_presale';

describe('Finalize transfer', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let vault: PublicKey;
  let wsolVaultAta: PublicKey;

  before(async () => {
    [presale] = PublicKey.findProgramAddressSync(
      [Buffer.from('presale'), mint.toBuffer(), Buffer.from('presale_id')],
      program.programId,
    );

    [vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), presale.toBuffer()],
      program.programId,
    );

    wsolVaultAta = await getAssociatedTokenAddress(WSOL_MINT, presale, true, TOKEN_PROGRAM_ID);
  });
  it('should fail if signed not by owner or admin', async () => {
    let error: anchor.AnchorError;

    try {
      await program.methods
        .finalizeTransfer()
        .accountsPartial({
          signer: feeCollector.publicKey,
          presale: presale,
          vault: vault,
          vaultWsolAta: wsolVaultAta,
          wsolMint: WSOL_MINT,
          tokenMint: mint,
        })
        .signers([feeCollector])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'Unauthorized');
  });

  it('should transfer sol to wsol vault', async () => {
    await program.methods
      .finalizeTransfer()
      .accountsPartial({
        signer: admin.publicKey,
        presale: presale,
        vault: vault,
        vaultWsolAta: wsolVaultAta,
        wsolMint: WSOL_MINT,
        tokenMint: mint,
      })
      .signers([admin])
      .rpc();

    const presaleData = await program.account.presaleState.fetch(presale);
    const balanceBefore = await getMinimumBalanceForRentExemptAccount(
      provider.connection,
      'confirmed',
    );
    const balanceAfter = await provider.connection.getBalance(wsolVaultAta);

    assert.equal(balanceBefore + tokenAmounts.liquiditySols, balanceAfter);
    assert.equal(tokenAmounts.liquiditySols, presaleData.solPoolReserve.toNumber());
    assert.equal(tokenAmounts.tokensForLiquidity, presaleData.tokenPoolReserve.toNumber());
  });
});
