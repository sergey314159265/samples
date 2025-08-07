import * as anchor from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createMint,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import {
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

export let admin: Keypair;
export let feeCollector: Keypair;
export let manager: Keypair;
export let mint: PublicKey;
export let token2022Mint: PublicKey;

export const FACTORY_CONFIG_SEED = 'launchpad_factory_config';
export const POOL_LP_MINT_SEED = 'pool_lp_mint';
export const POOL_SEED = 'pool';
export const LP_TOKEN_LOCK_SEED = 'lp_token_lock';
export const AUTH_SEED = 'vault_and_lp_mint_auth_seed';
export const POOL_VAULT_SEED = 'pool_vault';
export const OBSERVATION_SEED = 'observation';
export const AMM_CONFIG = new PublicKey('9zSzfkYy6awexsHvmggeH36pfVUdDGyCcwmjT3AQPBj6');
export const RAYDIUM_FEE_COLLECTOR = new PublicKey('G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2');
export const WSOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');
export const CP_SWAP_PROGRAM = new PublicKey('CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW');
export const creatorFee = 300;
export const serviceFee = 400;
export const tokenPrice = new anchor.BN(0.1 * 10 ** 9);

anchor.setProvider(anchor.AnchorProvider.env());
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

before(async () => {
  admin = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([
      242, 81, 198, 42, 212, 68, 170, 54, 201, 186, 169, 135, 182, 97, 173, 221, 226, 101, 186, 4,
      164, 80, 76, 182, 70, 16, 65, 212, 42, 176, 67, 227, 124, 2, 239, 70, 149, 70, 111, 104, 113,
      97, 38, 152, 18, 15, 183, 235, 249, 254, 140, 168, 67, 71, 133, 125, 229, 31, 175, 40, 65,
      253, 244, 80,
    ]),
  );
  feeCollector = anchor.web3.Keypair.generate();
  manager = anchor.web3.Keypair.generate();

  const airdropSignatureAdmin = await provider.connection.requestAirdrop(
    admin.publicKey,
    4 * anchor.web3.LAMPORTS_PER_SOL,
  );
  await provider.connection.confirmTransaction(airdropSignatureAdmin);

  const airdropSignatureFeeCollector = await provider.connection.requestAirdrop(
    feeCollector.publicKey,
    anchor.web3.LAMPORTS_PER_SOL,
  );
  await provider.connection.confirmTransaction(airdropSignatureFeeCollector);

  const airdropSignatureManager = await provider.connection.requestAirdrop(
    feeCollector.publicKey,
    anchor.web3.LAMPORTS_PER_SOL,
  );
  await provider.connection.confirmTransaction(airdropSignatureManager);

  mint = await createMint(
    provider.connection,
    admin,
    admin.publicKey,
    admin.publicKey,
    9,
    Keypair.generate(),
    {
      commitment: 'confirmed',
    },
    TOKEN_PROGRAM_ID,
  );

  token2022Mint = await createMint(
    provider.connection,
    admin,
    admin.publicKey,
    admin.publicKey,
    9,
    Keypair.generate(),
    {
      commitment: 'confirmed',
    },
    TOKEN_2022_PROGRAM_ID,
  );
});

export async function createAssociatedTokenAccount(
  connection: Connection,
  payer: Signer,
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve = false,
  confirmOptions?: ConfirmOptions,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
): Promise<void> {
  const associatedToken = getAssociatedTokenAddressSync(
    mint,
    owner,
    allowOwnerOffCurve,
    programId,
    associatedTokenProgramId,
  );

  const transaction = new Transaction().add(
    createAssociatedTokenAccountInstruction(
      payer.publicKey,
      associatedToken,
      owner,
      mint,
      programId,
      associatedTokenProgramId,
    ),
  );

  await sendAndConfirmTransaction(connection, transaction, [payer], confirmOptions);
}

export function calculateTokensToTransfer(
  hardCap: number,
  serviceFee: number,
  liquidityBp: number,
  decimals: number,
  tokenPrice: number,
  listingRate: number,
): TokenAmounts {
  const multiplier = 10 ** decimals;
  const serviceFeeAmount = (hardCap * serviceFee) / 10000;
  const netHardcap = hardCap - serviceFeeAmount;
  const liquiditySols = (netHardcap * liquidityBp) / 10000;
  const tokensForPresale = (hardCap * multiplier) / tokenPrice;
  const tokensForLiquidity = (liquiditySols * multiplier) / listingRate;
  const presaleTokens = tokensForPresale + tokensForLiquidity;

  return {
    presaleTokens,
    tokensForLiquidity,
    liquiditySols,
  };
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface TokenAmounts {
  presaleTokens: number;
  tokensForLiquidity: number;
  liquiditySols: number;
}
