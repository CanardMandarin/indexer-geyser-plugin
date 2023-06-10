use hashbrown::HashSet;
use solana_program::pubkey::Pubkey;

use crate::{config::Instructions, Error, Heuristic, Result};

/// Abstraction over a Solana instruction container
#[allow(clippy::module_name_repetitions)]
pub trait InstructionInfo<'a>: 'a {
    /// An iterator over the input account indices of this instruction
    type AccountIndices: IntoIterator<Item = u8> + 'a;

    /// The index of this instruction's target program
    fn program_index(&self) -> u8;

    /// The indices of this instruction's input accounts
    fn account_indices(&'a self) -> Self::AccountIndices;

    /// The data contained in this instruction
    fn data(&self) -> &[u8];
}

/// Helper for performing screening logic on Solana instructions
#[derive(Debug)]
pub struct Selector {
    programs: HashSet<Pubkey>,
}

impl Selector {
    /// Construct a new selector from the given configuration block
    ///
    /// # Errors
    /// Fails if a program address is incorrectly specified
    pub fn from_config(config: Instructions) -> Result<Self> {
        let Instructions {
            programs,
            all_token_calls,
        } = config;

        let programs = programs
            .into_iter()
            .map(|s| s.parse::<Pubkey>())
            .collect::<Result<_, _>>()
            .map_err(|e| Error::InstructionConfig("programs", e.into()))?;

        let mut ret = Self {
            programs,
        };


        Ok(ret)
    }

    /// Returns true if this selector will never select anything
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    /// Returns true if the given instruction addressed to the given program
    /// has been requested by this selector's configuration
    ///
    /// # Errors
    /// This function fails if an input account or program address cannot be
    /// retrieved
    #[inline]
    pub fn is_selected<'a>(
        &self,
        get_acct: impl Fn(u8) -> Option<&'a Pubkey>,
        ins: &impl InstructionInfo<'a>,
    ) -> Result<bool> {
        let pgm = ins.program_index();
        let pgm = get_acct(pgm).ok_or(Error::InstructionMissingAccount(pgm))?;
        if !self.programs.contains(pgm) {
            return Ok(false);
        }

        Ok(true)
    }
}
