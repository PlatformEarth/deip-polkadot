//! # Deip Module
//! A module for managing digital assets.
//!
//! - [`multisig::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! This module contains functionality for managing different types of digital assets. 
//!
//! It provides a hierarchy to simply operate digital assets in the real world. 
//! The module contains entities Project and  Content of the Project with belongs to multi Account aka Team. 
//!
//! Besides, the Module provides Proof of share functionality. Proof of Share is a term we 
//! use for a special cryptographic proof that a sender actually sent, and the receiver 
//! has actually received an encrypted payload and a key to decrypt it. Please refer to the attached image.
//! Includes entities like NDA and NDA Access Request.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `add_domain` - Add cryptographic hash of Domain
//! * `create_project` - Create Project belongs to Account (Team)
//! * `update_project` - Update Project info
//! * `create_project_content` - Create Project Content (Digital Asset)
//! * `create_project_nda` - Create NDA contract between sides
//! * `create_nda_content_access_request` - Some side request access to the data of contract
//! * `fulfill_nda_content_access_request` - Granter fulfill access request to the data
//! * `reject_nda_content_access_request` - Granter reject access request to the data
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    codec::{Decode, Encode}, debug, ensure,
    decl_module, decl_storage, decl_event, decl_error, 
    StorageMap,
    dispatch::{ DispatchResult }
};
use frame_system::{ self as system, ensure_signed };
use sp_std::vec::Vec;
use sp_runtime::{ RuntimeDebug };
use sp_core::{ H160 };

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// A maximum number of Domains. When domains reaches this number, no new domains can be added.
pub const MAX_DOMAINS: u32 = 100;

/// Possible statuses of Project inherited from Project Content type
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
enum ProjectContentType {
    Announcement,
    FinalResult,
    MilestoneArticle,
    MilestoneBook,
    MilestoneChapter,
    MilestoneCode,
    MilestoneConferencePaper,
    MilestoneCoverPage,
    MilestoneData,
    MilestoneExperimentFindings,
    MilestoneMethod,
    MilestoneNegativeResults,
    MilestonePatent,
    MilestonePoster,
    MilestonePreprint,
    MilestonePresentation,
    MilestoneRawData,
    MilestoneResearchProposal,
    MilestoneTechnicalReport,
    MilestoneThesis,
}


impl Default for ProjectContentType {
    fn default() -> ProjectContentType { ProjectContentType::Announcement }
}

/// Configuration trait. Pallet depends on frame_system and pallet_timestamp. 
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

/// Unique Project ID reference
pub type ProjectId = H160;
/// Unique Domain reference
pub type Domain = H160;
/// Unique Project Contnt reference 
pub type ProjectContentId = H160;
/// Unique NDA reference 
pub type NdaId = H160;
/// Unique NdaAccess Request reference 
pub type NdaAccessRequestId = H160;

pub type ProjectOf<T> = Project<<T as system::Config>::Hash, <T as system::Config>::AccountId>;
pub type NdaOf<T> = Nda<<T as system::Config>::Hash, <T as system::Config>::AccountId, <T as pallet_timestamp::Config>::Moment>;
pub type NdaAccessRequestOf<T> = NdaAccessRequest<<T as system::Config>::Hash, <T as system::Config>::AccountId>;
pub type ProjectContentOf<T> = ProjectContent<<T as system::Config>::Hash, <T as system::Config>::AccountId>;

/// Core entity of pallet. Everything connected to Project. 
/// Only Account (Team) stand before Project in hierarchy.
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Project<Hash, AccountId> {
    /// Determine visible project or not 
    is_private: bool,
    /// Reference for external world and uniques control 
    external_id: ProjectId,
    /// Reference to the Team 
    team_id: AccountId,
    /// Hash of Project description
    description: Hash,
    /// List of Domains aka tags Project matches
    domains: Vec<Domain>,
    /// List of Project Members. Determine who is participated in the project
    members: Vec<AccountId>
}

