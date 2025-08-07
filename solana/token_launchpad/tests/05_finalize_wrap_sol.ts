import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

import { TokenLaunchpad } from '../target/types/token_launchpad';
import { WSOL_MINT, admin, feeCollector, mint } from './00_setup_tests';
import { tokenAmounts } from './02_create_presale';

describe('Finalize wrap sol', () => {
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
        .finalizeWrapSol()
        .accountsPartial({
          signer: feeCollector.publicKey,
          presale: presale,
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

  it('should wrap sol', async () => {
    await program.methods
      .finalizeWrapSol()
      .accountsPartial({
        signer: admin.publicKey,
        presale: presale,
        vaultWsolAta: wsolVaultAta,
        wsolMint: WSOL_MINT,
        tokenMint: mint,
      })
      .signers([admin])
      .rpc();

    const wsolVaultBalance = await provider.connection.getTokenAccountBalance(wsolVaultAta);

    assert.equal(tokenAmounts.liquiditySols.toString(), wsolVaultBalance.value.amount);
  });
});
