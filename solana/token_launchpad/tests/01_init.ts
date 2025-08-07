import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';

import { LaunchpadFactory } from '../target/types/launchpad_factory';
import {
  FACTORY_CONFIG_SEED,
  admin,
  creatorFee,
  feeCollector,
  manager,
  serviceFee,
} from './00_setup_tests';

describe('Init factory', () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.LaunchpadFactory as anchor.Program<LaunchpadFactory>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  before(async () => {});
  it('should fail if exceeds maximum creator fee', async () => {
    const args: [anchor.BN, number] = [new anchor.BN(10_000_000_100), 400];
    let error: anchor.AnchorError;

    try {
      await program.methods
        .initialize(...args)
        .accounts({
          admin: admin.publicKey,
          feeCollectorInfo: feeCollector.publicKey,
          manager: manager.publicKey,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should fail if exceeds maximum service fee', async () => {
    const args: [anchor.BN, number] = [new anchor.BN(300), 2600];
    let error: anchor.AnchorError;

    try {
      await program.methods
        .initialize(...args)
        .accounts({
          admin: admin.publicKey,
          feeCollectorInfo: feeCollector.publicKey,
          manager: manager.publicKey,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      error = err as anchor.AnchorError;
    }
    assert.equal(error.error.errorCode.code, 'Invalid');
  });
  it('should init the factory', async () => {
    const [factoryConfigAddress, _] = PublicKey.findProgramAddressSync(
      [Buffer.from(FACTORY_CONFIG_SEED)],
      program.programId,
    );

    const args: [anchor.BN, number] = [new anchor.BN(creatorFee), serviceFee];

    await program.methods
      .initialize(...args)
      .accounts({
        admin: admin.publicKey,
        feeCollectorInfo: feeCollector.publicKey,
        manager: manager.publicKey,
      })
      .signers([admin])
      .rpc();

    const factoryData = await program.account.factory.fetch(factoryConfigAddress);

    assert.deepEqual(factoryData.admin, admin.publicKey);
    assert.equal(factoryData.creatorFee.toNumber(), 300);
    assert.equal(factoryData.serviceFee, 400);
    assert.deepEqual(factoryData.feeCollector, feeCollector.publicKey);
    assert.deepEqual(factoryData.manager, manager.publicKey);
  });
});
