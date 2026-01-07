import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorProject } from "../target/types/anchor_project";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

// Define the test suite
describe("anchor_project", () => {

  // Configure the provider to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Create TS types from IDL
  const program = anchor.workspace.AnchorProject as Program<AnchorProject>;

  // Create user from local keypair (~/.config/solana/id.json)
  const user = provider.wallet as anchor.Wallet;

  // Declare types
  let mintX: anchor.web3.PublicKey;
  let mintY: anchor.web3.PublicKey;
  let userTokenX: anchor.web3.PublicKey;
  let userTokenY: anchor.web3.PublicKey;
  let vaultTokenX: anchor.web3.PublicKey;
  let vaultTokenY: anchor.web3.PublicKey;
  let stateAccountPDA: anchor.web3.PublicKey;

  // Constants
  const SPREAD_BPS = new anchor.BN(500); // 5%

  // Run this before all tests in this suite
  before(async () => {
    // 1. Create Mints (since we are on localnet)
    mintX = await createMint(
      provider.connection,
      user.payer,
      user.publicKey,
      null,
      6
    );
    mintY = await createMint(
      provider.connection,
      user.payer,
      user.publicKey,
      null,
      6
    );

    // 2. Create user token accounts
    userTokenX = await createAccount(
      provider.connection,
      user.payer,
      mintX,
      user.publicKey
    );
    userTokenY = await createAccount(
      provider.connection,
      user.payer,
      mintY,
      user.publicKey
    );

    // 3. Mint tokens to user token accounts
    await mintTo(
      provider.connection,
      user.payer,
      mintX,
      userTokenX,
      user.payer,
      10_000_000_000 // 10,000 X tokens (because 6 decimals)
    );
    await mintTo(
      provider.connection,
      user.payer,
      mintY,
      userTokenY,
      user.payer,
      10_000_000_000 // 10,000 Y tokens
    );

    // 4. Derive State PDA
    const [statePDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state"), mintX.toBuffer(), mintY.toBuffer()],
      program.programId
    );
    stateAccountPDA = statePDA;

    // 5. We need to find the ATA for the State Account PDA
    vaultTokenX = await getAssociatedTokenAddress(
        mintX,
        stateAccountPDA,
        true 
    );
    vaultTokenY = await getAssociatedTokenAddress(
        mintY,
        stateAccountPDA,
        true
    );
  });

  // Actual test case
  it("Calling the init instruction", async () => {
    try {

      // Call the TS type's init() method
      await program.methods
        .init(SPREAD_BPS)
        .accounts({
          user: user.publicKey,
          tokenXMint: mintX,
          tokenYMint: mintY,
        })
        .rpc();

      // Fetch account and assert expected values
      const state = await program.account.stateAccount.fetch(stateAccountPDA);
      assert.ok(state.spreadBps.eq(SPREAD_BPS));
      assert.ok(state.xToYScaledPrice.eq(new anchor.BN(0))); // Initialized to 0
      assert.ok(state.authority.equals(user.publicKey));

      // Mint initial liquidity to vaults (no instruction for this yet)
      await mintTo(
        provider.connection,
        user.payer,
        mintX,
        vaultTokenX,
        user.payer,
        10_000_000_000
      );
      await mintTo(
        provider.connection,
        user.payer,
        mintY,
        vaultTokenY,
        user.payer,
        10_000_000_000
      );

      // Verify balances
      const vaultXAmount = (await getAccount(provider.connection, vaultTokenX)).amount;
      const vaultYAmount = (await getAccount(provider.connection, vaultTokenY)).amount;
      
      assert.equal(Number(vaultXAmount), 10_000_000_000);
      assert.equal(Number(vaultYAmount), 10_000_000_000);

    } catch (e) {
      console.error(e);
      throw e;
    }
  });

});