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

fn derive_counter(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            solana_counter::constants::COUNTER_SEED,
            authority.as_ref(),
        ],
        &solana_counter::id(),
    )
    .0
}

fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = solana_counter::id();
    let payer = Keypair::new();
    let counter = derive_counter(&payer.pubkey());

    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/solana_counter.so"
    ));

    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    (svm, payer, counter)
}

fn submit(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> bool {
    // Ensure repeated instructions use distinct transaction blockhashes.
    svm.expire_blockhash();

    let blockhash = svm.latest_blockhash();
    let message =
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);

    let transaction =
        VersionedTransaction::try_new(VersionedMessage::Legacy(message), &[payer])
            .unwrap();

    svm.send_transaction(transaction).is_ok()
}

fn initialize_instruction(
    authority: &Keypair,
    counter: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        solana_counter::id(),
        &solana_counter::instruction::Initialize {}.data(),
        solana_counter::accounts::Initialize {
            authority: authority.pubkey(),
            counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn increment_instruction(
    counter: Pubkey,
    authority: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        solana_counter::id(),
        &solana_counter::instruction::Increment {}.data(),
        solana_counter::accounts::Increment {
            counter,
            authority,
        }
        .to_account_metas(None),
    )
}

fn read_counter(
    svm: &LiteSVM,
    counter: Pubkey,
) -> solana_counter::state::Counter {
    let account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &account.data;

    solana_counter::state::Counter::try_deserialize(&mut data).unwrap()
}

#[test]
fn test_duplicate_initialization_is_rejected() {
    let (mut svm, payer, counter) = setup();

    assert!(submit(
        &mut svm,
        &payer,
        initialize_instruction(&payer, counter),
    ));

    assert!(!submit(
        &mut svm,
        &payer,
        initialize_instruction(&payer, counter),
    ));

    let state = read_counter(&svm, counter);
    assert_eq!(state.count, 0);
    assert_eq!(state.authority, payer.pubkey());
}

#[test]
fn test_unauthorized_increment_is_rejected() {
    let (mut svm, payer, counter) = setup();

    assert!(submit(
        &mut svm,
        &payer,
        initialize_instruction(&payer, counter),
    ));

    let unauthorized = Keypair::new();
    svm.airdrop(&unauthorized.pubkey(), 100_000_000).unwrap();

    assert!(!submit(
        &mut svm,
        &unauthorized,
        increment_instruction(counter, unauthorized.pubkey()),
    ));

    let state = read_counter(&svm, counter);
    assert_eq!(state.count, 0);
    assert_eq!(state.authority, payer.pubkey());
}

#[test]
fn test_counter_overflow_is_rejected() {
    let (mut svm, payer, counter) = setup();

    assert!(submit(
        &mut svm,
        &payer,
        initialize_instruction(&payer, counter),
    ));

    for _ in 0..solana_counter::constants::MAX_COUNT {
        assert!(submit(
            &mut svm,
            &payer,
            increment_instruction(counter, payer.pubkey()),
        ));
    }

    let state = read_counter(&svm, counter);
    assert_eq!(state.count, solana_counter::constants::MAX_COUNT);

    assert!(!submit(
        &mut svm,
        &payer,
        increment_instruction(counter, payer.pubkey()),
    ));

    let state = read_counter(&svm, counter);
    assert_eq!(state.count, solana_counter::constants::MAX_COUNT);
}

#[test]
fn test_each_wallet_gets_a_distinct_counter() {
    let (mut svm, alice, alice_counter) = setup();

    let bob = Keypair::new();
    svm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    let bob_counter = derive_counter(&bob.pubkey());

    assert_ne!(alice_counter, bob_counter);

    assert!(submit(
        &mut svm,
        &alice,
        initialize_instruction(&alice, alice_counter),
    ));

    assert!(submit(
        &mut svm,
        &bob,
        initialize_instruction(&bob, bob_counter),
    ));

    let alice_state = read_counter(&svm, alice_counter);
    let bob_state = read_counter(&svm, bob_counter);

    assert_eq!(alice_state.count, 0);
    assert_eq!(bob_state.count, 0);

    assert_eq!(alice_state.authority, alice.pubkey());
    assert_eq!(bob_state.authority, bob.pubkey());

    // Alice can update her counter without changing Bob's counter.
    assert!(submit(
        &mut svm,
        &alice,
        increment_instruction(alice_counter, alice.pubkey()),
    ));

    assert_eq!(read_counter(&svm, alice_counter).count, 1);
    assert_eq!(read_counter(&svm, bob_counter).count, 0);
}
