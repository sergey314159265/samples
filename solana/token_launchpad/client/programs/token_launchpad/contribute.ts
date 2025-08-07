import { CONTRIBUTE_SEED, VAULT_SEED } from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { IDL, TokenLaunchpad } from '@target/types/token_launchpad';

export interface ContributeConfig {
  presale: PublicKey;
  user: PublicKey;
  token: PublicKey;
  presaleProgramId: PublicKey;
  tokenProgramId: PublicKey;
  systemProgramId: PublicKey;
}

interface ContributeAccounts {
  presale: PublicKey;
  vault: PublicKey;
  contribution: PublicKey;
  user: PublicKey;
  token: PublicKey;
  tokenProgram: PublicKey;
  systemProgram: PublicKey;
}

export async function contributeToPresale(config: ContributeConfig, amount: anchor.BN) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<TokenLaunchpad>(IDL, config.presaleProgramId, provider);

  // Derive the PDA for the vault using VAULT_SEED and the presale account.
  const [vaultPDA, _vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), config.presale.toBuffer()],
    config.presaleProgramId,
  );

  // Derive the PDA for the contribution account using CONTRIBUTE_SEED, presale, and user.
  const [contributionPDA, _contributionBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(CONTRIBUTE_SEED), config.presale.toBuffer(), config.user.toBuffer()],
    config.presaleProgramId,
  );

  const contributeAccounts: ContributeAccounts = {
    presale: config.presale,
    vault: vaultPDA,
    contribution: contributionPDA,
    user: config.user,
    token: config.token,
    tokenProgram: config.tokenProgramId,
    systemProgram: config.systemProgramId,
  };

  try {
    const tx = await program.methods
      .contribute(amount)
      .accounts({
        presale: contributeAccounts.presale,
        vault: contributeAccounts.vault,
        contribution: contributeAccounts.contribution,
        user: contributeAccounts.user,
        token: contributeAccounts.token,
        tokenProgram: contributeAccounts.tokenProgram,
        systemProgram: contributeAccounts.systemProgram,
      })
      .rpc();

    console.log('Contribution successful!');
    console.log('Transaction signature:', tx);
  } catch (err) {
    console.error('Error contributing to presale:', err);
  }
}
