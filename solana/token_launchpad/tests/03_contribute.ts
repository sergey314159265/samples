import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';

import { TokenLaunchpad } from '../target/types/token_launchpad';
import { admin, mint, sleep, tokenPrice } from './00_setup_tests';

describe('Contribute', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenLaunchpad as anchor.Program<TokenLaunchpad>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let presale: PublicKey;
  let contribution: PublicKey;
  let whitelistEntry: PublicKey;

  before(async () => {
    // artificial delay so that presale is able to start before contribution
    await sleep(3000);

    [presale] = PublicKey.findProgramAddressSync(
      [Buffer.from('presale'), mint.toBuffer(), Buffer.from('presale_id')],
      program.programId,
    );

    [whitelistEntry] = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), presale.toBuffer(), admin.publicKey.toBuffer()],
      program.programId,
    );

    [contribution] = PublicKey.findProgramAddressSync(
      [Buffer.from('contribute'), presale.toBuffer(), admin.publicKey.toBuffer()],
      program.programId,
    );
  });
  it('should fail if lower than min buy', async () => {
    let error: anchor.AnchorError;

    try {
      const amount = 0.5 * 10 ** 9;

      await program.methods
        .contribute(new anchor.BN(amount))
        .accounts({
          presale: presale,
          user: admin.publicKey,
          whitelistEntry: whitelistEntry,
          token: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'ContributionNotWithinLimits');
  });
  it('should fail if higher than max buy', async () => {
    let error: anchor.AnchorError;

    try {
      const amount = 1.6 * 10 ** 9;

      await program.methods
        .contribute(new anchor.BN(amount))
        .accounts({
          presale: presale,
          user: admin.publicKey,
          whitelistEntry: whitelistEntry,
          token: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'ContributionNotWithinLimits');
  });
  it('should create contribution', async () => {
    const amount = 1.5 * 10 ** 9;
    const tokensBought = (amount * 10 ** 9) / tokenPrice.toNumber();

    await program.methods
      .contribute(new anchor.BN(amount))
      .accounts({
        presale: presale,
        user: admin.publicKey,
        whitelistEntry: whitelistEntry,
        token: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    const contributionData = await program.account.contributionState.fetch(contribution);

    assert.equal(amount, contributionData.amount.toNumber());
    assert.deepEqual(admin.publicKey, contributionData.contributor);
    assert.equal(tokensBought, contributionData.tokensPurchased.toNumber());
  });
  it('should fail if hardcap reached', async () => {
    let error: anchor.AnchorError;

    try {
      const amount = 0.75 * 10 ** 9;

      await program.methods
        .contribute(new anchor.BN(amount))
        .accounts({
          presale: presale,
          user: admin.publicKey,
          whitelistEntry: whitelistEntry,
          token: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'ContributionNotWithinLimits');
  });
});
