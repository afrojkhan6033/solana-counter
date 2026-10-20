# Solana Counter

A Solana program written in Rust using Anchor. This project demonstrates wallet-specific Program Derived Addresses (PDAs), account initialization, authority validation, bounded counter updates, and Devnet interaction.

## Features

- One counter account per wallet using a PDA
- Wallet-authorized initialization and increment instructions
- Maximum counter value of 10
- Account validation and error handling
- Rust integration tests using LiteSVM
- Node.js client for interacting with Solana Devnet

## Tech Stack

- Rust
- Solana
- Anchor
- LiteSVM
- Node.js
- @anchor-lang/core
- @solana/web3.js

## Deployed Program

- Network: Solana Devnet
- Program ID: `HoFbEqxqKZ3LPxmeKq3XYfpSkQeTsi9PnAwWVMDNrqqC`

## Build

Run `anchor build` from the project root.

## Tests

Run `cargo test -p solana-counter --tests` from the project root.

## Devnet Client

Install dependencies using `cd app && npm install`.

The client expects your local wallet file at `~/.config/solana/devnet-wallet.json`. Never commit this file or share its private key.

Initialize your counter using `node app/interact.mjs init`.

Read your counter using `node app/interact.mjs read`.

Increment your counter using `node app/interact.mjs increment`.

Each wallet has its own counter PDA. Initialize it only once. The counter rejects increments after reaching 10.

## Security

This project is for learning and experimentation and has not been audited for production use. Never commit wallet JSON files, private keys, recovery phrases, or secret environment variables.
