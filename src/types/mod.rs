//! ## Type Definitions

pub mod account;
pub mod bulk;
pub mod enums;
pub mod single;

pub use self::{
    account::AccountCreditBalance,
    bulk::{
        BulkAddressVerificationResult, BulkContactVerificationResult, BulkEmailVerificationResult,
        BulkPhoneNumberVerificationResult, BulkVerificationRequest, BulkVerificationResponse,
        BulkVerificationResult, CreateListErrorResponse, CreateListResponse,
        CreateListSuccessResponse, DeleteListErrorResponse, DeleteListResponse,
        DeleteListSuccessResponse, GetListStatesResponse, UpdateListResponse,
        VerificationListErrorMessage, VerificationListState,
    },
    enums::{BatchCreationStatus, BatchState, BulkListDirective, VerificationStatus},
    single::{
        AddressArrayBuilder, AddressVerificationArray, AddressVerificationRequest,
        AddressVerificationResponse, EmailAndAddressVerificationRequest,
        EmailAndAddressVerificationResponse, EmailAndPhoneVerificationRequest,
        EmailAndPhoneVerificationResponse, EmailVerificationArray, EmailVerificationRequest,
        EmailVerificationResponse, FullVerificationRequest, FullVerificationResponse,
        PhoneAndAddressVerificationRequest, PhoneAndAddressVerificationResponse,
        PhoneNumberVerificationArray, PhoneNumberVerificationRequest,
        PhoneNumberVerificationResponse, StreetAddressArray, VerificationRequest,
        VerificationRequestBuilder, VerificationResponse,
    },
};
