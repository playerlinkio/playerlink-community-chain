#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	sp_runtime::traits::AccountIdConversion,
	traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
	BoundedVec,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
// use primitives::Balance;
use frame_support::sp_runtime::{app_crypto::TryFrom, traits::Verify, MultiSignature};
use scale_info::TypeInfo;
use sp_application_crypto::sr25519::Signature;
use sp_runtime::{traits::One, RuntimeDebug};
use sp_std::vec::Vec;

use sp_core::crypto::AccountId32;

pub use pallet::*;

pub type CollectionId = u32;
pub type ServeId = u32;
pub type Balance = u128;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Certificate
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Certificate {
	account_id: Vec<u8>,
	collection_id: Vec<u8>,
	serve_id: Vec<u8>,
}
/// SignatureData
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignatureData {
	address: AccountId32,
	message: Certificate,
	signature: Vec<u8>,
}
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
pub struct Serve<AccountId, BoundedString> {
	/// storage deposit fess
	escrow_account: AccountId,
	/// local serve user number
	registered_serve_number: u32,
	serve_metadata: ServeMetadata<BoundedString>,
	/// registered deposit fees for the Local Sever
	serve_deposit: Balance,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ServeTypes {
	RecordTime,
	RecordTimes,
	RecordNumbers,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ServeWays {
	RESTFUL,
	GRAPHQL,
	RPC,
	OTHER,
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
	serve_ways: ServeWays,
	serve_state: ServeState,
	serve_switch: bool,
	serve_version: BoundedString,
	serve_name: BoundedString,
	serve_description: BoundedString,
	serve_url: BoundedString,
	serve_price: Balance,
	server_limit_time: Option<TimeTypes>,
	server_limit_times: Option<u32>,
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

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateCollectionDeposit: Get<BalanceOf<Self>>;

		// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The currency that people are electing with.
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Collections<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, Collection<T::AccountId>>;

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
	pub(super) type ServiceCertificate<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(T::AccountId, CollectionId),
		Blake2_128Concat,
		ServeId,
		Balance,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_serve_id)]
	pub(super) type NextCollectionServeId<T: Config> =
		StorageMap<_, Blake2_128Concat, CollectionId, ServeId, ValueQuery>;

