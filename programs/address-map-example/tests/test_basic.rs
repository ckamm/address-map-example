use address_map_example::solana_address_lookup_table_instruction;
use program_test::*;
use solana_program::instruction::Instruction;
use solana_program_test::*;
use solana_sdk::{
    message::v0::AddressLookupTable, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transport::TransportError,
};

mod program_test;

async fn register(
    context: &TestContext,
    user: &Keypair,
    mint: Pubkey,
    token_account: Pubkey,
) -> Result<(Pubkey, Pubkey), TransportError> {
    let (registrar, bump) = Pubkey::find_program_address(
        &[
            b"registrar".as_ref(),
            &user.pubkey().to_bytes(),
            &mint.to_bytes(),
        ],
        &context.program_id,
    );
    let recent_slot = 0;
    let address_map = solana_address_lookup_table_instruction::derive_lookup_table_address(
        &registrar,
        recent_slot,
    )
    .0;

    let data = anchor_lang::InstructionData::data(&address_map_example::instruction::Register {
        bump,
        recent_slot,
    });

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

    context
        .solana
        .process_versioned_transaction(&instructions, Some(&[&signer]), None)
        .await?;

    Ok((registrar, address_map))
}

async fn balance(
    context: &TestContext,
    address_map: Pubkey,
    address_map_addresses: Vec<Pubkey>,
    expected: u64,
) -> Result<(), TransportError> {
    let data =
        anchor_lang::InstructionData::data(&address_map_example::instruction::Balance { expected });

    let mut accounts = anchor_lang::ToAccountMetas::to_account_metas(
        &address_map_example::accounts::Balance {},
        None,
    );
    for key in &address_map_addresses {
        accounts.push(anchor_lang::prelude::AccountMeta::new_readonly(*key, false));
    }

    let instructions = vec![Instruction {
        program_id: context.program_id,
        accounts,
        data,
    }];

    let address_lookup_table = AddressLookupTable {
        account_key: address_map,
        addresses: address_map_addresses,
    };

    context
        .solana
        .process_versioned_transaction(&instructions, None, Some(&vec![address_lookup_table]))
        .await
}

#[allow(unaligned_references)]
#[tokio::test]
async fn test_basic() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let mint = context.mints[0].pubkey.unwrap();
    let token1 = context.users[1].token_accounts[0];
    let token2 = context.users[2].token_accounts[0];

    let (_registrar, address_map) = register(&context, &context.users[1].key, mint, token1)
        .await
        .unwrap();
    // required to make the address map change live
    context.solana.advance_clock_by_slots(5).await;
    balance(&context, address_map, vec![token1], 1000000000000000000)
        .await
        .unwrap();

    register(&context, &context.users[1].key, mint, token2)
        .await
        .unwrap();
    context.solana.advance_clock_by_slots(5).await;
    balance(
        &context,
        address_map,
        vec![token1, token2],
        2000000000000000000,
    )
    .await
    .unwrap();

    Ok(())
}
