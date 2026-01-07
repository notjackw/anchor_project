import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorProject } from "../target/types/anchor_project";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

// Define the test suite
describe("Swap Exact In", () => {

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
  let vaultTokenX: anchor.web3.PublicKey; // Changed to PublicKey (ATA)
  let vaultTokenY: anchor.web3.PublicKey; // Changed to PublicKey (ATA)
  let stateAccountPDA: anchor.web3.PublicKey;

  const SPREAD_BPS = new anchor.BN(500); // 5%
  const PRICE_SCALE = new anchor.BN(1_000_000);
  const INITIAL_PRICE = PRICE_SCALE; // 1:1

  // Run this before all tests in this suite
  before(async () => {
    // 1. Create Mints
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

    // 2. Create User Token Accounts
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

    // 3. Mint tokens to user
    await mintTo(
      provider.connection,
      user.payer,
      mintX,
      userTokenX,
      user.payer,
      10_000_000_000 // 10,000 tokens
    );
    await mintTo(
      provider.connection,
      user.payer,
      mintY,
      userTokenY,
      user.payer,
      10_000_000_000 // 10,000 tokens
    );

    // 4. Derive State PDA
    const [statePDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state"), mintX.toBuffer(), mintY.toBuffer()],
      program.programId
    );
    stateAccountPDA = statePDA;

    // 5. Derive Vault ATAs (owned by State PDA)
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

    // 6. Initialize Program
    await program.methods
      .init(SPREAD_BPS)
      .accounts({
        user: user.publicKey,
        tokenXMint: mintX,
        tokenYMint: mintY,
      })
      .rpc();

    // 7. Update Price to 1:1 (Initial is 0, need valid price for swap)
    await program.methods
      .updateParams(INITIAL_PRICE, null)
      .accounts({
        tokenXMint: mintX,
        tokenYMint: mintY,
      })
      .rpc();

    // 8. Fund Vaults with Liquidity
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
  });

  // Test 1: Input is X
  it("Test 1: Swaps Exact In (input is X)", async () => {
    // Price is 1:1, Spread is 5% (500bps)
    // Input 1000 X
    // Output should be: 1000 Y - 5% = 950 Y
    const inputAmount = new anchor.BN(1_000_000);
    const userMinOutAmount = new anchor.BN(900_000); // Expect 950_000
    const inputIsX = true;

    const userXBefore = (await getAccount(provider.connection, userTokenX))
      .amount;
    const userYBefore = (await getAccount(provider.connection, userTokenY))
      .amount;

    await program.methods
      .swapExactIn(inputAmount, inputIsX, userMinOutAmount)
      .accounts({
        userWallet: user.publicKey,
        userTokenXAcct: userTokenX,
        userTokenYAcct: userTokenY,
        vaultTokenXAcct: vaultTokenX,
        vaultTokenYAcct: vaultTokenY,
        tokenXMint: mintX,
        tokenYMint: mintY,
      })
      .rpc();

    const userXAfter = (await getAccount(provider.connection, userTokenX))
      .amount;
    const userYAfter = (await getAccount(provider.connection, userTokenY))
      .amount;

    // X decreased by 1_000_000
    assert.ok(new anchor.BN(Number(userXBefore)).sub(new anchor.BN(Number(userXAfter))).eq(inputAmount));
    // Y increased by ~950_000
    assert.ok(Number(userYAfter) > Number(userYBefore));
  });

  it("Test 2: Swaps Exact In (input is Y)", async () => {
    // Price is 1:1, Spread is 5% (500bps)
    // Input 1000 Y
    // Output should be: 1000 X - 5% = 950 X
    const inputAmount = new anchor.BN(1_000_000);
    const userMinOutAmount = new anchor.BN(900_000); // Expect 950_000
    const inputIsX = false;

    const userXBefore = (await getAccount(provider.connection, userTokenX))
      .amount;
    const userYBefore = (await getAccount(provider.connection, userTokenY))
      .amount;

    await program.methods
      .swapExactIn(inputAmount, inputIsX, userMinOutAmount)
      .accounts({
        userWallet: user.publicKey,
        userTokenXAcct: userTokenX,
        userTokenYAcct: userTokenY,
        vaultTokenXAcct: vaultTokenX,
        vaultTokenYAcct: vaultTokenY,
        tokenXMint: mintX,
        tokenYMint: mintY,
      })
      .rpc();

    const userXAfter = (await getAccount(provider.connection, userTokenX))
      .amount;
    const userYAfter = (await getAccount(provider.connection, userTokenY))
      .amount;

    // Y decreased by 1_000_000
    assert.ok(new anchor.BN(Number(userYBefore)).sub(new anchor.BN(Number(userYAfter))).eq(inputAmount));
    // X increased bx 950_000
    assert.ok(Number(userXAfter) > Number(userXBefore));
  });
});