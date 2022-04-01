//! # Ethereum 2 Light Client Verifier
//!
//! This module implements the `Verifier` interface. Other modules should reference
//! this module using the `Verifier` type and perform verification using `Verifier::verify`.
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod merklization;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, log, transactional};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_core::hashing::sha2_256;

type Root = H256;
type Domain = H256;
type ValidatorIndex = u64;
type ProofBranch  = Vec<Vec<u8>>;
type ForkVersion = [u8; 4];

const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 22;
const CURRENT_SYNC_COMMITTEE_DEPTH: u64 = 5;

const NEXT_SYNC_COMMITTEE_DEPTH: u64 = 5;
const NEXT_SYNC_COMMITTEE_INDEX: u64 = 23;

const FINALIZED_ROOT_DEPTH: u64 = 6;
const FINALIZED_ROOT_INDEX: u64 = 41;

const MIN_SYNC_COMMITTEE_PARTICIPANTS: u64 = 1;

/// GENESIS_FORK_VERSION('0x00000000')
const GENESIS_FORK_VERSION: [u8; 4] = [30, 30, 30, 30];

/// DomainType('0x07000000')
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#domain-types
const DOMAIN_SYNC_COMMITTEE: [u8; 8] = [30, 37, 30, 30, 30, 30, 30, 30];

/// Beacon block header as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BeaconBlockHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	pub slot: u64,
	// The index of the validator that proposed the block.
	pub proposer_index: ValidatorIndex,
	// The block root of the parent block, forming a block chain.
	pub parent_root: Root,
	// The hash root of the post state of running the state transition through this block.
	pub state_root: Root,
	// The hash root of the Eth1 block
	pub body_root: Root,
}

