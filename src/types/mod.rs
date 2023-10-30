//! ## Type Definitions

pub mod account;
pub mod bulk;
pub mod enums;
pub mod single;

pub use self::{
    account::AccountCreditBalance,
    bulk::{
        BulkContactVerificationResult, BulkListCRUDError, BulkListCRUDResponse,
        BulkVerificationRequest, BulkVerificationResponse, BulkVerificationResult,
        CreateListResponse, DeleteListResponse, GetListStatesResponse, UpdateListResponse,
        VerificationListState,
    },
    enums::{BatchState, BulkListDirective, VerificationError, VerificationStatus},
    single::{
        AddressArrayBuilder, AddressVerificationArray, EmailVerificationArray,
        PhoneNumberVerificationArray, StreetAddressArray, VerificationRequest,
        VerificationRequestBuilder, VerificationResponse,
    },
};
