//! Various assertions.

/// Runs a block, returning a [anchor_lang::prelude::Result<()>].
#[macro_export]
macro_rules! test_assertion {
    ($body: block) => {
        (|| -> crate::Result<()> {
            $body
            Ok(())
        })()
    };
}

/// Asserts that the given assertion block does not throw any errors.
///
/// Recommended for use in tests only.
#[macro_export]
macro_rules! assert_does_not_throw {
    ($body: block $(,)?) => {
        assert!($crate::test_assertion!($body).is_ok())
    };
}

/// Asserts that the given assertion block throws a specific error.
///
/// Recommended for use in tests only.
#[macro_export]
macro_rules! assert_throws {
    ($body: block, $right: expr $(,)?) => {
        assert_eq!(
            $crate::IntoCmpError::into_cmp_error($crate::test_assertion!($body).err()),
            $crate::IntoCmpError::into_cmp_error(Some(::anchor_lang::prelude::error!($right)))
        )
    };
}

/// Formats an error as a `&str`.
///
/// # Example
///
/// ```
/// # use anchor_lang::prelude::*;
/// #[error_code]
/// pub enum ErrorCode {
///   #[msg("This is my error")]
///   MyError
/// }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// assert_eq!(format_err!(ErrorCode::MyError), "MyError: This is my error");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! format_err {
    ($err: expr) => {
        &*format!("{:?}: {}", $err, $err)
    };
}

/// Returns the given error as a program error.
///
/// # Example
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[error_code]
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let fail = false;
/// if fail {
///     return program_err!(MyError);
/// }
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! program_err {
    ($error:ident $(,)?) => {
        Err(crate::ErrorCode::$error.into())
    };
}

/// Logs where in the code the macro was invoked.
#[macro_export]
macro_rules! log_code_location {
    () => {
        msg!("Error thrown at {}:{}", file!(), line!());
    };
}

