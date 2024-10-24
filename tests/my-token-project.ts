import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RWAPMemeToken } from "../target/types/rwap_meme_token";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("RWAPMemeToken", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.RwapMemeToken as Program<RWAPMemeToken>;
  
  // Global variables to store mint and token accounts
  let mint: PublicKey = null as unknown as PublicKey;
  let mintAuthority: Keypair = Keypair.generate();
  let freezeAuthority: Keypair = Keypair.generate();
  let admin: Keypair = Keypair.generate();
  let tokenAccount: PublicKey = null as unknown as PublicKey;
  let state: PublicKey = null as unknown as PublicKey;

  it("Initialize the token!", async () => {
    // Derive state account using seeds
    [state] = await PublicKey.findProgramAddress(
      [Buffer.from("state")],
      program.programId
    );

    // Create mint and associated token account
    mint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      freezeAuthority.publicKey,
      9 // Decimals
    );

    tokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      mintAuthority,
      mint,
      mintAuthority.publicKey
    );

    // Call the initialize_token function
    await program.methods
      .initializeToken()
      .accounts({
        mint,
        mintAuthority: mintAuthority.publicKey,
        freezeAuthority: freezeAuthority.publicKey,
        admin: admin.publicKey,
        state,
        to: tokenAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([admin, mintAuthority])
      .rpc();

    // Check if the state was initialized properly
    const stateAccount = await program.account.state.fetch(state);
    assert.isFalse(stateAccount.isPaused); // Contract should not be paused initially
    assert.equal(stateAccount.admin.toBase58(), admin.publicKey.toBase58()); // Admin should be set
  });

  it("Mint additional tokens", async () => {
    const mintAmount = new anchor.BN(10000000); // Mint 10 tokens (9 decimals)

    await program.methods
      .mintTokens(mintAmount)
      .accounts({
        mint,
        mintAuthority: mintAuthority.publicKey,
        to: tokenAccount,
        state,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([mintAuthority])
      .rpc();

    // Verify that tokens were minted
    const tokenAccountInfo = await getAccount(provider.connection, tokenAccount);
    assert.equal(Number(tokenAccountInfo.amount), 10000000); // Minted 10 tokens
  });

  it("Pause and unpause the contract", async () => {
    // Pause the contract
    await program.methods
      .pauseContract()
      .accounts({
        admin: admin.publicKey,
        state,
      })
      .signers([admin])
      .rpc();

    // Verify the contract is paused
    let stateAccount = await program.account.state.fetch(state);
    assert.isTrue(stateAccount.isPaused);

    // Unpause the contract
    await program.methods
      .unpauseContract()
      .accounts({
        admin: admin.publicKey,
        state,
      })
      .signers([admin])
      .rpc();

    // Verify the contract is unpaused
    stateAccount = await program.account.state.fetch(state);
    assert.isFalse(stateAccount.isPaused);
  });

  it("Burn tokens", async () => {
    const burnAmount = new anchor.BN(5000000); // Burn 5 tokens

    await program.methods
      .burnTokens(burnAmount)
      .accounts({
        mint,
        from: tokenAccount,
        authority: mintAuthority.publicKey,
        state,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([mintAuthority])
      .rpc();

    // Verify the tokens were burned
    const tokenAccountInfo = await getAccount(provider.connection, tokenAccount);
    assert.equal(Number(tokenAccountInfo.amount), 5000000); // Burned 5 tokens, 5 should remain
  });
});

// Helper functions for token creation
async function createMint(connection, payer, mintAuthority, freezeAuthority, decimals) {
  return await anchor.utils.token.createMint(
    connection,
    payer,
    mintAuthority,
    freezeAuthority,
    decimals
  );
}

async function createAssociatedTokenAccount(connection, payer, mint, owner) {
  return await anchor.utils.token.createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    owner
  );
}

async function getAccount(connection, tokenAccount) {
  return await anchor.utils.token.getAccount(connection, tokenAccount);
}