/// Sync committee as it is stored in the runtime storage.
#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo
)]
pub struct SyncCommittee {
	pub pubkeys: Vec<Vec<u8>>,
	pub aggregate_pubkey: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct LightClientInitialSync {
	pub header: BeaconBlockHeader,
	pub current_sync_committee: SyncCommittee,
	pub current_sync_committee_branch: ProofBranch,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct LightClientSyncCommitteePeriodUpdate {
	pub attested_header: BeaconBlockHeader,
	pub next_sync_committee: SyncCommittee,
	pub next_sync_committee_branch: ProofBranch,
	pub finalized_header: BeaconBlockHeader,
	pub finality_branch: ProofBranch,
	pub sync_committee_aggregate: SyncAggregate,
	pub fork_version: ForkVersion,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct LightClientFinalizedHeaderUpdate {
	pub finalized_header: BeaconBlockHeader,
	pub finality_branch: ProofBranch,
	pub sync_committee_aggregate: SyncAggregate,
	pub fork_version: ForkVersion,
	pub genesis_validators_root: H256,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct SigningData {
	pub object_root: Root,
	pub domain: Domain,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
use milagro_bls::{Signature, AggregateSignature, PublicKey, AmclError, AggregatePublicKey};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
		AncientHeader,
		SkippedSyncCommitteePeriod,
		Unknown,
		InsufficientSyncCommitteeParticipants,
		InvalidSyncCommiteeSignature,
		InvalidHeaderMerkleProof,
		InvalidSyncCommitteeMerkleProof,
		InvalidSignature,
		InvalidSignaturePoint,
		InvalidAggregatePublicKeys,
		InvalidHash,
		SignatureVerificationFailed,
		NoBranchExpected,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	pub(super) type FinalizedHeaders<T: Config> = StorageMap<_, Identity, H256, BeaconBlockHeader, OptionQuery>;

	#[pallet::storage]
	pub(super) type FinalizedHeadersBySlot<T: Config> = StorageMap<_, Identity, u64, H256, OptionQuery>;

	/// Current sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type CurrentSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

	/// Next sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type NextSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		// genesis header goes header, maybe?
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn initial_sync(
			origin: OriginFor<T>,
			initial_sync: LightClientInitialSync,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"Received update {:?}. Starting initial_sync",
				initial_sync
			);

			Self::process_initial_sync(initial_sync)
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn sync_committee_period_update(
			origin: OriginFor<T>,
			sync_committee_period_update: LightClientSyncCommitteePeriodUpdate,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"Received update {:?}. Applying sync committee period update",
				sync_committee_period_update
			);

			Self::process_sync_committee_period_update(sync_committee_period_update)
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn import_finalized_header(
			origin: OriginFor<T>,
			finalized_header_update: LightClientFinalizedHeaderUpdate,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"Received update {:?}. Importing finalized header",
				finalized_header_update
			);

			Self::process_finalized_header(finalized_header_update)
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_initial_sync(
			initial_sync: LightClientInitialSync,
		) -> DispatchResult {
			Self::verify_sync_committee(
				initial_sync.current_sync_committee, 
				initial_sync.current_sync_committee_branch, 
				initial_sync.header.state_root,
				CURRENT_SYNC_COMMITTEE_DEPTH,
				CURRENT_SYNC_COMMITTEE_INDEX
			)?;

			Self::store_header(initial_sync.header);
			
			Ok(())
		}

		fn process_sync_committee_period_update(
			update: LightClientSyncCommitteePeriodUpdate,
		) -> DispatchResult {		
			Self::verify_sync_committee(
				update.next_sync_committee, 
				update.next_sync_committee_branch, 
				update.finalized_header.state_root,
				NEXT_SYNC_COMMITTEE_DEPTH,
				NEXT_SYNC_COMMITTEE_INDEX
			)?;

			Self::verify_header(
				update.finalized_header, 
				update.finality_branch, 
				update.attested_header.state_root,
				FINALIZED_ROOT_DEPTH,
				FINALIZED_ROOT_INDEX
			)?;

			Ok(())
		}

		fn process_finalized_header(
			update: LightClientFinalizedHeaderUpdate,
		) -> DispatchResult {		
			// TODO merkle proof
			let sync_commitee_bits = Self::convert_to_binary(update.sync_committee_aggregate.sync_committee_bits.clone());

			ensure!(Self::get_sync_committee_sum(update.sync_committee_aggregate.sync_committee_bits) >= MIN_SYNC_COMMITTEE_PARTICIPANTS as u64,
				Error::<T>::InsufficientSyncCommitteeParticipants
			);

			let mut sync_committee = <CurrentSyncCommittee<T>>::get();

			let mut participant_pubkeys: Vec<Vec<u8>> = Vec::new();

			// Gathers all the pubkeys of the sync committee members that participated in siging the header.
			for (bit, pubkey) in sync_commitee_bits
				.iter()
				.zip(sync_committee.pubkeys.iter_mut())
			{
				if *bit == 1 as u8 {
					let pubk = pubkey.clone();
					participant_pubkeys.push(pubk.to_vec());
				}
			}

			// Domains are used for for seeds, for signatures, and for selecting aggregators.
			let domain = Self::compute_domain(
				DOMAIN_SYNC_COMMITTEE.to_vec(),
				Some(update.fork_version),
				update.genesis_validators_root,
			)?;

			// Hash tree root of SigningData - object root + domain
			let signing_root = Self::compute_signing_root(update.finalized_header, domain)?;

			// Verify sync committee aggregate signature.
			Self::bls_fast_aggregate_verify(
				participant_pubkeys,
				signing_root,
				update.sync_committee_aggregate.sync_committee_signature,
			)?;

			Ok(())
		}

		pub(super) fn bls_fast_aggregate_verify(
			pubkeys: Vec<Vec<u8>>,
			message: H256,
			signature: Vec<u8>,
		) -> DispatchResult {
			let sig = Signature::from_bytes(&signature[..]);

			if let Err(e) = sig {
				return Err(Error::<T>::InvalidSignature.into());
			}

			let agg_sig = AggregateSignature::from_signature(&sig.unwrap());

			let public_keys_res: Result<Vec<PublicKey>, _> =
				pubkeys.iter().map(|bytes| PublicKey::from_bytes(&bytes)).collect();

			if let Err(e) = public_keys_res {
				match e {
					AmclError::InvalidPoint => return Err(Error::<T>::InvalidSignaturePoint.into()),
					_ => return Err(Error::<T>::InvalidSignature.into()),
				};
			}

			let agg_pub_key_res = AggregatePublicKey::into_aggregate(&public_keys_res.unwrap());

			if let Err(e) = agg_pub_key_res {
				return Err(Error::<T>::InvalidAggregatePublicKeys.into());
			}

			ensure!(
				agg_sig.fast_aggregate_verify_pre_aggregated(
					&message.as_bytes(),
					&agg_pub_key_res.unwrap()
				),
				Error::<T>::SignatureVerificationFailed
			);

			Ok(())
		}

		fn compute_signing_root(beacon_header: BeaconBlockHeader, domain: Domain) -> Result<Root, DispatchError> {
			let beacon_header_root = merklization::hash_tree_root_beacon_header(beacon_header).map_err(|_| DispatchError::Other("Beacon header hash tree root failed"))?;

			let hash_root = merklization::hash_tree_root_signing_data(SigningData {
				object_root: beacon_header_root.into(),
				domain,
			}).map_err(|_| DispatchError::Other("Signing root hash tree root failed"))?;

			Ok(hash_root.into())
		}

		fn verify_sync_committee(sync_committee: SyncCommittee, sync_committee_branch: ProofBranch, header_state_root: H256, depth: u64, index: u64) -> DispatchResult {
			let sync_committee_root = merklization::hash_tree_root_sync_committee(sync_committee).map_err(|_| DispatchError::Other("Sync committee hash tree root failed"))?;

			let mut branch =  Vec::<H256>::new();

			for vec_branch in sync_committee_branch.iter() {
				branch.push(H256::from_slice(vec_branch.as_slice()));
			}

			ensure!(
				Self::is_valid_merkle_branch(
					sync_committee_root.into(),
					branch,
					depth,
					index,
					header_state_root
				),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

			Ok(())
		}

		fn verify_header(header: BeaconBlockHeader, proof_branch: ProofBranch, attested_header_state_root: H256, depth: u64, index: u64) -> DispatchResult {
			let leaf = merklization::hash_tree_root_beacon_header(header).map_err(|_| DispatchError::Other("Header hash tree root failed"))?;

			let mut branch =  Vec::<H256>::new();

			for vec_branch in proof_branch.iter() {
				branch.push(H256::from_slice(vec_branch.as_slice()));
			}

			ensure!(
				Self::is_valid_merkle_branch(
					leaf.into(),
					branch,
					depth,
					index,
					attested_header_state_root
				),
				Error::<T>::InvalidHeaderMerkleProof
			);

			Ok(())
		}

		fn store_header(header: BeaconBlockHeader) {
			<FinalizedHeaders<T>>::insert(header.body_root.clone(), header.clone());

			<FinalizedHeadersBySlot<T>>::insert(header.slot, header.body_root);
		}

		/// Sums the bit vector of sync committee particpation.
		/// 
		/// # Examples
		/// 
		/// let sync_committee_bits = vec![0, 1, 0, 1, 1, 1];
		/// ensure!(get_sync_committee_sum(sync_committee_bits), 4);
		pub(super) fn get_sync_committee_sum(sync_committee_bits: Vec<u8>) -> u64 {
			sync_committee_bits.iter().fold(0, |acc: u64, x| acc + *x as u64)
		}

		/// Return the domain for the domain_type and fork_version.
		pub(super) fn compute_domain(
			domain_type: Vec<u8>,
			fork_version: Option<ForkVersion>,
			genesis_validators_root: Root,
		) -> Result<Domain, DispatchError> {
			let unwrapped_fork_version: ForkVersion;
			if fork_version.is_none() {
				unwrapped_fork_version = GENESIS_FORK_VERSION;
			} else {
				unwrapped_fork_version = fork_version.unwrap();
			}
			// TODO this may not be needed because we pass genesis_validators_root from relayer.
			//if genesis_validators_root is None:
			//	genesis_validators_root = Root()  # all bytes zero by default

			let fork_data_root =
				Self::compute_fork_data_root(unwrapped_fork_version, genesis_validators_root)?;

			let mut domain = [0u8; 32];

			domain[0..4].copy_from_slice(&(domain_type));
			domain[4..32].copy_from_slice(&(fork_data_root.0[..28]));

			Ok(domain.into())
		}

		fn compute_fork_data_root(current_version: ForkVersion, genesis_validators_root: Root) -> Result<Root, DispatchError> {		
			let hash_root = merklization::hash_tree_root_fork_data(ForkData {
				current_version,
				genesis_validators_root: genesis_validators_root.into(),
			}).map_err(|_| DispatchError::Other("Fork data hash tree root failed"))?;

			Ok(hash_root.into())
		}

		pub(super) fn is_valid_merkle_branch(
			leaf: H256,
			branch: Vec<H256>,
			depth: u64,
			index: u64,
			root: Root,
		) -> bool {
			let mut value = leaf;
			for i in 0..depth {
				if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
					// left node
					let mut data = [0u8; 64];
					data[0..32].copy_from_slice(&(value.0));
					data[32..64].copy_from_slice(&(branch[i as usize].0));
					value = sha2_256(&data).into();
				} else {
					let mut data = [0u8; 64]; // right node
					data[0..32].copy_from_slice(&(branch[i as usize].0));
					data[32..64].copy_from_slice(&(value.0));
					value = sha2_256(&data).into();
				}
			}
			return value == root;
		}

		pub(super) fn convert_to_binary(input: Vec<u8>) -> Vec<u8> {
			let mut result = Vec::new();

			for input_decimal in input.iter() {
				let mut tmp = Vec::new();

				let mut remaining = *input_decimal;

				while remaining != 0 {
					let remainder = remaining % 2;
					tmp.push(remainder);
					remaining = remaining / 2;
				}
				
				tmp.reverse();

				result.append(&mut tmp);
			}
			 
			result
		}
	}
}
