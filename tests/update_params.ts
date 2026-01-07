import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorProject } from "../target/types/anchor_project";
import {
  createMint,
  createAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Update Params", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorProject as Program<AnchorProject>;
  const user = provider.wallet as anchor.Wallet;

  let mintX: anchor.web3.PublicKey;
  let mintY: anchor.web3.PublicKey;
  let stateAccountPDA: anchor.web3.PublicKey;

  const SPREAD_BPS = new anchor.BN(500); // 5%
  const PRICE_SCALE = new anchor.BN(1_000_000);
  const INITIAL_PRICE = PRICE_SCALE; // 1:1

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

    // 2. Derive State PDA
    const [statePDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state"), mintX.toBuffer(), mintY.toBuffer()],
      program.programId
    );
    stateAccountPDA = statePDA;

    // 3. Initialize Program
    await program.methods
      .init(SPREAD_BPS)
      .accounts({
        user: user.publicKey,
        tokenXMint: mintX,
        tokenYMint: mintY,
      })
      .rpc();
  });

  it("Updates global parameters", async () => {
    try {
      // Update price to 1:1 using the new instruction
      await program.methods
        .updateParams(INITIAL_PRICE, null)
        .accounts({
          tokenXMint: mintX,
          tokenYMint: mintY,
        })
        .rpc();

      const state = await program.account.stateAccount.fetch(stateAccountPDA);
      assert.ok(state.xToYScaledPrice.eq(INITIAL_PRICE));
    } catch (e) {
      console.error(e);
      throw e;
    }
  });
});