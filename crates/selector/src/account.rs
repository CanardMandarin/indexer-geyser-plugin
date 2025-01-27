use hashbrown::HashSet;
use indexer_rabbitmq::geyser::StartupType;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Account as TokenAccount;

use crate::{config::Accounts, Error, Heuristic, Result};

/// Abstraction over a Solana account container
#[allow(clippy::module_name_repetitions)]
pub trait AccountInfo {
    /// The bytes representing this account's owner public key
    fn owner(&self) -> &[u8];

    /// The bytes representing this account's public key
    fn pubkey(&self) -> &[u8];

    /// The data contained in this account
    fn data(&self) -> &[u8];
}

#[cfg(feature = "solana-geyser-plugin-interface")]
impl<'a> AccountInfo
    for solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaAccountInfo<'a>
{
    #[inline]
    fn owner(&self) -> &[u8] {
        self.owner
    }

    #[inline]
    fn pubkey(&self) -> &[u8] {
        self.pubkey
    }

    #[inline]
    fn data(&self) -> &[u8] {
        self.data
    }
}

/// Helper for performing screening logic on Solana accounts
#[derive(Debug)]
pub struct Selector {
    owners: HashSet<[u8; 32]>,
    pubkeys: HashSet<[u8; 32]>,
    mints: HashSet<Pubkey>,
    startup: Option<bool>,
}

impl Selector {
    /// Construct a new selector from the given configuration block
    ///
    /// # Errors
    /// Fails if an owner, public-key, or mint address is incorrectly specified
    pub fn from_config(config: Accounts) -> Result<Self> {
        let Accounts {
            owners,
            all_tokens,
            pubkeys,
            mints,
            startup,
        } = config;

        let owners = owners
            .into_iter()
            .map(|s| s.parse().map(Pubkey::to_bytes))
            .collect::<Result<_, _>>()
            .map_err(|e| Error::AccountConfig("owners", e.into()))?;

        let pubkeys = pubkeys
            .into_iter()
            .map(|s| s.parse().map(Pubkey::to_bytes))
            .collect::<Result<_, _>>()
            .map_err(|e| Error::AccountConfig("pubkeys", e.into()))?;

        let mints = mints
            .into_iter()
            .map(|s| s.parse::<Pubkey>())
            .collect::<Result<_, _>>()
            .map_err(|e| Error::AccountConfig("pubkeys", e.into()))?;

        let mut ret = Self {
            owners,
            pubkeys,
            mints,
            startup,
        };

        Ok(ret)
    }

    /// Returns the startup-based selector configuration
    #[inline]
    #[must_use]
    pub fn startup(&self) -> StartupType {
        StartupType::new(self.startup)
    }

    /// Returns true if the given account associated with the given startup flag
    /// has been requested by this selector's configuration
    #[inline]
    pub fn is_selected(&self, acct: &impl AccountInfo, is_startup: bool) -> bool {
        let owner = acct.owner();
        let pubkey = acct.pubkey();
        let data = acct.data();

        if self.startup.map_or(false, |s| is_startup != s) {
            return false;
        }

        if self.pubkeys.contains(pubkey) {
            return true;
        }

        let token = once_cell::unsync::Lazy::new(|| {
            if owner == spl_token::id().as_ref() && data.len() == TokenAccount::get_packed_len() {
                TokenAccount::unpack_from_slice(data).ok()
            } else {
                None
            }
        });

        if !self.mints.is_empty() && token.map_or(false, |t| self.mints.contains(&t.mint)) {
            return true;
        }

        if !self.owners.contains(owner) {
            return false;
        }

        if maybe_not_nft.unwrap_or(false) {
            return false;
        }

        true
    }
}
