#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use evm::{
	backend::TransactionalBackend,
	interpreter::{
		etable,
		runtime::{RuntimeBackend, RuntimeEnvironment},
		ExitError,
	},
	standard::{
		Config, EtableResolver, ExecutionEtable, GasometerEtable, Invoker, TransactArgs,
		TransactValue,
	},
};
use evm_precompile::StandardPrecompileSet;

pub static MAINNET_ETABLE: etable::Chained<GasometerEtable<'static>, ExecutionEtable<'static>> =
	etable::Chained(GasometerEtable::new(), ExecutionEtable::new());
pub static MAINNET_PRECOMPILE_SET: StandardPrecompileSet = StandardPrecompileSet;
pub static MAINNET_RESOLVER: EtableResolver<
	'static,
	'static,
	StandardPrecompileSet,
	etable::Chained<GasometerEtable<'static>, ExecutionEtable<'static>>,
> = EtableResolver::new(&MAINNET_PRECOMPILE_SET, &MAINNET_ETABLE);
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

pub static FRONTIER_CONFIG: Config = Config::frontier();
pub static ISTANBUL_CONFIG: Config = Config::istanbul();
pub static BERLIN_CONFIG: Config = Config::berlin();
pub static LONDON_CONFIG: Config = Config::london();
pub static MERGE_CONFIG: Config = Config::merge();
pub static SHANGHAI_CONFIG: Config = Config::shanghai();
pub static CANCUN_CONFIG: Config = Config::cancun();

const TRANSACT_MAINNET_HEAP_DEPTH: Option<usize> = Some(4);
pub fn transact<H>(args: TransactArgs<'static>, backend: &mut H) -> Result<TransactValue, ExitError>
where
	H: TransactionalBackend + RuntimeEnvironment + RuntimeBackend,
{
	evm::transact(args, TRANSACT_MAINNET_HEAP_DEPTH, backend, &MAINNET_INVOKER)
}
