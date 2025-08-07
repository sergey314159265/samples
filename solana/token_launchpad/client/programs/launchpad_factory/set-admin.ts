import { FACTORY_CONFIG_SEED } from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { IDL, LaunchpadFactory } from '@target/types/launchpad_factory';

/**
 * Configuration parameters for changing the Launchpad Factory admin
 * @property tokenLaunchpadFactoryProgramId - The public key of the deployed Launchpad Factory program
 */
export interface ChangeLaunchpadFactoryAdminConfig {
  tokenLaunchpadFactoryProgramId: PublicKey;
}

/**
 * Executes admin change transaction for the Launchpad Factory program
 * @param config - Configuration object containing program identifier
 * @param newAdmin - Public key of the new admin account
 *
 * This function:
 * - Derives the factory config PDA using predefined seeds
 * - Constructs and sends a `setAdmin` instruction to the program
 * - Requires current admin wallet connection for transaction signing
 * - Updates program authority through BPFLoader upgradeable program
 */
export async function changeLaunchpadFactoryAdmin(
  config: ChangeLaunchpadFactoryAdminConfig,
  newAdmin: PublicKey,
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

  const programDataAddress = PublicKey.findProgramAddressSync(
    [config.tokenLaunchpadFactoryProgramId.toBuffer()],
    new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),
  )[0];

  try {
    const tx = await program.methods
      .setAdmin()
      .accounts({
        admin: provider.wallet.publicKey,
        factoryConfig: factoryConfigAddress,
        newAdmin: newAdmin,
        programData: programDataAddress,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log('Transaction successful:', tx);
  } catch (err) {
    console.error('Error executing set_admin:', err);
  }
}
