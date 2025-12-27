import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";

describe("shared_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // CHANGE if your workspace program name differs (Anchor.toml / IDL name)
  const program = (anchor.workspace as any).AnchorProject as anchor.Program;

  const connection = provider.connection;
  const user1 = provider.wallet as anchor.Wallet;
  const user2 = anchor.web3.Keypair.generate();

  const LAMPORTS = anchor.web3.LAMPORTS_PER_SOL;

  async function airdrop(pubkey: anchor.web3.PublicKey, sol = 2) {
    const sig = await connection.requestAirdrop(pubkey, sol * LAMPORTS);
    await connection.confirmTransaction(sig, "confirmed");
  }

  let vaultPda: anchor.web3.PublicKey;

  before(async () => {
    await airdrop(user2.publicKey, 2);

    [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [user1.publicKey.toBuffer(), user2.publicKey.toBuffer()],
      program.programId
    );
  });

  it("create", async () => {
    await program.methods
      .create()
      .accounts({
        user1: user1.publicKey,
        user2: user2.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const bal = await connection.getBalance(vaultPda, "confirmed");
    assert.isAbove(bal, 0, "vault should exist (rent-exempt lamports)");
  });

  it("deposit (user1)", async () => {
    const before = await connection.getBalance(vaultPda, "confirmed");
    const amount = new anchor.BN(0.2 * LAMPORTS);

    await program.methods
      .deposit(user2.publicKey, amount) // matches deposit(ctx, user2: Pubkey, amount: u64)
      .accounts({
        user1: user1.publicKey,
        user2: user2.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const after = await connection.getBalance(vaultPda, "confirmed");
    assert.equal(after - before, amount.toNumber());
  });

  it("withdraw fails with only user1 signature", async () => {
    const amount = new anchor.BN(0.1 * LAMPORTS);

    let failed = false;
    try {
      await program.methods
        .withdraw(amount)
        .accounts({
          user1: user1.publicKey,
          user2: user2.publicKey,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        // only provider(user1) signs automatically; user2 missing
        .rpc();
    } catch {
      failed = true;
    }
    assert.isTrue(failed);
  });

  it("withdraw succeeds with both signatures", async () => {
    const beforeVault = await connection.getBalance(vaultPda, "confirmed");
    const beforeUser1 = await connection.getBalance(user1.publicKey, "confirmed");
    const amount = new anchor.BN(0.1 * LAMPORTS);

    await program.methods
      .withdraw(amount)
      .accounts({
        user1: user1.publicKey,
        user2: user2.publicKey,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user2]) // provider signs for user1; we add user2
      .rpc();

    const afterVault = await connection.getBalance(vaultPda, "confirmed");
    const afterUser1 = await connection.getBalance(user1.publicKey, "confirmed");

    assert.equal(beforeVault - afterVault, amount.toNumber());
    assert.isAbove(afterUser1, beforeUser1, "user1 should receive lamports");
  });
});
