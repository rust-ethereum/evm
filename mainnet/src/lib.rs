//! Mainnet convenience functions for [evm](https://docs.rs/evm).

#![deny(warnings)]
#![forbid(unused_variables, unused_imports)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use evm::{
	backend::TransactionalBackend,
	interpreter::{
		ExitError, etable,
		runtime::{RuntimeBackend, RuntimeEnvironment},
	},
	standard::{
		Config, EtableResolver, ExecutionEtable, GasometerEtable, Invoker, TransactArgs,
		TransactValue,
	},
};
use evm_precompile::StandardPrecompileSet;

#[doc(hidden)]
pub use evm;
#[doc(hidden)]
pub use evm_precompile;

/// Mainnet [evm::interpreter::etable::Etable].
pub static MAINNET_ETABLE: etable::Chained<GasometerEtable<'static>, ExecutionEtable<'static>> =
	etable::Chained(GasometerEtable::new(), ExecutionEtable::new());
/// PrecompileSet for mainnet.
pub static MAINNET_PRECOMPILE_SET: StandardPrecompileSet = StandardPrecompileSet;
/// Mainnet [EtableResolver].
pub static MAINNET_RESOLVER: EtableResolver<
	'static,
	'static,
	StandardPrecompileSet,
	etable::Chained<GasometerEtable<'static>, ExecutionEtable<'static>>,
> = EtableResolver::new(&MAINNET_PRECOMPILE_SET, &MAINNET_ETABLE);
/// Mainnet [Invoker].
pub static MAINNET_INVOKER: Invoker<
	'static,
	'static,
	EtableResolver<
		'static,
		'static,
		StandardPrecompileSet,
		etable::Chained<GasometerEtable<'static>, ExecutionEtable<'static>>,
	>,
> = Invoker::new(&MAINNET_RESOLVER);

/// Config for the Frontier hard fork.
pub static FRONTIER_CONFIG: Config = Config::frontier();
/// Config for the Istanbul hard fork.
pub static ISTANBUL_CONFIG: Config = Config::istanbul();

const TRANSACT_MAINNET_HEAP_DEPTH: Option<usize> = Some(4);
/// Same as [transact], but use all `'static` lifetime to avoid a few stack allocations.
pub fn transact_static<H>(
	args: TransactArgs<'static>,
	backend: &mut H,
) -> Result<TransactValue, ExitError>
where
	H: TransactionalBackend + RuntimeEnvironment + RuntimeBackend,
{
	evm::transact(args, TRANSACT_MAINNET_HEAP_DEPTH, backend, &MAINNET_INVOKER)
}

/// Create a mainnet invoker.
#[macro_export]
macro_rules! with_mainnet_invoker {
	// This can technically be written as a normal function as well, but we then must explicitly write the complex type.
	( |$invoker:ident| $body:expr ) => {{
		let precompiles = $crate::evm_precompile::StandardPrecompileSet;
		let gas_etable =
			$crate::evm::interpreter::etable::Single::new($crate::evm::standard::eval_gasometer);
		let exec_etable = $crate::evm::standard::DispatchEtable::runtime();
		let etable = $crate::evm::interpreter::etable::Chained(gas_etable, exec_etable);
		let resolver = $crate::evm::standard::EtableResolver::new(&precompiles, &etable);
		let $invoker = $crate::evm::standard::Invoker::new(&resolver);
		$body
	}};
}

/// Invoke a transaction on mainnet.
pub fn transact<'config, H>(
	args: TransactArgs<'config>,
	backend: &mut H,
) -> Result<TransactValue, ExitError>
where
	H: TransactionalBackend + RuntimeEnvironment + RuntimeBackend,
{
	with_mainnet_invoker!(|invoker| {
		evm::transact(args, TRANSACT_MAINNET_HEAP_DEPTH, backend, &invoker)
	})
}