/// Unwraps a block which returns an [Option].
///
/// This is useful for running checked math expressions within a function which returns [Result].
///
/// # Examples
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let result = unwrap_opt_block!({
///     let one: u64 = 1;
///     let three: u64 = 3;
///     let four = one.checked_add(three)?;
///     four.checked_add(3)
/// });
/// assert_eq!(result, 7);
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_opt_block {
    ($body:block $($arg:tt)*) => {
        $crate::unwrap_opt!(
            #[allow(clippy::redundant_closure_call)]
            (|| { $body } )() $($arg)*)
    };
}

/// Unwraps the result of a block of checked integer math.
///
/// # Example
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let result = unwrap_checked!({
///   let one: u64 = 1;
///   let three: u64 = 3;
///   one.checked_add(three)
/// });
/// assert_eq!(result, 4);
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_checked {
    ($body:block $(,)?) => {
        $crate::unwrap_opt_block!($body, $crate::VipersError::IntegerOverflow)
    };
}

/// Throws an error.
///
/// # Example
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[error_code]
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let fail = false;
/// if fail {
///     throw_err!(MyError);
/// }
/// Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! throw_err {
    ($error:ident $(,)?) => {
        $crate::throw_err!(crate::ErrorCode::$error);
    };
    ($error:expr $(,)?) => {
        $crate::log_code_location!();
        return Err(::anchor_lang::prelude::error!($error));
    };
}

/// Asserts that the ATA is the one of the given owner/mint.
///
/// Warning: this uses a lot of compute units due to the need to generate a PDA.
/// It is recommended to cache this value.
#[cfg(feature = "spl-associated-token-account")]
#[macro_export]
#[deprecated(
    since = "1.5.6",
    note = "This uses a lot of compute units due to the need to generate a PDA. This is also not a valid way to check ownership since ATAs can be transferred. Use assert_keys_eq on the token account owner instead."
)]
macro_rules! assert_ata {
    ($ata: expr, $owner: expr, $mint: expr $(,)?) => {
        $crate::assert_ata!($ata, $owner, $mint, "ata mismatch")
    };
    ($ata: expr, $owner: expr, $mint: expr, $msg: expr $(,)?) => {{
        let __ata = $crate::AsKeyRef::as_key_ref(&$ata);
        let __real_ata = $crate::ata::get_associated_token_address(
            $crate::AsKeyRef::as_key_ref(&$owner),
            $crate::AsKeyRef::as_key_ref(&$mint),
        );
        if &__real_ata != __ata {
            msg!(
                "ATA mismatch: {}: {} (left) != {} (right)",
                $msg,
                __ata,
                __real_ata
            );
            msg!("Owner: {}", $crate::AsKeyRef::as_key_ref(&$owner));
            msg!("Mint: {}", $crate::AsKeyRef::as_key_ref(&$mint));
            $crate::throw_err!($crate::VipersError::ATAMismatch);
        }
    }};
}

/// Asserts that the given [anchor_spl::token::TokenAccount] is an associated token account.
///
/// Warning: this uses a lot of compute units due to the need to generate a PDA.
/// Use this macro sparingly.
#[macro_export]
macro_rules! assert_is_ata {
    ($ata: expr $(,)?) => {
        $crate::assert_ata!($ata, "invalid ata")
    };
    ($ata: expr, $msg: expr $(,)?) => {{
        $crate::assert_owner!($ata, token, "ATA not owned by token program");
        let __owner = $ata.owner;
        let __mint = $ata.mint;
        let __ata = anchor_lang::Key::key(&$ata);
        let __real_ata =
            $crate::spl_associated_token_account::get_associated_token_address(&__owner, &__mint);
        if __real_ata != __ata {
            msg!(
                "Invalid ATA: {}: {} (left) != {} (right)",
                $msg,
                __ata,
                __real_ata
            );
            msg!("Owner: {}", __owner);
            msg!("Mint: {}", __mint);
            $crate::throw_err!($crate::VipersError::InvalidATA);
        }
    }};
}

/// Asserts that an account is owned by the given program.
///
/// As of Anchor 0.15, Anchor handles this for you automatically.
/// You should not need to use this.
#[macro_export]
#[deprecated(
    since = "1.5.6",
    note = "As of Anchor 0.15, Anchor handles this for you automatically."
)]
macro_rules! assert_owner {
    ($program_account: expr, $owner: expr $(,)?) => {
        $crate::assert_owner!($program_account, $owner, "owner mismatch")
    };
    ($program_account: expr, $owner: ident $(,)?) => {
        $crate::assert_owner!($program_account, $owner);
    };
    ($program_account: expr, $owner: ident, $msg: expr $(,)?) => {
        let __program_id = $crate::program_ids::$owner::ID;
        $crate::assert_owner!($program_account, $owner, $msg);
    };
    ($program_account: expr, $owner: expr, $msg: expr $(,)?) => {{
        let __program_account =
            anchor_lang::ToAccountInfo::to_account_info(&$program_account).owner;
        let __owner = $crate::AsKeyRef::as_key_ref(&$owner);
        if __program_account != __owner {
            msg!(
                "Owner mismatch: {}: expected {}, got {}",
                $msg,
                __program_account,
                __owner
            );
            return Err($crate::VipersError::OwnerMismatch.into());
        }
    }};
}

