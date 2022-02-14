#![cfg_attr(not(feature = "std"), no_std)]

use base58::FromBase58;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::DispatchResult,
	ensure, log,
	sp_runtime::{
		app_crypto::TryFrom,
		traits::{AccountIdConversion, Verify},
		MultiSignature,
	},
	traits::{Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency},
	transactional,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use scale_info::TypeInfo;
use sp_application_crypto::sr25519::Signature;
use sp_core::crypto::AccountId32;
use sp_runtime::RuntimeDebug;
use sp_std::{str::from_utf8, vec::Vec};

pub use pallet::*;

pub type CollectionId = u32;
pub type ServeId = u32;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Certificate
#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug, TypeInfo)]
pub struct Certificate {
	pub account_id: AccountId32,
	pub collection_id: CollectionId,
	pub serve_id: ServeId,
	pub times: u32,
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
	pub serve_number: u32,
	/// all registered serve user number
	pub registered_serve_number_all: u32,
}

/// Collection Serve
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ServeMetadata<Balance> {
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
}

/// Serve
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Serve<AccountId, Balance> {
	/// storage deposit fess
	escrow_account: AccountId,
	/// local serve user number
	registered_serve_number: u32,
	serve_metadata: ServeMetadata<Balance>,
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
		type CreateCollectionMinBalance: Get<BalanceOf<Self>>;

		/// The minimum balance to create collection
		#[pallet::constant]
		type CreateServeMinBalance: Get<BalanceOf<Self>>;

		/// The minimum byte to create collection
		#[pallet::constant]
		type MessageMinBytes: Get<u32>;

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
		Serve<T::AccountId, BalanceOf<T>>,
	>;
	#[pallet::storage]
	pub(super) type UseServeEscrowAccount<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(AccountId32, CollectionId),
		Blake2_128Concat,
		ServeId,
		T::AccountId,
	>;

	#[pallet::storage]
	pub(super) type ServiceCertificate<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(AccountId32, CollectionId),
		Blake2_128Concat,
		ServeId,
		Certificate,
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
		ServiceCertificateCreated(T::AccountId, CollectionId, ServeId, Certificate),
		Deposit(T::AccountId, T::AccountId, BalanceOf<T>),
		Withdrawal(T::AccountId, T::AccountId, BalanceOf<T>),
		RemoveServe(T::AccountId, u32, u32),
		Test(bool),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableCollectionId,
		CollectionNotFound,
		ServeNotFound,
		BadMetadata,
		NotEnoughBalance,
		SignatureUsed,
		NumberOverflow,
		NotEnoughBalanceForRegisteredServerCertificate,
		UseServeDepositTooSmall,
		LessThanMessageMinBytes,
		ServiceCertificateNotFound,
		ServiceCertificateExisted,
		NoPermission,
		ParseJsonFailed,
		VerifySignatureFailed,
		UseServeEscrowAccountNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn registered_server_collection(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let min_balance = T::CreateCollectionMinBalance::get();
			T::Currency::reserve(&who, min_balance).map_err(|_| Error::<T>::NotEnoughBalance)?;
			let collection_id = NextCollectionId::<T>::get();
			NextCollectionId::<T>::mutate(|old_collection_id| {
				*old_collection_id = collection_id.saturating_add(1);
			});
			let collection = Collection {
				serve_builder: who.clone(),
				serve_number: 0,
				registered_serve_number_all: 0,
			};

			Collections::<T>::insert(collection_id, collection);
			Self::deposit_event(Event::CollectionCreated(collection_id, who));
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
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
			serve_price: BalanceOf<T>,
			server_limit_time: Option<TimeTypes>,
			server_limit_times: Option<u32>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let collection =
				Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
			ensure!(collection.serve_builder == who, Error::<T>::NoPermission);

			let min_balance = T::CreateServeMinBalance::get();
			T::Currency::reserve(&who, min_balance).map_err(|_| Error::<T>::NotEnoughBalance)?;

			let collection_serve_id = NextCollectionServeId::<T>::get(collection_id);
			let escrow_account: <T as frame_system::Config>::AccountId = Self::account_id(
				who.clone().encode(),
				collection_id,
				collection_serve_id.saturating_sub(1),
			);
			NextCollectionServeId::<T>::mutate(collection_id, |old_collection_serve_id| {
				*old_collection_serve_id = old_collection_serve_id.saturating_add(1);
			});
			let serve_metadata = ServeMetadata {
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
			};

			let new_serve = Serve { escrow_account, registered_serve_number: 0, serve_metadata };
			CollectionServe::<T>::insert(collection_id, collection_serve_id, new_serve);

			Collections::<T>::mutate(collection_id, |collection| {
				if let Some(ref mut c) = collection {
					c.serve_number =
						c.serve_number.checked_add(1).ok_or(Error::<T>::NumberOverflow)?;
				}
				Self::deposit_event(Event::CollectionServeCreated(
					collection_id,
					collection_serve_id,
					who,
				));
				Ok(())
			})
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn registered_use_server_certificate(
			origin: OriginFor<T>,
			collection_id: u32,
			serve_id: u32,
			use_serve_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let who_u8_32: [u8; 32] = (who.clone().encode().as_slice()).try_into().unwrap();
			let who_account_id32 = AccountId32::from(who_u8_32);
			let service_certificate =
				ServiceCertificate::<T>::get((who_account_id32.clone(), collection_id), serve_id);
			ensure!(service_certificate == None, Error::<T>::ServiceCertificateExisted);
			CollectionServe::<T>::get(collection_id, serve_id)
				.ok_or(Error::<T>::CollectionNotFound)?;
			let escrow_account: <T as frame_system::Config>::AccountId =
				Self::account_id(who.clone().encode(), collection_id, serve_id);
			<T as Config>::Currency::transfer(
				&who,
				&escrow_account,
				use_serve_deposit,
				ExistenceRequirement::KeepAlive,
			)?;

			UseServeEscrowAccount::<T>::insert(
				(who_account_id32.clone(), collection_id),
				serve_id,
				escrow_account,
			);

			let certificate = Certificate {
				account_id: who_account_id32.clone(),
				collection_id,
				serve_id,
				times: 0,
			};
			log::debug!(target: "serve", "Certificate = {:?}", certificate);
			ServiceCertificate::<T>::insert(
				(who_account_id32.clone(), collection_id),
				serve_id,
				certificate.clone(),
			);

			Self::deposit_event(Event::ServiceCertificateCreated(
				who,
				collection_id,
				serve_id,
				certificate,
			));
			Collections::<T>::mutate(collection_id, |collection| {
				let collection = collection.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
				collection.registered_serve_number_all = collection
					.registered_serve_number_all
					.checked_add(1)
					.ok_or(Error::<T>::NumberOverflow)?;
				CollectionServe::<T>::mutate(collection_id, serve_id, |serve| {
					let serve = serve.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
					serve.registered_serve_number = serve
						.registered_serve_number
						.checked_add(1)
						.ok_or(Error::<T>::NumberOverflow)?;
					Ok(())
				})
			})
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn deposit(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			serve_id: ServeId,
			use_serve_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let who_u8_32: [u8; 32] = (who.clone().encode().as_slice()).try_into().unwrap();
			let who_account_id32 = AccountId32::from(who_u8_32);
			let escrow_account = UseServeEscrowAccount::<T>::get(
				(who_account_id32.clone(), collection_id),
				serve_id,
			)
			.ok_or(Error::<T>::UseServeEscrowAccountNotFound)?;
			<T as Config>::Currency::transfer(
				&who,
				&escrow_account,
				use_serve_deposit,
				ExistenceRequirement::KeepAlive,
			)?;
			Self::deposit_event(Event::Deposit(who, escrow_account, use_serve_deposit));
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn withdrawal(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			serve_id: ServeId,
			use_serve_withdrawal: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let who_u8_32: [u8; 32] = (who.clone().encode().as_slice()).try_into().unwrap();
			let who_account_id32 = AccountId32::from(who_u8_32);
			let escrow_account = UseServeEscrowAccount::<T>::get(
				(who_account_id32.clone(), collection_id),
				serve_id,
			)
			.ok_or(Error::<T>::UseServeEscrowAccountNotFound)?;
			<T as Config>::Currency::transfer(
				&escrow_account,
				&who,
				use_serve_withdrawal,
				ExistenceRequirement::AllowDeath,
			)?;
			Self::deposit_event(Event::Withdrawal(escrow_account, who, use_serve_withdrawal));
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn check(
			origin: OriginFor<T>,
			address: AccountId32,
			message: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			ensure!(
				message.len() >= T::MessageMinBytes::get() as usize,
				Error::<T>::LessThanMessageMinBytes
			);
			log::debug!(target: "serve", "Certificate = {:?}", message.len());

			let certificate =
				Self::parse_json(message.clone()).map_err(|_| Error::<T>::ParseJsonFailed)?;
			log::debug!(target: "serve", "Certificate = {:?}", certificate);

			let c = ServiceCertificate::<T>::get(
				(address.clone(), certificate.collection_id),
				certificate.serve_id,
			)
			.ok_or(Error::<T>::ServiceCertificateNotFound)?;
			log::debug!(target: "serve", "Certificate = {:?}", c);

			let t = certificate.eq(&c);
			log::debug!(target: "serve", "Certificate = {:?}", t);
			ensure!(certificate.eq(&c), Error::<T>::ServiceCertificateNotFound);

			let u: &[u8; 64] = <&[u8; 64]>::try_from(signature.as_slice()).unwrap();
			let sign = Signature::from_raw(*u);
			let multi_sig = MultiSignature::from(sign);
			let result = multi_sig.verify(message.as_slice(), &address);
			ensure!(result, Error::<T>::VerifySignatureFailed);

			let use_serve_escrow_ccount = UseServeEscrowAccount::<T>::get(
				(address.clone(), certificate.collection_id),
				certificate.serve_id,
			)
			.unwrap();
			let serve = CollectionServe::<T>::get(certificate.collection_id, certificate.serve_id)
				.ok_or(Error::<T>::ServeNotFound)?;
			let collection = Collections::<T>::get(certificate.collection_id)
				.ok_or(Error::<T>::CollectionNotFound)?;
			<T as Config>::Currency::transfer(
				&use_serve_escrow_ccount,
				&collection.serve_builder,
				serve.serve_metadata.serve_price,
				ExistenceRequirement::AllowDeath,
			)?;
			ServiceCertificate::<T>::mutate(
				(address.clone(), certificate.collection_id),
				certificate.serve_id,
				|c| {
					let old_certificate =
						c.as_mut().ok_or(Error::<T>::ServiceCertificateNotFound)?;
					old_certificate.times =
						old_certificate.times.checked_add(1).ok_or(Error::<T>::NumberOverflow)?;
					Self::deposit_event(Event::Test(result));
					Ok(())
				},
			)
		}
		// TODO: remove_serve
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn remove_serve(
		// 	origin: OriginFor<T>,
		// 	collection_id: u32,
		// 	serve_id: u32,
		// ) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		// 	let collection =
		// Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
		// 	ensure!(collection.serve_builder==who,Error::<T>::NoPermission);
		// 	CollectionServe::<T>::mutate(collection_id, serve_id, |serve|{
		// 		*serve= None;
		// 		Self::deposit_event(Event::RemoveServe(_who,collection_id,serve_id));
		// 		Ok(())
		// 	})
		// }
		// TODO: remove_collection
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn remove_collection(
		// 	origin: OriginFor<T>,
		// 	collection_id: u32,
		// 	serve_id: u32,
		// ) -> DispatchResult {
		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {
	pub fn parse_json(message: Vec<u8>) -> Result<Certificate, Error<T>> {
		let mut flag: Vec<u8> = Vec::new();
		for x in 0..message.len() {
			if let Some(44) = message.get(x) {
				flag.push(x as u8);
			}
		}
		let account_id_vec = message[15..(flag[0] as usize - 1)].to_vec();
		let account_id_str =
			from_utf8(account_id_vec.as_slice()).map_err(|_| Error::<T>::ParseJsonFailed)?;
		let res = account_id_str.from_base58().unwrap();
		let public_key: [u8; 32] = <[u8; 32]>::try_from(&res[1..33]).unwrap();
		let account_id32 = AccountId32::from(public_key);
		let collection_id = message[flag[0] as usize + 18..(flag[1] as usize - 1)].to_vec();
		let collection_id_str =
			from_utf8(collection_id.as_slice()).map_err(|_| Error::<T>::ParseJsonFailed)?;
		let collection_id_u32 =
			(collection_id_str.parse::<u32>()).map_err(|_| Error::<T>::ParseJsonFailed)?;
		let serve_id = message[flag[1] as usize + 13..flag[2] as usize - 1].to_vec();
		let serve_id_str =
			from_utf8(serve_id.as_slice()).map_err(|_| Error::<T>::ParseJsonFailed)?;
		let serve_id_u32 = serve_id_str.parse::<u32>().map_err(|_| Error::<T>::ParseJsonFailed)?;
		let times = message[flag[2] as usize + 10..message.len() - 2].to_vec();
		let times_str = from_utf8(times.as_slice()).map_err(|_| Error::<T>::ParseJsonFailed)?;
		let times_u32 = times_str.parse::<u32>().map_err(|_| Error::<T>::ParseJsonFailed)?;

		Ok(Certificate {
			account_id: account_id32,
			collection_id: collection_id_u32,
			serve_id: serve_id_u32,
			times: times_u32,
		})
	}
	// The account ID of the vault
	fn account_id(mut account_id: Vec<u8>, collection_id: u32, serve_id: u32) -> T::AccountId {
		account_id.push(collection_id as u8);
		account_id.push(serve_id as u8);
		T::PalletId::get().into_sub_account(account_id)
	}
}
