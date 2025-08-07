import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { IDL, LaunchpadFactory } from '@target/types/launchpad_factory';

/**
 * Configuration for retrieving factory program data
 * @property tokenLaunchpadFactoryProgramId - Program address for factory instance
 */
export interface FetchLaunchpadFactoryProgramDataConfig {
  tokenLaunchpadFactoryProgramId: PublicKey;
}

/**
 * Retrieves current factory configuration state
 * @param config - Program identifier configuration
 *
 * Fetches and displays:
 * - Current admin authority
 * - Active fee collector address
 * - Configured fee structure
 * - Program version information
 */
export async function fetchLaunchpadFactoryProgramData(
  config: FetchLaunchpadFactoryProgramDataConfig,
) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<LaunchpadFactory>(
    IDL,
    config.tokenLaunchpadFactoryProgramId,
    provider,
  );

  const [factoryConfigAddress] = PublicKey.findProgramAddressSync(
    [Buffer.from('factory_config')],
    config.tokenLaunchpadFactoryProgramId,
  );

  try {
    const factoryAccount = await program.account.factory.fetch(factoryConfigAddress);
    console.log('Factory Account Data:', factoryAccount);
  } catch (err) {
    console.error('Error fetching factory account:', err);
  }
}
