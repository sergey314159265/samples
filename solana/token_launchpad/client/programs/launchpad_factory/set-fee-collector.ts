import { FACTORY_CONFIG_SEED } from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { IDL, LaunchpadFactory } from '@target/types/launchpad_factory';

/**
 * Configuration parameters for modifying fee collector address
 * @property tokenLaunchpadFactoryProgramId - On-chain program identifier
 */
export interface ChangeLaunchpadFactoryFeeCollectorConfig {
  tokenLaunchpadFactoryProgramId: PublicKey;
}

/**
 * Updates the fee collector address for protocol fees
 * @param config - Program configuration parameters
 * @param newFeeCollector - Public key of new fee collection account
 *
 * Operation details:
 * - Requires current admin authority signature
 * - Modifies fee collector in factory config PDA
 * - Does not affect existing fee allocations
 * - Immediate effect on future transactions
 */
export async function changeLaunchpadFactoryFeeCollector(
  config: ChangeLaunchpadFactoryFeeCollectorConfig,
  newFeeCollector: PublicKey,
) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<LaunchpadFactory>(
    IDL,
    config.tokenLaunchpadFactoryProgramId,
    provider,
  );

  const [factoryConfigAddress] = PublicKey.findProgramAddressSync(
    [Buffer.from(FACTORY_CONFIG_SEED)],
    config.tokenLaunchpadFactoryProgramId,
  );

  try {
    const tx = await program.methods
      .setFeeCollector()
      .accounts({
        admin: provider.wallet.publicKey,
        factoryConfig: factoryConfigAddress,
        feeCollectorInfo: newFeeCollector,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log('Transaction successful:', tx);
  } catch (err) {
    console.error('Error executing set_admin:', err);
  }
}