/// Digital asset. Contains information of content and authors of Digital asset.
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct ProjectContent<Hash, AccountId> {
    /// Reference for external world and uniques control 
    external_id: ProjectContentId,
    /// Reference to the Project 
    project_external_id: ProjectId,
    /// Reference to the Team
    team_id: AccountId,
    /// Type of content. Determine status of Project
    content_type: ProjectContentType,
    /// Hash of the content ddescription
    description: Hash,
    /// Hast of digital asset
    content: Hash,
    /// Authors of Digital asset
    authors: Vec<AccountId>,
    /// List of References to other digital assets whith will be used in current digital asset.
    references: Option<Vec<ProjectContentId>>
    
}

/// NDA contract between parties. Usually about dislocating or not dislocating some confidential info
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct Nda<Hash, AccountId, Moment>  {
    /// Reference to Multisig Account with involved parties
    contract_creator: AccountId,
    /// Reference for external world and uniques control 
    external_id: NdaId,
    /// Unix Timestamp. Exparation date of contract
    end_date: Moment,
    /// Unix Timestamp. Entry into force of the contract
    start_date: Option<Moment>,
    /// Hash of the contract
    contract_hash: Hash,
    /// Involved Parties
    parties: Vec<AccountId>,
    /// Involved Projects 
    projects: Vec<ProjectId>,
}

/// Statuses of NDA access requests
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
enum NdaAccessRequestStatus {
    Pending,
    Fulfilled,
    Rejected,
}


impl Default for NdaAccessRequestStatus {
    fn default() -> NdaAccessRequestStatus { NdaAccessRequestStatus::Pending }
}

/// NDA access request. One of the partice may decide to request to receive 
/// some info included into contract. Holder should fulfill or reject this request. 
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct NdaAccessRequest<Hash, AccountId>  {
    /// Reference for external world and uniques control 
    external_id: NdaAccessRequestId,
    /// Reference to NDA 
    nda_external_id: NdaId,
    /// Reference to Requester (creator of this request)
    requester: AccountId,
    /// Payload witch need to be decrypted
    encrypted_payload_hash: Hash,
    /// IV of encrypted payload
    encrypted_payload_iv: Vec<u8>,
    /// Execution status
    status: NdaAccessRequestStatus,
    /// Reference to access granter if approved
    grantor: Option<AccountId>,
    /// Ecrypted key witch can decrypt payload
    encrypted_payload_encryption_key: Option<Vec<u8>>,
    /// Proof that requester has access to the encrypted data with his key 
    proof_of_encrypted_payload_encryption_key: Option<Vec<u8>>,
}

