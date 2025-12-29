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

  // TODO
})