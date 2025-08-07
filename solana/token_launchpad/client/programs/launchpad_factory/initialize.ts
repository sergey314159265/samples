import { FACTORY_CONFIG_SEED } from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { IDL, LaunchpadFactory } from '@target/types/launchpad_factory';

/**
 * Configuration for factory initialization
 * @property tokenLaunchpadFactoryProgramId - Target program ID for initialization
 */
export interface InitializeLaunchpadFactoryProgramDataConfig {
  tokenLaunchpadFactoryProgramId: PublicKey;
}

/**
 * Initializes the Launchpad Factory program with base configuration
 * @param config - Program initialization parameters
 * @param feeCollector - Initial fee collection account address
 * @param creatorFee - Fee basis points for creation operations
 * @param serviceFee - Platform service fee percentage
 *
 * Critical initialization steps:
 * - Creates factory config PDA with seed derivation
 * - Sets initial fee structure parameters
 * - Establishes immutable program parameters
 * - Requires initial admin authority signature
 * - Can only be executed once per program instance
 */
export async function initializeLaunchpadFactory(
  config: InitializeLaunchpadFactoryProgramDataConfig,
  feeCollector: PublicKey,
  creatorFee: anchor.BN,
  serviceFee: number,
) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<LaunchpadFactory>(
    IDL,
    config.tokenLaunchpadFactoryProgramId,
    provider,
  );

  const [factoryConfigAddress, _] = PublicKey.findProgramAddressSync(
    [Buffer.from(FACTORY_CONFIG_SEED)],
    config.tokenLaunchpadFactoryProgramId,
  );

  const args: [anchor.BN, number] = [creatorFee, serviceFee];

  try {
    const tx = await program.methods
      .initialize(...args)
      .accounts({
        admin: provider.wallet.publicKey,
        factoryConfig: factoryConfigAddress,
        feeCollectorInfo: feeCollector,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log('Factory initialized successfully!');
    console.log('Transaction signature:', tx);
    console.log('Factory config address:', factoryConfigAddress.toString());
  } catch (err) {
    console.error('Error initializing factory:', err);
  }
}
