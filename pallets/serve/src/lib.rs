#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
};
use primitives::Balance;
use sp_runtime::{traits::One, RuntimeDebug};
use scale_info::TypeInfo;
pub use pallet::*;

pub type CollectionId = u32;
pub type ServeId = u32;


/// Serve Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Collection<AccountId> {
	/// Collection owner
	pub serve_builder: AccountId,
	/// all registered serve number
	pub serve_number: u8,
	/// all registered serve user number
	pub registered_serve_number_all: u32,
	/// all registered deposit fees for the Severs
	pub serve_deposit_all: Balance,

}

/// Serve
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Serve<AccountId,BoundedString> {
	/// storage deposit fess
	escrow_account: AccountId,
	/// local serve user number
	registered_serve_number: u32,
	serve_metadata:ServeMetadata<BoundedString>,
	/// registered deposit fees for the Local Sever
	serve_deposit:Balance,
}


/// Collection Serve
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ServeMetadata<BoundedString> {
	serve_types: ServeTypes,
	serve_version: BoundedString,
	serve_state:ServeState,
	serve_switch:bool,
	serve_name:BoundedString,
	serve_description:BoundedString,
	serve_url:BoundedString,
	serve_price:Balance,
	server_limit_time:Option<TimeTypes>,
	server_limit_times:Option<u32>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ServeTypes {
	RecordTime,
	RecordTimes,
	RecordNumbers,
}


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum TimeTypes {
	Hour,
	Day,
	Month,
	Year
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ServeState {
	Dev,
	Test,
	Prd,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The maximum length of metadata stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Collections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Collection<T::AccountId>,
	>;


	#[pallet::storage]
	pub(super) type CollectionServe<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Blake2_128Concat,
		ServeId,
		Serve<T::AccountId, BoundedVec<u8, <T as pallet::Config>::StringLimit>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_serve_id)]
	pub(super) type NextCollectionServeId<T: Config> =
	StorageMap<_, Blake2_128Concat, CollectionId, ServeId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated(CollectionId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableCollectionId,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn registered_server_collection(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_registered_server_collection(&who)?;

			Ok(().into())
		}

		// An example dispatchable that may throw a custom error.
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		//
		//
		// 	match <Something<T>>::get() {
		// 		None => Err(Error::<T>::NoneValue)?,
		// 		Some(old) => {
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_registered_server_collection(
		who: &T::AccountId,
	) -> Result<CollectionId, DispatchError>{
		let collection_id =
			NextCollectionId::<T>::try_mutate(|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			})?;

		let collection = Collection {
			serve_builder: who.clone(),
			serve_number: 0,
			registered_serve_number_all: 0,
			serve_deposit_all: 0
		};

		Collections::<T>::insert(collection_id, collection);
		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	}
}