/// Asserts that two accounts share the same key.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[error_code]
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = anchor_lang::solana_program::sysvar::clock::ID;
/// let two = anchor_lang::solana_program::system_program::ID;
/// assert_keys_eq!(one, two); // throws an error
/// Ok(())
/// # }
/// ```
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[error_code]
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = anchor_lang::solana_program::sysvar::clock::ID;
/// let two = anchor_lang::solana_program::system_program::ID;
/// assert_keys_eq!(one, two, "invalid"); // throws an error
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! assert_keys_eq {
    ($account_a: expr, $account_b: expr $(,)?) => {
        $crate::assert_keys_eq!($account_a, $account_b, $crate::VipersError::KeyMismatch);
    };
    ($account_a: expr, $account_b: expr, $err_code: ident $(,)?) => {
        $crate::assert_keys_eq!($account_a, $account_b, crate::ErrorCode::$err_code);
    };
    ($account_a: expr, $account_b: expr, $msg: literal $(,)?) => {
        $crate::assert_keys_eq!(
            $account_a,
            $account_b,
            $crate::VipersError::KeyMismatch,
            &*format!("Key mismatch: {}", $msg),
        );
    };
    ($account_a: expr, $account_b: expr, $err: expr $(,)?) => {
        $crate::assert_keys_eq!($account_a, $account_b, $err, $crate::format_err!($err));
    };
    ($account_a: expr, $account_b: expr, $err: expr, $msg: expr $(,)?) => {{
        let __key_a = &$account_a;
        let __key_b = &$account_b;
        let __account_a = $crate::AsKeyRef::as_key_ref(__key_a);
        let __account_b = $crate::AsKeyRef::as_key_ref(__key_b);
        if __account_a != __account_b {
            msg!($msg);
            msg!(stringify!($account_a != $account_b));
            msg!("Left: {}", __account_a);
            msg!("Right: {}", __account_b);
            $crate::throw_err!($err);
        }
    }};
}

/// Asserts that a token account is "zero".
///
/// This means that:
/// - the `amount` is zero
/// - the `delegate` is `None`
/// - the `close_authority` is `None`
///
/// This is useful for checking to see that a bad actor cannot
/// modify PDA-owned token accounts.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate seagull_vipers;
/// # use anchor_lang::prelude::*;
/// # use anchor_spl::token::*;
/// #[error_code]
/// pub enum ErrorCode { MyError }
///
/// # fn main() {
///
/// let mut zero_account = spl_token::state::Account::default();
/// assert_does_not_throw!({
///   assert_is_zero_token_account!(zero_account);
/// });
///
/// let mut non_zero_account = spl_token::state::Account::default();
/// non_zero_account.amount = 10;
/// assert_throws!({
///   assert_is_zero_token_account!(non_zero_account);
/// }, seagull_vipers::VipersError::TokenAccountIsNonZero);
///
/// non_zero_account = spl_token::state::Account::default();
/// non_zero_account.delegate = spl_token::ID.into();
/// assert_throws!({
///   assert_is_zero_token_account!(non_zero_account);
/// }, seagull_vipers::VipersError::TokenAccountIsNonZero);
///
/// non_zero_account = spl_token::state::Account::default();
/// non_zero_account.close_authority = spl_token::ID.into();
/// assert_throws!({
///   assert_is_zero_token_account!(non_zero_account);
/// }, seagull_vipers::VipersError::TokenAccountIsNonZero);
/// # }
/// ```
#[macro_export]
macro_rules! assert_is_zero_token_account {
    ($token_account: expr $(,)?) => {
        $crate::assert_is_zero_token_account!(
            $token_account,
            $crate::VipersError::TokenAccountIsNonZero
        );
    };
    ($token_account: expr, $err_code: ident $(,)?) => {
        $crate::assert_is_zero_token_account!($token_account, crate::ErrorCode::$err_code);
    };
    ($token_account: expr, $msg: literal $(,)?) => {
        $crate::assert_is_zero_token_account!(
            $token_account,
            $crate::VipersError::TokenAccountIsNonZero,
            &*format!("Token account is non-zero: {}", $msg),
        );
    };
    ($token_account: expr, $err: expr $(,)?) => {
        $crate::assert_is_zero_token_account!($token_account, $err, $crate::format_err!($err));
    };
    ($token_account: expr, $err: expr, $msg: expr $(,)?) => {{
        $crate::invariant!(
            $token_account.amount == 0
                && $token_account.delegate.is_none()
                && $token_account.close_authority.is_none(),
            $err,
            $msg
        );
    }};
}

