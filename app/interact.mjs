import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  AnchorProvider,
  Program,
  Wallet,
} from "@anchor-lang/core";

import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(scriptDir, "..");

const PROGRAM_ID = "HoFbEqxqKZ3LPxmeKq3XYfpSkQeTsi9PnAwWVMDNrqqC";
const EXPECTED_WALLET = "Fy3KndzKuAfnArtGMmyiCqGXQJdHgBkjcceuAiRuKgE9";

const idlPath = path.join(
  projectRoot,
  "target",
  "idl",
  "solana_counter.json"
);

const walletPath = path.join(
  os.homedir(),
  ".config",
  "solana",
  "devnet-wallet.json"
);

function fail(message) {
  throw new Error(message);
}

async function main() {
  const command = process.argv[2];

  if (!["init", "read", "increment"].includes(command)) {
    console.log("Usage: node app/interact.mjs <init|read|increment>");
    return;
  }

  if (!fs.existsSync(idlPath)) {
    fail(`IDL not found: ${idlPath}. Run anchor build first.`);
  }

  if (!fs.existsSync(walletPath)) {
    fail(`Devnet wallet not found: ${walletPath}`);
  }

  const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));

  if (idl.address !== PROGRAM_ID) {
    fail("The IDL program address does not match the expected program ID.");
  }

  // Load the local keypair without displaying its private key.
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  const payer = Keypair.fromSecretKey(Uint8Array.from(secret));

  if (payer.publicKey.toBase58() !== EXPECTED_WALLET) {
    fail(
      "The wallet file does not match the expected Devnet address. " +
      "Check your wallet configuration before signing transactions."
    );
  }

  const connection = new Connection(
    clusterApiUrl("devnet"),
    "confirmed"
  );

  const provider = new AnchorProvider(
    connection,
    new Wallet(payer),
    AnchorProvider.defaultOptions()
  );

  const program = new Program(idl, provider);

  // One PDA per wallet: ["counter", wallet public key].
  const [counterPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("counter"),
      payer.publicKey.toBuffer(),
    ],
    program.programId
  );

  const balance = await connection.getBalance(payer.publicKey);

  console.log("Cluster: Devnet");
  console.log("Wallet:", payer.publicKey.toBase58());
  console.log("Counter PDA:", counterPda.toBase58());
  console.log("Wallet balance:", balance / LAMPORTS_PER_SOL, "SOL");

  const existingAccount = await connection.getAccountInfo(counterPda);

  if (command === "init") {
    if (existingAccount) {
      fail("This wallet's counter PDA already exists. Use read or increment.");
    }

    const signature = await program.methods
      .initialize()
      .accounts({
        authority: payer.publicKey,
        counter: counterPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Counter initialized.");
    console.log("Transaction:", signature);
    console.log(
      `Explorer: https://explorer.solana.com/tx/${signature}?cluster=devnet`
    );
  }

  if (command === "increment") {
    if (!existingAccount) {
      fail("Counter not initialized. Run the init command first.");
    }

    const signature = await program.methods
      .increment()
      .accounts({
        counter: counterPda,
        authority: payer.publicKey,
      })
      .rpc();

    console.log("Counter increment submitted.");
    console.log("Transaction:", signature);
    console.log(
      `Explorer: https://explorer.solana.com/tx/${signature}?cluster=devnet`
    );
  }

  if (command === "read") {
    if (!existingAccount) {
      console.log("Counter not initialized yet.");
      console.log("Initialize it with: node app/interact.mjs init");
      return;
    }

    const state = await program.account.counter.fetch(counterPda);

    console.log("Counter count:", state.count.toString());
    console.log("Counter authority:", state.authority.toBase58());
  }
}

main().catch((error) => {
  console.error("Client error:", error.message ?? error);
  process.exitCode = 1;
});
