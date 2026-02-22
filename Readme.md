# Anchor IDL Guard

Every version of Anchor since 0.3.0 is subject to several minor attack vectors in the IDL instruction handlers that allow an attacker to:

- Create a program-owned IDLBuffer account and inject arbitrary data at any offset ≥44 bytes.
- Permissionlessly take over *any* program-owned account with a discriminator of 8 leading zero bytes that is ≥44 bytes in length.
- Close these accounts to reclaim their lamports.
- Grief the program owner and trick users by stealing the program's canonical IDL account.

While these will be addressed in the future by the use of the Program Metadata Program, many users are yet to upgrade to modern versions of Anchor, and thus will remain susceptible to these attacks.

In order to prevent this in current and legacy versions of anchor, `anchor_idl_guard::entrypoint` provides a drop-in replacement for Anchor's default entrypoint macro that gates access to the `IdlCreate` and `IdlCreateBuffer` instructions behind a simple whitelist.

## Attack Vectors

### 1. `IdlCreateBuffer` Account Takeover and LoF

As Anchor's `IdlCreateBuffer` instruction writes a buffer authority into any account passed to it **without doing any signer checks**. An attacker can target any program-owned account that:

- Has ≥44 bytes of data (8-byte discriminator + 32-byte authority + 4-byte buffer length), and
- 8 leading zero bytes (common in `AccountInfo`, `UncheckedAccount`, and misused `zero-copy` and `zero` constraint account layouts)

To take over any such account, the attacker simply calls `IdlCreateBuffer` on the victim account, instantiating themselves as the buffer authority, enabling them to subsequently call `IdlCloseAccount` to drain all lamports from the account, resulting in loss of funds for any accounts matching the above criteria.

### 2. `IdlCreateBuffer` Arbitrary Data Injection

By simply creating a new account with `system_program::create_account` and assigning the owner to an Anchor program, an attacker is also able to inject abritrary data into this account at any offset ≥44 and later close and reclaim the account by the same method described above.

### 3. `IdlCreateAccount` Hijack

Anchor's `IdlCreateAccount` instruction creates the canonical IDL account (derived via `create_account_with_seed` at `PDA("anchor:idl")`). Because there is no signer authority check, anyone can invoke this instruction to create or take ownership of the program's IDL account. This lets an attacker publish a spoofed IDL, which could be used to phish users with a fake interface to the program.

## How It Works

1. `anchor_idl_guard` exports a single `entrypoint!` macro that replaces Anchor's default program entrypoint with a gated version. 
2. Before forwarding any instruction to Anchor's `entry()` function, it inspects the instruction data for the IDL discriminator (`0x40f4bc78a7e9690a`). 
3. If an IDL instruction is detected, it checks the secondary discriminator for the two offeding instructions:
   1.  `0x00` (`IdlCreateAccount`), and 
   2.  `0x01` (`IdlCreateBuffer`)
4. The signer of the instruction is matched against the whitelist of authorized public keys.
5. If a match is not found, the instruction will error out with `MissingRequiredSignature`.

## Usage

### 1. Add the dependency

```toml
[dependencies]
anchor-idl-guard = "0.1.0"
```

### 2. Enable the `no-entrypoint` feature

Our `entrypoint!` macro replaces Anchor's default entrypoint, so you must disable it by activating the `no-entrypoint` feature flag:

```toml
[features]
default = ["no-entrypoint"]
no-entrypoint = []
```

### 3. Add the macro to your program

Pass your program's upgrade authority (or any trusted key) as a string literal:

```rust
use anchor_lang::prelude::*;

declare_id!("YourProgramId111111111111111111111111111111");

anchor_idl_guard::entrypoint!("YourUpgradeAuthority111111111111111111111111");

#[program]
pub mod my_program {
    use super::*;
    // ...
}
```

You can also pass in multiple authorities for greater flexibility:

```rust
entrypoint!([
    "Authority1111111111111111111111111111111111", "Authority2222222222222222222222222222222222222"
]);
```

### Feature Flags

| Feature | Description |
|---|---|
| `no-entrypoint` | **Required.** Disables Anchor's default entrypoint so the IDL guard entrypoint can replace it. |
| `cpi` | Disables the custom entrypoint when the program is used as a CPI dependency. |
| `no-log-ix-name` | Suppresses `msg!` logging of gated instruction names for compute savings. |


## Alternatives

It is also possible to mitigate IDL-based attacks by simply enabling the `no-idl` feature in your Cargo.toml, however doing so will also prevent you from being able to publish your IDL which may not be your intended outcome.