/// Asserts that two accounts do not share the same key.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # impl From<ErrorCode> for ProgramError { fn from(code: ErrorCode) -> Self { ProgramError::Custom(10) } }
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = Pubkey::default();
/// let two = Pubkey::default();
/// assert_keys_neq!(one, two); // throws an error
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! assert_keys_neq {
    ($account_a: expr, $account_b: expr $(,)?) => {
        $crate::assert_keys_neq!(
            $account_a,
            $account_b,
            $crate::VipersError::KeysMustNotMatch
        );
    };
    ($account_a: expr, $account_b: expr, $err_code: ident $(,)?) => {
        $crate::assert_keys_neq!($account_a, $account_b, crate::ErrorCode::$err_code);
    };
    ($account_a: expr, $account_b: expr, $msg: literal $(,)?) => {
        $crate::assert_keys_neq!(
            $account_a,
            $account_b,
            $crate::VipersError::KeysMustNotMatch,
            &*format!("Keys must not match: {}", $msg),
        );
    };
    ($account_a: expr, $account_b: expr, $err: expr $(,)?) => {
        $crate::assert_keys_neq!($account_a, $account_b, $err, $crate::format_err!($err));
    };
    ($account_a: expr, $account_b: expr, $err: expr, $msg: expr $(,)?) => {{
        let __key_a = &$account_a;
        let __key_b = &$account_b;
        let __account_a = $crate::AsKeyRef::as_key_ref(__key_a);
        let __account_b = $crate::AsKeyRef::as_key_ref(__key_b);
        if __account_a == __account_b {
            msg!($msg);
            msg!(stringify!($account_a == $account_b));
            msg!("Left: {}", __account_a);
            msg!("Right: {}", __account_b);
            $crate::throw_err!($err);
        }
    }};
}

/// Ensures an [Option] can be unwrapped, otherwise returns the error.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # impl From<ErrorCode> for ProgramError { fn from(code: ErrorCode) -> Self { ProgramError::Custom(10) } }
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = 1_u64;
/// let two = 2_u64;
/// let my_value = unwrap_or_err!(one.checked_sub(2), MyError); // throws an error
/// Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_or_err {
    ($option:expr, $error:ident $(,)?) => {
        $option.ok_or_else(|| -> ProgramError { crate::ErrorCode::$error.into() })?
    };
}

/// Unwraps the result of a checked integer operation.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = 1_u64;
/// let two = 2_u64;
/// let my_value = unwrap_int!(one.checked_sub(2)); // returns an error
/// Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_int {
    ($option:expr $(,)?) => {
        $crate::unwrap_opt!($option, $crate::VipersError::IntegerOverflow)
    };
}

/// Unwraps a bump seed.
///
/// # Example
///
/// ```ignore
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// fn my_instruction(ctx: Context<MyAccounts>) -> Result<()> {
///     let my_data = &mut ctx.accounts.my_data
///     my_data.bump = unwrap_bump!(ctx, "my_data");
///     Ok(())
/// }
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_bump {
    ($ctx:expr, $bump:literal $(,)?) => {
        *$crate::unwrap_opt!(
            $ctx.bumps.get($bump),
            $crate::VipersError::UnknownBump,
            format!("Unknown bump: {}", $bump)
        )
    };
}

/// Tries to unwrap the [Result], otherwise returns the error
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[error_code]
/// # pub enum ErrorCode { MyError }
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// fn function_returning_result() -> Result<u64> {
///     err!(ErrorCode::MyError)
/// }
///
/// let my_value = try_or_err!(function_returning_result(), MyError);
/// # Ok(()) }
/// ```
#[macro_export]
macro_rules! try_or_err {
    ($result:expr, $error:ident $(,)?) => {
        $result.or_else(|_| -> ::anchor_lang::Result<_> { ::anchor_lang::prelude::err!($error) })?
    };
}

