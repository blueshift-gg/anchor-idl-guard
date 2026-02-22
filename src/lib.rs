#[macro_export]
macro_rules! entrypoint {
    ($pubkey:literal) => {
        $crate::entrypoint!([$pubkey]);
    };
    ([$($pubkey:literal),+ $(,)?]) => {
        #[cfg(not(feature = "no-entrypoint"))]
        compile_error!("anchor_safe_idl::entrypoint! requires the `no-entrypoint` feature to disable Anchor's default entrypoint. Add `default = [\"no-entrypoint\"]` to your [features] in Cargo.toml.");

        #[cfg(not(feature = "cpi"))]
        pub const IDL_AUTHORITIES: &[anchor_lang::solana_program::pubkey::Pubkey] =
            &[$(anchor_lang::pubkey!($pubkey)),+];

        #[cfg(not(feature = "cpi"))]
        anchor_lang::solana_program::entrypoint!(__safe_idl_gated_entry);

        #[cfg(not(feature = "cpi"))]
        fn __safe_idl_gated_entry<'a>(
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            accounts: &'a [anchor_lang::solana_program::account_info::AccountInfo<'a>],
            data: &[u8],
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            // IDL CreateBuffer: 8-byte IDL tag + variant 0x01
            if data.len() >= 9 && data[..8] == [0x40, 0xf4, 0xbc, 0x78, 0xa7, 0xe9, 0x69, 0x0a] {
                match data[8] {
                    // Create IDL
                    0 => {
                        #[cfg(not(feature = "no-log-ix-name"))]
                        anchor_lang::solana_program::msg!("Instruction: SafeIDLCreateAccount");
                        let authority = accounts
                            .first()
                            .ok_or(anchor_lang::solana_program::program_error::ProgramError::NotEnoughAccountKeys)?;
                        if !IDL_AUTHORITIES.iter().any(|k| k == authority.key) {
                            return Err(anchor_lang::solana_program::program_error::ProgramError::MissingRequiredSignature);
                        }
                    }
                    // Create buffer
                    1 => {
                        let authority = accounts
                            .get(1)
                            .ok_or(anchor_lang::solana_program::program_error::ProgramError::NotEnoughAccountKeys)?;
                        if !IDL_AUTHORITIES.iter().any(|k| k == authority.key) {
                            return Err(anchor_lang::solana_program::program_error::ProgramError::MissingRequiredSignature);
                        }
                        #[cfg(not(feature = "no-log-ix-name"))]
                        anchor_lang::solana_program::msg!("Instruction: SafeIDLCreateBuffer");
                    }
                    _ => ()
                }
            }
            entry(program_id, accounts, data)
        }
    };
}
