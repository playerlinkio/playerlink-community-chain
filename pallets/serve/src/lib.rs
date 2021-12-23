#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	sp_runtime::traits::AccountIdConversion,
	traits::Get,
	BoundedVec,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use primitives::Balance;
use scale_info::TypeInfo;
use sp_runtime::{traits::One, RuntimeDebug};


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
	Year,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ServeState {
	Dev,
	Test,
	Prd,
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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::BlockNumberFor;


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
		CollectionServeCreated(CollectionId, ServeId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableCollectionId,
		CollectionFound,
		BadMetadata
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_serve(
			origin: OriginFor<T>,
			collection_id:u32,
			serve_types: ServeTypes,
			serve_version: Vec<u8>,
			serve_state:ServeState,
			serve_switch:bool,
			serve_name:Vec<u8>,
			serve_description:Vec<u8>,
			serve_url:Vec<u8>,
			serve_price:Balance,
			server_limit_time:Option<TimeTypes>,
			server_limit_times:Option<u32>,
			serve_deposit:Balance
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
			
			Self::do_add_serve(
				who,
				collection_id,
				serve_types: ServeTypes,
				serve_version: Vec<u8>,
				serve_state:ServeState,
				serve_switch:bool,
				serve_name:Vec<u8>,
				serve_description:Vec<u8>,
				serve_url:Vec<u8>,
				serve_price:Balance,
				server_limit_time:Option<TimeTypes>,
				server_limit_times:Option<u32>,
				serve_deposit
			)?;

			Ok(().into())
		}


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

	pub fn do_add_serve(
		who: T::AccountId,
		collection_id:u32,
		serve_types: ServeTypes,
		serve_version: Vec<u8>,
		serve_state:ServeState,
		serve_switch:bool,
		serve_name:Vec<u8>,
		serve_description:Vec<u8>,
		serve_url:Vec<u8>,
		serve_price:Balance,
		server_limit_time:Option<TimeTypes>,
		server_limit_times:Option<u32>,
		serve_deposit:Balance
	)-> DispatchResult{
		// Generate account from collection_id
		let next_collection_serve_id = NextCollectionServeId::<T>::get(collection_id);
		let escrow_account: <T as frame_system::Config>::AccountId =
			<T as pallet::Config>::PalletId::get().into_sub_account(next_collection_serve_id);

		let serve_id = NextCollectionServeId::<T>::try_mutate(
			collection_id,
			|id| -> Result<CollectionId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
				Ok(current_id)
			},
		)?;

		let bounded_serve_version: BoundedVec<u8, T::StringLimit> =
			serve_version.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let bounded_serve_name: BoundedVec<u8, T::StringLimit> =
			serve_name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let bounded_serve_description: BoundedVec<u8, T::StringLimit> =
			serve_description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let bounded_serve_url: BoundedVec<u8, T::StringLimit> =
			serve_url.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;


		let serve_metadata = ServeMetadata{
			serve_types,
			serve_version: bounded_serve_version,
			serve_state,
			serve_switch,
			serve_name: bounded_serve_name,
			serve_description: bounded_serve_description,
			serve_url: bounded_serve_url,
			serve_price,
			server_limit_time,
			server_limit_times
		};

		let new_serve = Serve{
			escrow_account,
			registered_serve_number: 0,
			serve_metadata,
			serve_deposit
		};

		CollectionServe::<T>::insert(collection_id, serve_id, new_serve);

		Self::deposit_event(Event::CollectionServeCreated(collection_id, serve_id, who));
		Ok(())

	}


}