/// Asserts that an invariant holds, otherwise logs the given message.
/// This is a drop-in replacement for `require!`.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// invariant!(1 == 2, "incorrect");
/// # Ok(()) }
/// ```
///
/// Invariants do not throw if they pass.
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// invariant!(1 == 1, "won't throw");
/// # Ok(()) }
/// ```
///
/// Error codes are optional:
///
/// ```
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// invariant!(1 == 1);
/// # Ok(()) }
/// ```
///
/// You may also use a crate ErrorCode:
///
/// ```
/// # #[macro_use] extern crate seagull_vipers;
/// # use anchor_lang::prelude::*;
/// #[error_code]
/// pub enum ErrorCode { MyError }
///
/// # fn main() {
/// assert_does_not_throw!({
///   invariant!(1 == 1, MyError);
/// });
/// assert_throws!({
///   invariant!(1 == 2);
/// }, seagull_vipers::VipersError::InvariantFailed);
/// assert_throws!({
///   invariant!(1 == 2, "this is stupid");
/// }, seagull_vipers::VipersError::InvariantFailed);
/// assert_throws!({
///   invariant!(1 == 2, MyError);
/// }, ErrorCode::MyError);
/// assert_throws!({
///   invariant!(1 == 2, MyError);
/// }, ErrorCode::MyError);
/// assert_throws!({
///   invariant!(1 == 2, MyError, "this is wack");
/// }, ErrorCode::MyError);
/// # }
/// ```
#[macro_export]
macro_rules! invariant {
    ($invariant: expr $(,)?) => {
        $crate::invariant!($invariant, $crate::VipersError::InvariantFailed);
    };
    ($invariant: expr, $err_code: ident $(,)?) => {
        $crate::invariant!($invariant, crate::ErrorCode::$err_code);
    };
    ($invariant: expr, $err_code: ident, $msg: expr $(,)?) => {
        $crate::invariant!($invariant, crate::ErrorCode::$err_code, $msg);
    };
    ($invariant: expr, $msg: literal $(,)?) => {
        $crate::invariant!(
            $invariant,
            $crate::VipersError::InvariantFailed,
            &*format!("Invariant failed: {}", $msg)
        );
    };
    ($invariant:expr, $err:expr $(,)?) => {
        $crate::invariant!($invariant, $err, $crate::format_err!($err));
    };
    ($invariant:expr, $err:expr, $msg: expr $(,)?) => {{
        if !($invariant) {
            msg!($msg);
            msg!(stringify!($invariant));
            $crate::throw_err!($err);
        }
    }};
}

/// Attempts to unwrap an [Option], and if it fails, prints an error.
///
/// # Example
///
/// ```should_panic
/// # use anchor_lang::prelude::*;
/// # #[macro_use] extern crate seagull_vipers; fn main() -> Result<()> {
/// let one = 1_u64;
/// let two = 2_u64;
/// let my_value = unwrap_opt!(one.checked_sub(2), "cannot do this"); // returns an error
/// # Ok(()) }
/// ```
#[macro_export]
macro_rules! unwrap_opt {
    ($option: expr $(,)?) => {
        $crate::unwrap_opt!(
            $option,
            $crate::VipersError::OptionUnwrapFailed,
            $crate::format_err!($crate::VipersError::OptionUnwrapFailed)
        )
    };
    ($option: expr, $err_code: ident $(,)?) => {
        $crate::unwrap_opt!($option, crate::ErrorCode::$err_code)
    };
    ($option: expr, $msg: literal $(,)?) => {
        $crate::unwrap_opt!($option, $crate::VipersError::OptionUnwrapFailed, $msg)
    };
    ($option:expr, $err:expr $(,)?) => {
        $crate::unwrap_opt!($option, $err, $crate::format_err!($err))
    };
    ($option:expr, $err:expr, $msg: expr $(,)?) => {
        $option.ok_or_else(|| -> anchor_lang::error::Error {
            msg!("Option unwrap failed: {:?}", $err);
            msg!(stringify!($option));
            $crate::log_code_location!();
            anchor_lang::prelude::error!($err)
        })?
    };
}

/// Asserts that two accounts share the same key.
///
/// Deprecated in favor of [assert_keys_eq].
#[deprecated]
#[macro_export]
macro_rules! assert_keys {
    ($account_a: expr, $account_b: expr $(,)?) => {
        $crate::assert_keys_eq!($account_a, $account_b, "key mismatch")
    };
    ($account_a: expr, $account_b: expr, $msg: expr $(,)?) => {
        $crate::assert_keys_eq!($account_a, $account_b, $msg)
    };
}