	#[pallet::type_value]
	pub(super) fn DefaultSignatureData() -> SignatureData {
		let certificate = Certificate {
			account_id: Default::default(),
			collection_id: Default::default(),
			serve_id: Default::default(),
		};
		SignatureData {
			address: Default::default(),
			message: certificate,
			signature: Default::default(),
		}
	}
	#[pallet::storage]
	#[pallet::getter(fn last_signature)]
	pub(super) type LastSignature<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(Vec<u8>, Vec<u8>),
		SignatureData,
		ValueQuery,
		DefaultSignatureData,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub(super) type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CollectionCreated(CollectionId, T::AccountId),
		CollectionServeCreated(CollectionId, ServeId, T::AccountId),
		ServiceCertificateCreated(T::AccountId, CollectionId, ServeId, Balance),
		Test(bool),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableCollectionId,
		CollectionFound,
		BadMetadata,
		NotEnoughBalance,
		SignatureUsed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn registered_server_collection(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_registered_server_collection(&who)?;

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn check(
			origin: OriginFor<T>,
			address: AccountId32,
			message: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let u: &[u8; 64] = <&[u8; 64]>::try_from(signature.as_slice()).unwrap();
			let sign = Signature::from_raw(*u);
			let multi_sig = MultiSignature::from(sign);
			let result = multi_sig.verify(message.as_slice(), &address);
			if result {
				let certificate = Self::parse_json(message);
				let signature_data = SignatureData {
					address,
					message: certificate.clone(),
					signature: signature.clone(),
				};
				ensure!(
					signature !=
						LastSignature::<T>::get((
							certificate.collection_id.clone(),
							certificate.serve_id.clone()
						))
						.signature,
					Error::<T>::SignatureUsed
				);
				LastSignature::<T>::insert(
					(certificate.collection_id, certificate.serve_id),
					signature_data,
				);
			}
			Self::deposit_event(Event::Test(result));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn registered_use_server_certificate(
			origin: OriginFor<T>,
			collection_id: u32,
			serve_id: u32,
			use_serve_deposit: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_registered_use_server_certificate(
				&who,
				collection_id,
				serve_id,
				use_serve_deposit,
			)?;

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_serve(
			origin: OriginFor<T>,
			collection_id: u32,
			serve_types: ServeTypes,
			serve_ways: ServeWays,
			serve_state: ServeState,
			serve_switch: bool,
			serve_version: Vec<u8>,
			serve_name: Vec<u8>,
			serve_description: Vec<u8>,
			serve_url: Vec<u8>,
			serve_price: Balance,
			server_limit_time: Option<TimeTypes>,
			server_limit_times: Option<u32>,
			serve_deposit: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;

			Self::do_add_serve(
				who,
				collection_id,
				serve_types,
				serve_ways,
				serve_state,
				serve_switch,
				serve_version,
				serve_name,
				serve_description,
				serve_url,
				serve_price,
				server_limit_time,
				server_limit_times,
				serve_deposit,
			)?;

			Ok(().into())
		}
		// TODO add remove serve
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn remove_serve(
		// 	origin: OriginFor<T>,
		// 	collection_id: u32,
		// 	serve_id: u32,
		// ) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	// Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionFound)?;
		// 	// who: T::AccountId, collection_id: u32, serve_id: u32
		// 	Self::do_remove_serve(who, collection_id, serve_id)?;

		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {
	pub fn parse_json(message: Vec<u8>) -> Certificate {
		let mut flag: Vec<u8> = Vec::new();
		for x in 0..message.len() {
			if let Some(44) = message.get(x) {
				flag.push(x as u8);
			}
		}
		let account_id = message[15..(*flag.get(0).unwrap() as usize - 1)].to_vec();
		let collection_id = message
			[*flag.get(0).unwrap() as usize + 18..(*flag.get(1).unwrap() as usize - 1)]
			.to_vec();
		let serve_id = message[*flag.get(1).unwrap() as usize + 13..message.len() - 2].to_vec();
		return Certificate { account_id, collection_id, serve_id }
	}
	// The account ID of the vault
	fn account_id() -> T::AccountId {
		<T as Config>::PalletId::get().into_account()
	}

	pub fn do_registered_server_collection(
		who: &T::AccountId,
	) -> Result<CollectionId, DispatchError> {
		// fees
		let deposit = T::CreateCollectionDeposit::get();
		let who_balance = <T as Config>::Currency::total_balance(&who);
		ensure!(who_balance >= deposit, Error::<T>::NotEnoughBalance);
		<T as Config>::Currency::transfer(who, &Self::account_id(), deposit, AllowDeath)?;

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
			serve_deposit_all: 0,
		};

		Collections::<T>::insert(collection_id, collection);
		Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
		Ok(collection_id)
	}

	pub fn do_registered_use_server_certificate(
		who: &T::AccountId,
		collection_id: u32,
		serve_id: u32,
		use_serve_deposit: Balance,
	) -> DispatchResult {
		CollectionServe::<T>::get(collection_id, serve_id).ok_or(Error::<T>::CollectionFound)?;

		// fees
		let deposit = T::CreateCollectionDeposit::get();
		let who_balance = <T as Config>::Currency::total_balance(&who);

		let serve_deposit_balance = BalanceOf::<T>::try_from(use_serve_deposit)
			.map_err(|_| "balance expect u128 type")
			.unwrap();

		ensure!(who_balance >= deposit, Error::<T>::NotEnoughBalance);
		ensure!(who_balance >= serve_deposit_balance, Error::<T>::NotEnoughBalance);
		<T as Config>::Currency::transfer(who, &Self::account_id(), deposit, AllowDeath)?;

		let escrow_account =
			CollectionServe::<T>::get(collection_id, serve_id).unwrap().escrow_account;

		<T as Config>::Currency::transfer(who, &escrow_account, serve_deposit_balance, AllowDeath)?;

		ServiceCertificate::<T>::insert((who, collection_id), serve_id, use_serve_deposit);

		Self::deposit_event(Event::ServiceCertificateCreated(
			who.clone(),
			collection_id,
			serve_id,
			use_serve_deposit,
		));

		Ok(())
	}

	pub fn do_add_serve(
		who: T::AccountId,
		collection_id: u32,
		serve_types: ServeTypes,
		serve_ways: ServeWays,
		serve_state: ServeState,
		serve_switch: bool,
		serve_version: Vec<u8>,
		serve_name: Vec<u8>,
		serve_description: Vec<u8>,
		serve_url: Vec<u8>,
		serve_price: Balance,
		server_limit_time: Option<TimeTypes>,
		server_limit_times: Option<u32>,
		serve_deposit: Balance,
	) -> DispatchResult {
		let who_balance = <T as Config>::Currency::total_balance(&who);
		let serve_deposit_balance = BalanceOf::<T>::try_from(serve_deposit)
			.map_err(|_| "balance expect u128 type")
			.unwrap();
		ensure!(who_balance >= serve_deposit_balance, Error::<T>::NotEnoughBalance);

		// Generate account from collection_id
		let next_collection_serve_id = NextCollectionServeId::<T>::get(collection_id);
		let escrow_account: <T as frame_system::Config>::AccountId =
			<T as pallet::Config>::PalletId::get().into_sub_account(next_collection_serve_id);

		<T as Config>::Currency::transfer(
			&who,
			&escrow_account,
			serve_deposit_balance,
			AllowDeath,
		)?;

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

		let serve_metadata = ServeMetadata {
			serve_types,
			serve_ways,
			serve_state,
			serve_switch,
			serve_version: bounded_serve_version,
			serve_name: bounded_serve_name,
			serve_description: bounded_serve_description,
			serve_url: bounded_serve_url,
			serve_price,
			server_limit_time,
			server_limit_times,
		};

		let new_serve =
			Serve { escrow_account, registered_serve_number: 0, serve_metadata, serve_deposit };
		CollectionServe::<T>::insert(collection_id, serve_id, new_serve);
		Self::deposit_event(Event::CollectionServeCreated(collection_id, serve_id, who));
		Ok(())
	}

	// pub fn do_remove_serve(who: T::AccountId, collection_id: u32, serve_id: u32) {
	// 	let Serve { serve_deposit, .. } = <CollectionServe<T>>::take(collection_id, serve_id);

	// remove storage, lock and unreserve.
	// T::Currency::remove_lock(T::PalletId::get(), who);

	// NOTE: we could check the deposit amount before removing and skip if zero, but it will be
	// a noop anyhow.
	// let _remainder = T::Currency::unreserve(who, serve_deposit);
	// debug_assert!(_remainder.is_zero());
	// }
}
