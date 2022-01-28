use anchor_spl::token::TokenAccount;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError, pubkey::Pubkey};
use std::borrow::BorrowMut;
use program_test::*;
use solana_program::instruction::Instruction;
use address_map_example::solana_address_lookup_table_instruction as solana_address_lookup_table_instruction;

mod program_test;

async fn register(context: &TestContext, user: &Keypair, mint: Pubkey, token_account: Pubkey)
    -> Result<(), TransportError> {
    let (registrar, bump) = Pubkey::find_program_address(
        &[
            b"registrar".as_ref(),
            &user.pubkey().to_bytes(),
            &mint.to_bytes(),
        ],
        &context.program_id,
    );
    let recent_slot = 0;
    let address_map = solana_address_lookup_table_instruction::derive_lookup_table_address(&registrar, recent_slot).0;

    let data = anchor_lang::InstructionData::data(
        &address_map_example::instruction::Register {
            bump,
            recent_slot,
        },
    );

    let accounts = anchor_lang::ToAccountMetas::to_account_metas(
        &address_map_example::accounts::Register {
            registrar,
            address_map,
            user: user.pubkey(),
            token_account,
            system_program: solana_sdk::system_program::id(),
            rent: solana_program::sysvar::rent::id(),
            address_lookup_table: solana_address_lookup_table_instruction::id(),
        },
        None,
    );

    let instructions = vec![Instruction {
        program_id: context.program_id,
        accounts,
        data,
    }];

    // clone the secrets
    let signer = Keypair::from_base58_string(&user.to_base58_string());

    context.solana
        .process_transaction(&instructions, Some(&[&signer]))
        .await
}

#[allow(unaligned_references)]
#[tokio::test]
async fn test_basic() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let payer = &context.users[0].key;
    let mint = context.mints[0].pubkey.unwrap();

    register(&context, &context.users[1].key, mint, context.users[1].token_accounts[0]).await.unwrap();

    Ok(())
}
