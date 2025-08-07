import { FACTORY_CONFIG_SEED, PRESALE_SEED, VAULT_SEED } from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { getAssociatedTokenAddressSync, TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { IDL, LaunchpadFactory } from '@target/types/launchpad_factory';

interface CreatePresalePDAs {
  factoryConfigPDA: PublicKey;
  presalePDA: PublicKey;
  vaultPDA: PublicKey;
  vaultATA: PublicKey;
  ownerATA: PublicKey;
  tokenProgramId: PublicKey;
}

interface CreatePresalePrograms {
  tokenLaunchpadFactoryProgramId: PublicKey;
  tokenLaunchpadProgramId: PublicKey;
  associatedTokenProgramId: PublicKey;
  systemProgramId: PublicKey;
}

interface CreatePresaleAddresses {
  mint: PublicKey;
  owner: PublicKey;
  feeCollector: PublicKey;
}

interface CreatePresaleAccounts
  extends CreatePresaleAddresses,
    CreatePresalePDAs,
    CreatePresalePrograms {
  [key: string]: PublicKey;
}

export interface CreatePresaleConfig extends CreatePresalePrograms, CreatePresaleAddresses {}

export interface CreatePresaleArgs {
  tokenPrice: anchor.BN;
  hardCap: anchor.BN;
  softCap: anchor.BN;
  minContribution: anchor.BN;
  maxContribution: anchor.BN;
  startTime: anchor.BN;
  endTime: anchor.BN;
  listingRate: anchor.BN;
  liquidityLockTime: anchor.BN;
  liquidityBp: number;
  refundType: number;
  listingOpt: number;
  liquidityType: number;
  identifier: string;
  affiliateEnabled: boolean;
  commRate: number;
}

export async function createPresale(config: CreatePresaleConfig, args: CreatePresaleArgs) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<LaunchpadFactory>(
    IDL,
    config.tokenLaunchpadFactoryProgramId,
    provider,
  );

  const mintOwner = await provider.connection.getAccountInfo(config.mint).then((info) => info?.owner);

  console.log("Config: ", config);

  const [factoryConfigPDA, _factoryConfigPDASeed] = PublicKey.findProgramAddressSync(
    [Buffer.from(FACTORY_CONFIG_SEED)],
    config.tokenLaunchpadFactoryProgramId,
  );

  const [presalePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(PRESALE_SEED), config.mint.toBuffer(), Buffer.from(args.identifier)],
    config.tokenLaunchpadProgramId,
  );

  const [vaultPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), presalePDA.toBuffer()],
    config.tokenLaunchpadProgramId,
  );

  const ownerATA = getAssociatedTokenAddressSync(
    config.mint,
    config.owner,
    false,
    TOKEN_2022_PROGRAM_ID,
    config.associatedTokenProgramId,
  );

  const vaultATA = getAssociatedTokenAddressSync(
    config.mint,
    presalePDA,
    true,
    TOKEN_2022_PROGRAM_ID,
    config.associatedTokenProgramId,
  );

  // const createOwnerATA = createAssociatedTokenAccountInstruction(
  //     provider.wallet.publicKey,
  //     ownerATA,
  //     config.owner,
  //     config.mint,
  //     TOKEN_2022_PROGRAM_ID,
  //     config.associatedTokenProgramId,
  // );
  //
  // const createVaultATA = createAssociatedTokenAccountInstruction(
  //     provider.wallet.publicKey,
  //     vaultATA,
  //     presalePDA,
  //     config.mint,
  //     TOKEN_2022_PROGRAM_ID,
  //     config.associatedTokenProgramId,
  // );
  //
  // console.log('Vault ATA: ', vaultATA.toBase58());
  //
  // // Add the instruction to a new transaction
  // const tx = new Transaction().add(createOwnerATA, createVaultATA);
  // const blockhash = await provider.connection.getLatestBlockhash();
  // tx.feePayer = provider.wallet.publicKey;
  // tx.recentBlockhash = blockhash.blockhash;
  // await provider.sendAndConfirm(tx);

  const presaleAccounts: CreatePresaleAccounts = {
    ...config,
    factoryConfigPDA: factoryConfigPDA,
    presalePDA: presalePDA,
    vaultPDA: vaultPDA,
    vaultATA: vaultATA,
    ownerATA: ownerATA,
    tokenProgramId: mintOwner
  };

  console.log(mapAccountNames(presaleAccounts));

  console.log(args);

  try {
    const tx = await program.methods
      .createPresale(
        args.tokenPrice,
        args.hardCap,
        args.softCap,
        args.minContribution,
        args.maxContribution,
        args.startTime,
        args.endTime,
        args.listingRate,
        args.liquidityLockTime,
        args.liquidityBp,
        args.refundType == 0 ? { burn: {} } : { refund: {} },
        args.listingOpt == 0 ? { auto: {} } : { manual: {} },
        args.liquidityType == 0 ? { burn: {} } : { lock: {} },
        args.identifier,
        args.affiliateEnabled,
        args.commRate,
      )
      .accounts(mapAccountNames(presaleAccounts))
      .rpc();

    console.log('Token launchpad created successfully!');
    console.log('Presale address:', presalePDA.toBase58());
    console.log('Transaction signature:', tx);
  } catch (err) {
    console.error('Error creating launchpad:', err);
  }
}

const mapAccountNames = (accounts: CreatePresaleAccounts) => {
  return {
    presale: accounts.presalePDA,
    vault: accounts.vaultPDA,
    tokenVaultAccount: accounts.vaultATA,
    factoryConfig: accounts.factoryConfigPDA,
    feeCollector: accounts.feeCollector,
    ownerTokenAccount: accounts.ownerATA,
    owner: accounts.owner,
    presaleProgram: accounts.tokenLaunchpadProgramId,
    tokenMint: accounts.mint,
    tokenProgram: accounts.tokenProgramId,
    associatedTokenProgram: accounts.associatedTokenProgramId,
    systemProgram: accounts.systemProgramId,
  };
};
