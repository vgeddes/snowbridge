pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode};
use ethabi::{self, Token};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{EnsureOrigin, Get},
};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256};
use sp_io::offchain_index;
use sp_runtime::traits::{Hash, StaticLookup, Zero};

use sp_std::prelude::*;

use snowbridge_core::{types::AuxiliaryDigestItem, ChannelId};

pub use weights::WeightInfo;

/// Wire-format for committed messages
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MessageBundle {
	nonce: u64,
	messages: Vec<Message>,
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Message {
	/// Unique message ID
	id: u64,
	/// Target application on the Ethereum side.
	target: H160,
	/// Payload for target application.
	payload: Vec<u8>,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Prefix for offchain storage keys.
		const INDEXING_PREFIX: &'static [u8];

		type Hashing: Hash<Output = H256>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u64>;

		/// Max number of messages per commitment
		#[pallet::constant]
		type MaxMessagesPerCommit: Get<u32>;

		type SetPrincipalOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageAccepted(u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message payload exceeds byte limit.
		PayloadTooLarge,
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
		/// Cannot increment nonce
		Overflow,
		/// Not authorized to send message
		NotAuthorized,
	}

	/// Interval between commitments
	#[pallet::storage]
	#[pallet::getter(fn interval)]
	pub(super) type Interval<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Messages waiting to be committed.
	#[pallet::storage]
	pub(super) type MessageQueue<T: Config> =
		StorageValue<_, BoundedVec<Message, T::MaxMessagesPerCommit>, ValueQuery>;

	/// Fee for accepting a message
	#[pallet::storage]
	#[pallet::getter(fn principal)]
	pub type Principal<T: Config> = StorageValue<_, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type NextId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub interval: T::BlockNumber,
		pub principal: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { interval: Default::default(), principal: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Interval<T>>::put(self.interval);
			<Principal<T>>::put(self.principal.clone());
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Generate a message commitment every [`Interval`] blocks.
		//
		// The commitment hash is included in an [`AuxiliaryDigestItem`] in the block header,
		// with the corresponding commitment is persisted offchain.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			if (now % Self::interval()).is_zero() {
				Self::commit()
			} else {
				T::WeightInfo::on_initialize_non_interval()
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::set_principal())]
		pub fn set_principal(
			origin: OriginFor<T>,
			principal: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::SetPrincipalOrigin::ensure_origin(origin)?;
			let principal = T::Lookup::lookup(principal)?;
			<Principal<T>>::put(Some(principal));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submit message on the outbound channel
		pub fn submit(who: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
			let principal = Self::principal();
			ensure!(principal.is_some(), Error::<T>::NotAuthorized,);
			ensure!(*who == principal.unwrap(), Error::<T>::NotAuthorized,);
			ensure!(
				<MessageQueue<T>>::decode_len().unwrap_or(0)
					< T::MaxMessagesPerCommit::get() as usize,
				Error::<T>::QueueSizeLimitReached,
			);
			ensure!(
				payload.len() <= T::MaxMessagePayloadSize::get() as usize,
				Error::<T>::PayloadTooLarge,
			);

			let next_id = <NextId<T>>::get();
			if next_id.checked_add(1).is_none() {
				return Err(Error::<T>::Overflow.into());
			}

			<MessageQueue<T>>::try_append(Message {
				id: next_id,
				target,
				payload: payload.to_vec(),
			})
			.map_err(|_| Error::<T>::QueueSizeLimitReached)?;
			Self::deposit_event(Event::MessageAccepted(next_id));

			<NextId<T>>::put(next_id + 1);

			Ok(())
		}

		fn commit() -> Weight {
			let messages: BoundedVec<Message, T::MaxMessagesPerCommit> = <MessageQueue<T>>::take();
			if messages.is_empty() {
				return T::WeightInfo::on_initialize_no_messages();
			}

			let nonce = <Nonce<T>>::get();
			let next_nonce = nonce.saturating_add(1);
			<Nonce<T>>::put(next_nonce);

			let bundle =
				MessageBundle { nonce: next_nonce, messages: messages.clone().into_inner() };

			let commitment_hash = Self::make_commitment_hash(&bundle);
			let average_payload_size = Self::average_payload_size(&bundle.messages);

			let digest_item =
				AuxiliaryDigestItem::Commitment(ChannelId::Basic, commitment_hash.clone()).into();
			<frame_system::Pallet<T>>::deposit_log(digest_item);

			let key = Self::make_offchain_key(commitment_hash);
			offchain_index::set(&*key, &bundle.encode());

			T::WeightInfo::on_initialize(messages.len() as u32, average_payload_size as u32)
		}

		fn make_commitment_hash(bundle: &MessageBundle) -> H256 {
			let messages: Vec<Token> = bundle
				.messages
				.iter()
				.map(|message| {
					Token::Tuple(vec![
						Token::Uint(message.id.into()),
						Token::Address(message.target),
						Token::Bytes(message.payload.clone()),
					])
				})
				.collect();
			let input = ethabi::encode(&vec![Token::Tuple(vec![
				Token::Uint(bundle.nonce.into()),
				Token::Array(messages),
			])]);
			<T as Config>::Hashing::hash(&input)
		}

		fn average_payload_size(messages: &[Message]) -> usize {
			let sum: usize = messages.iter().fold(0, |acc, x| acc + x.payload.len());
			// We overestimate message payload size rather than underestimate.
			// So add 1 here to account for integer division truncation.
			(sum / messages.len()).saturating_add(1)
		}

		fn make_offchain_key(hash: H256) -> Vec<u8> {
			(T::INDEXING_PREFIX, ChannelId::Basic, hash).encode()
		}
	}
}
