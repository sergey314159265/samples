import {
  AUTH_SEED,
  OBSERVATION_SEED,
  POOL_LP_MINT_SEED,
  POOL_SEED,
  POOL_VAULT_SEED,
  PRESALE_SEED,
  VAULT_SEED,
  WRAPPED_SOL_MINT_ADDRESS,
} from '@client/constants';
import * as anchor from '@coral-xyz/anchor';
import { ASSOCIATED_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/utils/token';
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import { PublicKey, Transaction } from '@solana/web3.js';
import { IDL, TokenLaunchpad } from '@target/types/token_launchpad';

import { fetchTokenLaunchpadPresaleData } from './fetch-presale-data';

export interface FinalizePresaleConfig {
  presaleAddress: PublicKey;
  tokenLaunchpadProgramId: PublicKey;
  associatedTokenProgramId: PublicKey;
  systemProgramId: PublicKey;
  // --raydium--
  cpSwapProgramId: PublicKey;
  ammConfig: PublicKey;
  createPoolFee: PublicKey;
}

interface FinalizePresaleAccounts {
  presale: PublicKey;
  owner: PublicKey;
  feeCollector: PublicKey;
  vault: PublicKey;
  tokenVaultAccount: PublicKey;
  ownerTokenAccount: PublicKey;
  tokenMint: PublicKey;
  tokenProgram: PublicKey;
  associatedTokenProgram: PublicKey;
  systemProgram: PublicKey;
}

export async function finalizePresale(config: FinalizePresaleConfig) {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new anchor.Program<TokenLaunchpad>(IDL, config.tokenLaunchpadProgramId, provider);

  console.log('Fetching presale data...');

  const presaleData = await fetchTokenLaunchpadPresaleData({
    tokenLaunchpadProgramId: config.tokenLaunchpadProgramId,
    presaleAddress: config.presaleAddress,
  });

  console.log('Presale data:', presaleData);

  const [token0Mint, token1Mint] =
    WRAPPED_SOL_MINT_ADDRESS > presaleData.token
      ? [WRAPPED_SOL_MINT_ADDRESS, presaleData.token]
      : [presaleData.token, WRAPPED_SOL_MINT_ADDRESS];
  const [mintOwner, token0MintOwner, token1MintOwner] = [
      await provider.connection.getAccountInfo(presaleData.token).then((info) => info?.owner),
      await provider.connection.getAccountInfo(token0Mint).then((info) => info?.owner),
      await provider.connection.getAccountInfo(token1Mint).then((info) => info?.owner),
  ];

  const token0ATA = getAssociatedTokenAddressSync(
    token0Mint,
    provider.wallet.publicKey,
    false,
    token0MintOwner,
    config.associatedTokenProgramId,
  );

  const token1ATA = getAssociatedTokenAddressSync(
    token1Mint,
    provider.wallet.publicKey,
    true,
    token1MintOwner,
    config.associatedTokenProgramId,
  );

  // Derive the presale PDA using seeds: [PRESALE_SEED, tokenMint, presaleIdentifier].
  const [presalePDA, _presaleBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(PRESALE_SEED), presaleData.token.toBuffer(), Buffer.from(presaleData.identifier)],
    config.tokenLaunchpadProgramId,
  );

  // Derive the vault PDA using seeds: [VAULT_SEED, presalePDA].
  const [vaultPDA, _vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), presalePDA.toBuffer()],
    config.tokenLaunchpadProgramId,
  );

  // Derive the associated token account for the token vault (authority = presale PDA).
  const tokenVaultAccount = await getAssociatedTokenAddress(
    presaleData.token,
    presalePDA,
    true,
    mintOwner,
    config.associatedTokenProgramId,
  );

  // Get (or create) the owner's associated token account for the token mint.
  const ownerTokenAccountInfo = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet as unknown as anchor.web3.Keypair,
    presaleData.token,
    presaleData.owner,
    true,
    null,
    null,
    mintOwner,
    config.associatedTokenProgramId
  );
  const ownerTokenAccount = ownerTokenAccountInfo.address;

  // Derive the authority PDA
  const [authority] = PublicKey.findProgramAddressSync(
    [Buffer.from(AUTH_SEED)],
    config.cpSwapProgramId,
  );

  // Derive the pool state PDA using the AMM config and token mints
  const [poolState] = PublicKey.findProgramAddressSync(
    [
      Buffer.from(POOL_SEED),
      config.ammConfig.toBuffer(),
      token0Mint.toBuffer(),
      token1Mint.toBuffer(),
    ],
    config.cpSwapProgramId,
  );

  // Derive the LP mint PDA
  const [lpMint] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_LP_MINT_SEED), poolState.toBuffer()],
    config.cpSwapProgramId,
  );
  const [creatorLpTokenAddress] = PublicKey.findProgramAddressSync(
    [provider.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), lpMint.toBuffer()],
    ASSOCIATED_PROGRAM_ID,
  );

  // Derive the token vault PDAs for token0 and token1
  const [token0Vault] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_VAULT_SEED), poolState.toBuffer(), token0Mint.toBuffer()],
    config.cpSwapProgramId,
  );
  const [token1Vault] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_VAULT_SEED), poolState.toBuffer(), token1Mint.toBuffer()],
    config.cpSwapProgramId,
  );

  // Derive the observation state PDA
  const [observationState] = PublicKey.findProgramAddressSync(
    [Buffer.from(OBSERVATION_SEED), poolState.toBuffer()],
    config.cpSwapProgramId,
  );

  const finalizeAccounts: FinalizePresaleAccounts = {
    presale: presalePDA,
    owner: presaleData.owner,
    feeCollector: presaleData.feeCollector,
    vault: vaultPDA,
    tokenVaultAccount: tokenVaultAccount,
    ownerTokenAccount: ownerTokenAccount,
    tokenMint: presaleData.token,
    tokenProgram: mintOwner,
    associatedTokenProgram: config.associatedTokenProgramId,
    systemProgram: config.systemProgramId,
  };

  console.log({ finalizeAccounts });

  // const poolStateAccountData = await provider.connection.getAccountInfo(poolState);
  const [token0Program, token1Program] = [
      await provider.connection.getAccountInfo(token0Mint).then((v) => v.owner),
      await provider.connection.getAccountInfo(token1Mint).then((v) => v.owner),
  ];

  try {
    const ix1 = await program.methods
      .finalizePresaleLpPool()
      .accounts({
        presale: config.presaleAddress,
        creator: provider.wallet.publicKey,
        tokenMint: presaleData.token,
        cpSwapProgram: config.cpSwapProgramId,
        ammConfig: config.ammConfig,
        authority,
        poolState,
        token0Mint,
        token1Mint,
        lpMint,
        creatorToken0: token0ATA,
        creatorToken1: token1ATA,
        creatorLpToken: creatorLpTokenAddress,
        token0Vault,
        token1Vault,
        createPoolFee: config.createPoolFee,
        observationState,
        token0Program,
        token1Program
      })
      .instruction();

    const ix2 = await program.methods
      .finalizePresale()
      .accounts({
        presale: finalizeAccounts.presale,
        owner: finalizeAccounts.owner,
        feeCollector: finalizeAccounts.feeCollector,
        vault: finalizeAccounts.vault,
        tokenVaultAccount: finalizeAccounts.tokenVaultAccount,
        ownerTokenAccount: finalizeAccounts.ownerTokenAccount,
        tokenMint: finalizeAccounts.tokenMint,
        tokenProgram: finalizeAccounts.tokenProgram,
        associatedTokenProgram: finalizeAccounts.associatedTokenProgram,
        systemProgram: finalizeAccounts.systemProgram,
        cpSwapProgram: config.cpSwapProgramId,
        ammConfig: config.ammConfig,
        poolState: poolState,
      })
      .instruction();

    const tx = new Transaction();
    tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;

    // if (poolStateAccountData?.lamports === 0) {
      tx.add(ix1);
    // }

    tx.add(ix2);

    provider.sendAndConfirm(tx);

    console.log('Presale finalized successfully!');
    console.log('Presale address: ', presalePDA.toBase58());
    console.log('Raydium Pool address: ', poolState.toBase58());
    console.log('Transaction signature:', tx);
  } catch (err) {
    console.error('Error finalizing presale:', err);
  }
}
