import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { IDL, TokenLaunchpad } from '@target/types/token_launchpad';

export interface FetchTokenLaunchpadPresaleData {
  tokenLaunchpadProgramId: PublicKey;
  presaleAddress: PublicKey;
}

export async function fetchTokenLaunchpadPresaleData(config: FetchTokenLaunchpadPresaleData) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<TokenLaunchpad>(IDL, config.tokenLaunchpadProgramId, provider);

  const presaleDate = await program.account.presaleState.fetch(config.presaleAddress);

  return presaleDate;
}
