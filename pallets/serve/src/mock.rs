use crate as pallet_serve;
use frame_support::{parameter_types, traits::GenesisBuild, PalletId};
use frame_system as system;
use pallet_balances::AccountData;
pub use pallet_serve::{ServeState, ServeTypes, ServeWays, TimeTypes};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Serve: pallet_serve::{Pallet, Call,Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const MaxDataSize: u32 = 1024 * 1024;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	// AccountId must be u128 for auction account
	type AccountId = u128;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	/// The type for recording an account's balance.
	type Balance = u128;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
}

parameter_types! {
	pub const MarketplacePalletId: PalletId = PalletId(*b"pl/serve");
	pub const StringLimit: u32 = 50;
	/// 100 PL  create a serve store
	pub const CreateCollectionMinBalance: Balance = 100 * 10_000_000_000;
	pub const CreateServeMinBalance: Balance = 10 * 10_000_000_000;
}

impl pallet_serve::Config for Test {
	type Event = Event;
	type StringLimit = StringLimit;
	type PalletId = MarketplacePalletId;
	type CreateCollectionMinBalance = CreateCollectionMinBalance;
	type CreateServeMinBalance = CreateServeMinBalance;
	type Currency = Balances;
}

// Build test environment by setting the admin `key` for the Genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 10_000_000_000_000),
			(2, 10_000_000_000_000),
			(3, 10_000_000_000_000),
			(4, 10_000_000_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub(crate) fn last_event() -> Event {
	system::Pallet::<Test>::events().pop().expect("Event expected").event
}

pub(crate) fn expect_event<E: Into<Event>>(e: E) {
	assert_eq!(last_event(), e.into());
}
