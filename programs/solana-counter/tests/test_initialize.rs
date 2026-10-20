use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize() {
    let program_id = solana_counter::id();
    let payer = Keypair::new();

    let counter = Pubkey::find_program_address(
        &[
            solana_counter::constants::COUNTER_SEED,
            payer.pubkey().as_ref(),
        ],
        &program_id,
    )
    .0;

    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/solana_counter.so"
    ));

    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &solana_counter::instruction::Initialize {}.data(),
        solana_counter::accounts::Initialize {
            authority: payer.pubkey(),
            counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let message =
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::Legacy(message), &[&payer])
            .unwrap();

    assert!(svm.send_transaction(transaction).is_ok());

    let account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &account.data;

    let state =
        solana_counter::state::Counter::try_deserialize(&mut data).unwrap();

    assert_eq!(state.count, 0);
    assert_eq!(state.authority, payer.pubkey());

    // Increment the same wallet's counter.
    svm.expire_blockhash();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &solana_counter::instruction::Increment {}.data(),
        solana_counter::accounts::Increment {
            counter,
            authority: payer.pubkey(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let message =
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::Legacy(message), &[&payer])
            .unwrap();

    assert!(svm.send_transaction(transaction).is_ok());

    let account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &account.data;

    let state =
        solana_counter::state::Counter::try_deserialize(&mut data).unwrap();

    assert_eq!(state.count, 1);
    assert_eq!(state.authority, payer.pubkey());
}
