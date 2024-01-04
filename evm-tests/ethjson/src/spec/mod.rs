// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Spec deserialization.

pub mod account;
pub mod authority_round;
pub mod basic_authority;
pub mod builtin;
pub mod clique;
pub mod engine;
pub mod ethash;
pub mod genesis;
pub mod hardcoded_sync;
pub mod instant_seal;
pub mod null_engine;
pub mod params;
pub mod seal;
#[allow(clippy::module_inception)]
pub mod spec;
pub mod state;
pub mod step_duration;
pub mod validator_set;

pub use self::account::Account;
pub use self::authority_round::{AuthorityRound, AuthorityRoundParams};
pub use self::basic_authority::{BasicAuthority, BasicAuthorityParams};
pub use self::builtin::{Builtin, Linear, Pricing};
pub use self::clique::{Clique, CliqueParams};
pub use self::engine::Engine;
pub use self::ethash::{BlockReward, Ethash, EthashParams};
pub use self::genesis::Genesis;
pub use self::hardcoded_sync::HardcodedSync;
pub use self::instant_seal::{InstantSeal, InstantSealParams};
pub use self::null_engine::{NullEngine, NullEngineParams};
pub use self::params::Params;
pub use self::seal::{AuthorityRoundSeal, Ethereum, Seal, TendermintSeal};
pub use self::spec::{ForkSpec, Spec};
pub use self::state::{HashOrMap, State};
pub use self::step_duration::StepDuration;
pub use self::validator_set::ValidatorSet;