decl_event! {
    /// Events type.
    pub enum Event<T> 
    where 
        AccountId = <T as frame_system::Config>::AccountId,
        Project = ProjectOf<T>,
    {
        // ==== Projects ====

        /// Event emitted when a project has been created. [BelongsTo, Project]
        ProjectCreated(AccountId, Project),
        /// Event emitted when a project is removed by the owner. [BelongsTo, Project]
        ProjectRemoved(AccountId, Project),
        /// Event emitted when a project is removed by the owner. [BelongsTo, ProjectId]
        ProjectUpdated(AccountId, ProjectId),

        // ==== Project Content ====
       
        /// Event emitted when a project contnet has been created. [BelongsTo, ProjectContentId]
        ProjectContnetCreated(AccountId, ProjectContentId),

        // ==== NDA ====
       
        /// Event emitted when a NDA has been created. [BelongsTo, NdaId]
        NdaCreated(AccountId, NdaId),
        /// Event emitted when a NDA Access request has been created. [BelongsTo, NdaAccessRequestId]
        NdaAccessRequestCreated(AccountId, NdaAccessRequestId),
        //  /// Event emitted when a NDA Access request has been fulfilled. [BelongsTo, NdaAccessRequestId]
        NdaAccessRequestFulfilled(AccountId, NdaAccessRequestId),
        //  /// Event emitted when a NDA Access request has been rejected. [BelongsTo, NdaAccessRequestId]
        NdaAccessRequestRejected(AccountId, NdaAccessRequestId),

        /// Added a domain. [Creator, Domain]
		DomainAdded(AccountId, Domain),
    }
}

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        // ==== Projects ====
        
        /// The project does not exist.
        NoSuchProject,
        /// The project is created by another account, so caller can't remove it.
        NotProjectOwner,
        /// Cannot add domain into the porject because this domain not exists
        DomainNotExists,
        /// Cannot add a project because a project with this ID is already a exists
		ProjectAlreadyExists,

        // ==== Project Content ====
       
        /// Cannot add a project content because a project content with this ID is already a exists.
        ProjectContentAlreadyExists,
        /// Project does not belong to the team.
        ProjectNotBelongToTeam,
        /// The Reference does not exist.
        NoSuchReference, 
        /// Cannot add a project content because a project with this ID is already a finished
		ProjectAlreadyFinished,


        // ==== Domains ====
        
        /// Cannot add another domain because the limit is already reached
        DomianLimitReached,
        /// Cannot add domain because this domain is already a exists
        DomainAlreadyExists,

        // ==== NDA ====
        
        /// Cannot add a NDA because a NDA with this ID is already a exists.
        NdaAlreadyExists,
        /// Nda Access Request with this ID is  already a exists.
        NdaAccessRequestAlreadyExists,
        /// The NDA with this ID does not exist.
        NoSuchNda,
        /// The NDA Access Request with this ID does not exist.
        NoSuchNdaAccessRequest,
        /// The start of the contract has not yet arrived, so contract can't be fulfilled or rejected
        NdaContractIsNotActiveYet,
        /// NDA start date must be later or equal current moment
        NdaStartDateMustBeLaterOrEqualCurrentMoment,
        /// NDA end date must be later current moment
        NdaEndDateMustBeLaterCurrentMoment,
        /// NDA start date must be less than end date
        NdaStartDateMustBeLessThanEndDate,
        /// Team of all projects must specified as party
        TeamOfAllProjectsMustSpecifiedAsParty,
        /// Nda access request already finalized
        NdaAccessRequestAlreadyFinalized,

        
        
        // ==== General =====

        /// Access Forbiten
        NoPermission,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Deip {
        /// Map from ProjectID to Project Info
        ProjectMap get(fn project): map hasher(identity) ProjectId => ProjectOf<T>;
        /// Project list, guarantees uniquest and provides Project listing
        Projects get(fn projects): Vec<(ProjectId, T::AccountId)>;

        /// Map to Project Content Info
        ProjectContentMap get(fn project_content_entity): double_map hasher(identity) ProjectId, hasher(identity) ProjectContentId => ProjectContentOf<T>;
        /// Project Content list, guarantees uniquest and provides Project Conent listing
        ProjectsContent get(fn project_content_list): Vec<(ProjectContentId, ProjectId, T::AccountId)>;

        /// NDA list, guarantees uniquest and provides NDA listing
        Ndas get(fn nda_list): Vec<(ProjectId, T::AccountId)>;
        /// Map to NDA Info
        NdaMap get(fn nda): map hasher(identity) NdaId => NdaOf<T>;
        
        /// NDA Access Requests list, guarantees uniquest and provides NDA Access Requests listing
        NdaAccessRequests get(fn nda_requests): Vec<(NdaAccessRequestId, NdaId, T::AccountId)>;
        /// Map to NDA Access Requests Info
        NdaAccessRequestMap get(fn nda_request): map hasher(identity) NdaAccessRequestId => NdaAccessRequestOf<T>;

        // The set of all Domains.
        Domains get(fn domains) config(): map hasher(blake2_128_concat) Domain => ();
        // The total number of domains stored in the map.
        // Because the map does not store its size, we must store it separately
        DomainCount get(fn domain_count) config(): u32;
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;
       
        /// Allow a user to create project.
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `project`: [Project](./struct.Project.html) to be created.
        #[weight = 10_000]
        fn create_project(origin, project: ProjectOf<T>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let account = ensure_signed(origin)?;
            
            for domain in &project.domains {
                ensure!(Domains::contains_key(&domain), Error::<T>::DomainNotExists);
            }

            let mut projects = Projects::<T>::get();

            // We don't want to add duplicate projects, so we check whether the potential new
			// project is already present in the list. Because the list is always ordered, we can
			// leverage the binary search which makes this check O(log n).
			match projects.binary_search_by_key(&project.external_id, |&(a,_)| a) {
				// If the search succeeds, the project is already a exists, so just return
				Ok(_) => return Err(Error::<T>::ProjectAlreadyExists.into()),
				// If the search fails, the project is not a exists and we learned the index where
				// they should be inserted
				Err(index) => {
					projects.insert(index, (project.external_id, project.team_id.clone()));
					Projects::<T>::put(projects);
				}
			};

            // Store the projects related to account
            ProjectMap::<T>::insert(project.external_id, project.clone());

            // Emit an event that the project was created.
            Self::deposit_event(RawEvent::ProjectCreated(account, project));
        }

        /// Allow a user to update project.
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `project_id`: [Project]((./struct.Project.html)) identifier (external_id) to be updated
        /// - `description`: Optional. Hash of description
        /// - `is_private`: Optional.  Determine visible project or not 
        /// - `members`: Optional.  New set of members
        #[weight = 10_000]
        fn update_project(origin, project_id: ProjectId, description: Option<T::Hash>, is_private: Option<bool>, members: Option<Vec<T::AccountId>>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let account = ensure_signed(origin)?;

            ProjectMap::<T>::mutate_exists(project_id, |maybe_project| -> DispatchResult {
                let project = maybe_project.as_mut().ok_or(Error::<T>::NoSuchProject)?;

                ensure!(project.team_id == account, Error::<T>::NoPermission);

                // TODO make sure that we don't lose first 2 bytes of the hash
                if let Some(value) = description  {
                    project.description = value;
                }

                if let Some(value) = is_private  {
                    project.is_private = value;
                }
                
                if let Some(value) = members  {
                    project.members = value;
                }

                Ok(())
            })?;

            // Emit an event that the project was updated.
            Self::deposit_event(RawEvent::ProjectUpdated(account, project_id));

            Ok(())
        }

        /// Allow a user to create project content.
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `content`: [Content](./struct.ProjectContent.html) to be created
        #[weight = 10_000]
        fn create_project_content(origin, content: ProjectContentOf<T>) {
            let account = ensure_signed(origin)?;

            let mut project_content = ProjectsContent::<T>::get();

            let index_to_insert_content = project_content.binary_search_by_key(&content.external_id, |&(a,_, _)| a)
                .err().ok_or(Error::<T>::ProjectContentAlreadyExists)?;

            let project = ProjectMap::<T>::get(content.project_external_id);

            ensure!(!project.external_id.is_zero(), Error::<T>::NoSuchProject);
            ensure!(project.team_id == content.team_id, Error::<T>::ProjectNotBelongToTeam);
            ensure!(!Self::is_project_finished(&project.external_id), Error::<T>::ProjectAlreadyFinished);


            if let Some(references) = &content.references {
                let is_all_references_exists = references
                    .iter()
                    .all(|&reference| project_content.binary_search_by_key(&reference,|&(id,_, _)| id).is_ok());

                ensure!(is_all_references_exists, Error::<T>::NoSuchReference);
            }

            project_content.insert(index_to_insert_content, (content.external_id, content.project_external_id,  content.team_id.clone()));
            ProjectsContent::<T>::put(project_content);

            // Store the content
            ProjectContentMap::<T>::insert(project.external_id, content.external_id, content.clone());

            // Emit an event that the content was created.
            Self::deposit_event(RawEvent::ProjectContnetCreated(account, content.external_id));
        }

        /// Allow a user to create [NDA](./struct.Nda.html).
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `end_date`: Unix Timestamp. Exparation date of contract
        /// - `contract_hash`: Hash of the contract
        /// - `maybe_start_date`: Optional. Unix Timestamp. Entry into force of the contract
        /// - `parties`: List of involved Parties
        /// - `projects`: List of involved Projects
        #[weight = 10_000]
        fn create_project_nda(origin,  
            external_id: NdaId,
            end_date: T::Moment,
            contract_hash: T::Hash,
            maybe_start_date: Option<T::Moment>,
            parties: Vec<T::AccountId>,
            projects: Vec<ProjectId>
        ) {
            let mut projects = projects;
            projects.dedup();
            let contract_creator = ensure_signed(origin)?;
            let timestamp = pallet_timestamp::Module::<T>::get();

            ensure!(end_date > timestamp, Error::<T>::NdaEndDateMustBeLaterCurrentMoment);

            if let Some(start_date) = maybe_start_date {
                ensure!(start_date >= timestamp, Error::<T>::NdaStartDateMustBeLaterOrEqualCurrentMoment);
                ensure!(end_date > start_date, Error::<T>::NdaStartDateMustBeLessThanEndDate);
            }
            
            projects.iter()
                .try_for_each(|id| -> DispatchResult {
                    let project = ProjectMap::<T>::get(id);

                    ensure!(!project.external_id.is_zero(), Error::<T>::NoSuchProject);
                    ensure!(parties.contains(&project.team_id), Error::<T>::TeamOfAllProjectsMustSpecifiedAsParty);

                    Ok(())
                })?;

            let mut nda_list = Ndas::<T>::get();

            let index_to_insert_nda = nda_list.binary_search_by_key(&external_id, |&(external_id, ..)| external_id)
                .err().ok_or(Error::<T>::NdaAlreadyExists)?;
            
               
            let nda = Nda {
                contract_creator: contract_creator.clone(),
                external_id,
                end_date,
                start_date: maybe_start_date,
                contract_hash,
                parties,
                projects
            };
            
            nda_list.insert(index_to_insert_nda, (nda.external_id, contract_creator.clone()));
            Ndas::<T>::put(nda_list);

            NdaMap::<T>::insert(nda.external_id, nda);

            // Emit an event that the NDA was created.
            Self::deposit_event(RawEvent::NdaCreated(contract_creator, external_id));

        }

        /// Create [request](./struct.NdaAccessRequest.html) to access NDA content
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `external_id`: Reference for external world and uniques control 
        /// - `nda_external_id`: Reference to NDA 
        /// - `encrypted_payload_hash`: Payload witch need to be decrypted
        /// - `encrypted_payload_iv`: IV of encrypted payload
        #[weight = 10_000]
        fn create_nda_content_access_request(
            origin, 
            external_id: NdaAccessRequestId,
            nda_external_id: NdaId,
            encrypted_payload_hash: T::Hash,
            encrypted_payload_iv: Vec<u8>,
        ) {
            let account = ensure_signed(origin)?;
            let timestamp = pallet_timestamp::Module::<T>::get();

            let nda = NdaMap::<T>::get(nda_external_id);
            
            ensure!(!nda.external_id.is_zero(), Error::<T>::NoSuchNda);
            ensure!(nda.start_date <= Some(timestamp), Error::<T>::NdaContractIsNotActiveYet);

            let mut nda_requests = NdaAccessRequests::<T>::get();

            let index_to_insert_nda_request = nda_requests.binary_search_by_key(&external_id, |&(external_id, ..)| external_id)
                .err().ok_or(Error::<T>::NdaAccessRequestAlreadyExists)?;
            
            let nda_request = NdaAccessRequest {
                external_id,
                nda_external_id, 

                requester: account.clone(),
                encrypted_payload_hash,
                encrypted_payload_iv,
                status: NdaAccessRequestStatus::Pending,
                grantor: None,
                encrypted_payload_encryption_key: None,
                proof_of_encrypted_payload_encryption_key: None,
            };
            nda_requests.insert(index_to_insert_nda_request, (external_id, nda_external_id, account.clone()));
            NdaAccessRequests::<T>::put(nda_requests);

            NdaAccessRequestMap::<T>::insert(nda_request.external_id, nda_request);

            // Emit an event that the NDA was created.
            Self::deposit_event(RawEvent::NdaAccessRequestCreated(account, external_id));
            

        }
        
        /// Fulfill NDA access request
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `external_id`: Reference for external world and uniques control 
        /// - `encrypted_payload_encryption_key`: Ecrypted key witch can decrypt payload
        /// - `proof_of_encrypted_payload_encryption_key`: Proof that requester has access to the encrypted data with his key 
        #[weight = 10_000]
        fn fulfill_nda_content_access_request(
            origin, 
            external_id: NdaAccessRequestId,
            encrypted_payload_encryption_key: Vec<u8>,
            proof_of_encrypted_payload_encryption_key: Vec<u8>,
        ) {
            let account = ensure_signed(origin)?;

            NdaAccessRequestMap::<T>::mutate_exists(external_id, |maybe_nda_access_request| -> DispatchResult {
                let mut nda_access_request = maybe_nda_access_request.as_mut().ok_or(Error::<T>::NoSuchNdaAccessRequest)?;

                ensure!(nda_access_request.status == NdaAccessRequestStatus::Pending, Error::<T>::NdaAccessRequestAlreadyFinalized);
                ensure!(NdaMap::<T>::contains_key(nda_access_request.nda_external_id), Error::<T>::NoSuchNda);

                nda_access_request.status = NdaAccessRequestStatus::Fulfilled;
                nda_access_request.grantor = Some(account.clone());
                nda_access_request.encrypted_payload_encryption_key = Some(encrypted_payload_encryption_key);
                nda_access_request.proof_of_encrypted_payload_encryption_key = Some(proof_of_encrypted_payload_encryption_key);

                Ok(())
            })?;

            // Emit an event that the NDA was fulfilled.
            Self::deposit_event(RawEvent::NdaAccessRequestFulfilled(account, external_id));

        }

        /// Reject NDA access request
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `external_id`: Reference for external world and uniques control 
         #[weight = 10_000]
         fn reject_nda_content_access_request(
             origin, 
             external_id: NdaAccessRequestId,
         ) {
             let account = ensure_signed(origin)?;
 
             NdaAccessRequestMap::<T>::mutate_exists(external_id, |maybe_nda_access_request| -> DispatchResult {
                let mut nda_access_request = maybe_nda_access_request.as_mut().ok_or(Error::<T>::NoSuchNdaAccessRequest)?;
                
                
                ensure!(nda_access_request.status == NdaAccessRequestStatus::Pending, Error::<T>::NdaAccessRequestAlreadyFinalized);
                ensure!(NdaMap::<T>::contains_key(nda_access_request.nda_external_id), Error::<T>::NoSuchNda);
 
                nda_access_request.status = NdaAccessRequestStatus::Rejected;
                 
                Ok(())
             })?;
 
             // Emit an event that the NDA was rejected.
             Self::deposit_event(RawEvent::NdaAccessRequestRejected(account, external_id));
 
         }
        
        /// Allow a user to create domains.
        ///
		/// The origin for this call must be _Signed_. 
        ///
		/// - `external_id`: Domain reference
        #[weight = 10_000]
        fn add_domain(origin, domain: Domain) {
            let account = ensure_signed(origin)?;
        
            let domain_count = DomainCount::get();
            ensure!(domain_count < MAX_DOMAINS, Error::<T>::DomianLimitReached);
        
            // We don't want to add duplicate domains, so we check whether the potential new
            // domain is already present in the list. Because the domains is stored as a hash
            // map this check is constant time O(1)
            ensure!(!Domains::contains_key(&domain), Error::<T>::DomainAlreadyExists);
        
            // Insert the new domin and emit the event
            Domains::insert(&domain, ());
            DomainCount::put(domain_count + 1); // overflow check not necessary because of maximum
            Self::deposit_event(RawEvent::DomainAdded(account, domain));
        }
    }
}

impl<T: Config> Module<T> {
	fn is_project_finished(project_id: &ProjectId) -> bool {
		ProjectContentMap::<T>::iter_prefix_values(project_id)
            .any(|x| x.content_type == ProjectContentType::FinalResult)
	}
}
