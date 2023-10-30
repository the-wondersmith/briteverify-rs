#![allow(dead_code)]
//! ## Single-Transaction / Real-Time Request/Response Body Pairs From The Official
//! ## [BriteVerify API Docs](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f)

// Crate-Level Imports
use super::MockRequestResponse;

// <editor-fold desc="// Email Only ...">

pub const OFFICIAL_EMAIL_VALID: MockRequestResponse = MockRequestResponse {
    request: r#"{"email":"sales@validity.com"}"#,
    response: r#"{
  "email": {
    "address": "sales@validity.com",
    "account": "sales",
    "domain": "validity.com",
    "status": "valid",
    "connected": null,
    "disposable": false,
    "role_address": true
  },
  "duration": 0.035602396
}"#,
};
pub const OFFICIAL_EMAIL_INVALID: MockRequestResponse = MockRequestResponse {
    request: r#"{"email":"invalidtest@validity.com"}"#,
    response: r#"{
  "email": {
    "address": "invalidtest@validity.com",
    "account": "invalidtest",
    "domain": "validity.com",
    "status": "invalid",
    "connected": null,
    "disposable": false,
    "role_address": false,
    "error_code": "email_account_invalid",
    "error": "Email account invalid"
  },
  "duration": 0.291414519
}"#,
};
pub const OFFICIAL_EMAIL_DISPOSABLE: MockRequestResponse = MockRequestResponse {
    request: r#"{"email":"fake@mailinator.com"}"#,
    response: r#"{
  "email": {
    "address": "fake@mailinator.com",
    "account": "fake",
    "domain": "mailinator.com",
    "status": "accept_all",
    "connected": null,
    "disposable": true,
    "role_address": false
  },
  "duration": 0.081746428
}"#,
};

// </editor-fold desc="// Email Only ...">

// <editor-fold desc="// Phone Only ...">

pub const OFFICIAL_PHONE_VALID: MockRequestResponse = MockRequestResponse {
    request: r#"{"phone":"18009618205"}"#,
    response: r#"{
  "phone": {
    "number": "18009618205",
    "service_type": "land",
    "phone_location": null,
    "status": "valid",
    "errors": []
  },
  "duration": 0.032635276
}"#,
};
pub const OFFICIAL_PHONE_INVALID: MockRequestResponse = MockRequestResponse {
    request: r#"{"phone":"135315"}"#,
    response: r#"{
  "phone": {
    "number": "135315",
    "service_type": null,
    "phone_location": null,
    "status": "invalid",
    "errors": [
      "invalid_phone_number"
    ]
  },
  "duration": 0.056220326
}"#,
};

// </editor-fold desc="// Phone Only ...">

// <editor-fold desc="// Address Only ...">

pub const OFFICIAL_ADDRESS_VALID: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "address": {
    "address1": "4010 W Boy Scout Blvd Ste 1100",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5796"
  }
}"#,
    response: r#"{
  "address": {
    "address1": "4010 W Boy Scout Blvd Ste 1100",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5796",
    "status": "valid",
    "errors": [],
    "corrected": false
  },
  "duration": 0.108034192
}"#,
};
pub const OFFICIAL_ADDRESS_CORRECTED: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "address": {
    "address1": "4010 Boy Scout Boulevard, Suite 1100",
    "city": "Tampa",
    "state": "FL",
    "zip": "33605"
  }
}"#,
    response: r#"{
  "address": {
    "address1": "4010 W Boy Scout Blvd Ste 1100",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5796",
    "status": "valid",
    "errors": [],
    "corrected": true
  },
  "duration": 0.084670758
}"#,
};
pub const OFFICIAL_ADDRESS_MISSING_SUITE: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "address": {
    "address1": "4010 Boy Scout Boulevard",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607"
  }
}"#,
    response: r#"{
  "address": {
    "address1": "4010 W Boy Scout Blvd",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5727",
    "status": "invalid",
    "errors": [
      "suite_missing"
    ],
    "corrected": true
  },
  "duration": 1.589391364
}"#,
};
pub const OFFICIAL_ADDRESS_UNKNOWN_STREET: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "address": {
    "address1": "4010 Girl Scout Ave",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607"
  }
}"#,
    response: r#"{
  "address": {
    "address1": "4010 Girl Scout Ave",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607",
    "status": "invalid",
    "errors": [
      "unknown_street"
    ],
    "corrected": false
  },
  "duration": 0.657597369
}"#,
};

// </editor-fold desc="// Address Only ...">

// <editor-fold desc="// Email, Phone, & Address ...">

pub const OFFICIAL_VALID_FULL_VERIFY: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "email": "sales@validity.com",
  "phone": "18009618205",
  "address": {
    "address1": "4010 Boy Scout Boulevard, Suite 1100",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607"
  }
}"#,
    response: r#"{
  "email": {
    "address": "sales@validity.com",
    "account": "sales",
    "domain": "validity.com",
    "status": "valid",
    "connected": null,
    "disposable": false,
    "role_address": true
  },
  "phone": {
    "number": "18009618205",
    "service_type": "land",
    "phone_location": null,
    "status": "valid",
    "errors": []
  },
  "address": {
    "address1": "4010 W Boy Scout Blvd Ste 1100",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5796",
    "status": "valid",
    "errors": [],
    "corrected": true
  },
  "duration": 2.388141409
}"#,
};
pub const OFFICIAL_INVALID_FULL_VERIFY: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "email": "nmijvnuiwfmuirfnw@gwgwegew.com",
  "phone": "1-800-961-8205",
  "address": {
    "address1": "4010 Boy Scout Boulevard, Suite 1100",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607"
  }
}"#,
    response: r#"{
  "email": {
    "address": "nmijvnuiwfmuirfnw@gwgwegew.com",
    "account": "nmijvnuiwfmuirfnw",
    "domain": "gwgwegew.com",
    "status": "invalid",
    "connected": null,
    "disposable": false,
    "role_address": false,
    "error_code": "email_domain_invalid",
    "error": "Email domain invalid"
  },
  "phone": {
    "number": "18009618205",
    "service_type": "land",
    "phone_location": null,
    "status": "valid",
    "errors": []
  },
  "address": {
    "address1": "4010 W Boy Scout Blvd Ste 1100",
    "address2": " ",
    "city": "Tampa",
    "state": "FL",
    "zip": "33607-5796",
    "status": "valid",
    "errors": [],
    "corrected": true
  },
  "duration": 0.285692082
}"#,
};

// </editor-fold desc="// Email, Phone, & Address ...">
