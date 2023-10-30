//! ## Bulk Verification Request/Response Body Pairs From The Official
//! ## [BriteVerify API Docs](https://docs.briteverify.com/#382f454d-dad2-49c3-b320-c7d117fcc20a)

// Crate-Level Imports
use super::MockRequestResponse;

// <editor-fold desc="// Create Bulk Verification List ...">

pub const ERROR_INVALID_DIRECTIVE: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "directive": "begin",
  "contacts": [
    {
      "phone": "4444444444",
      "email": "hello@example.com",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "invalid_state",
  "message": "Directive must be one of `terminate` or `start`"
}
"#,
};
pub const ERROR_INVALID_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "emails": [
    "john.doe@email.com",
    "jane.doe@email.com"
  ],
  "contacts": [
    {
      "phone": "5555555555",
      "email": "example@mail.com"
    },
    {
      "phone": "6666666666",
      "email": "another-example@mail.com"
    }
  ]
}
"#,
    response: r#"{
  "status": "duplicate_data",
  "message": "Request has both emails and contacts params"
}
"#,
};
pub const ERROR_MISSING_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r#"{"contacts":[{}]}"#,
    response: r#"{
  "status": "missing_data",
  "message": "Request has missing or invalid emails or contacts array."
}
"#,
};
pub const ERROR_APPEND_LIST_NOT_FOUND: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "contacts": [
    {
      "phone": "4444444444",
      "email": "hello@example.com"
    }
  ]
}
"#,
    response: r#"{
  "status": "not_found",
  "message": "No matching list found."
}
"#,
};
pub const ERROR_TERMINATE_MISSING_LIST_ID: MockRequestResponse = MockRequestResponse {
    request: r#"{"directive":"terminate"}"#,
    response: r#"{
  "status": "missing_data",
  "message": "Cannot terminate a list without a valid list id"
}
"#,
};
pub const ERROR_INVALID_MISSING_EMAILS_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r#"{"emails":[{}]}"#,
    response: r#"{
  "status": "missing_data",
  "message": "Request has empty emails array"
}
"#,
};
pub const ERROR_INVALID_MISSING_CONTACTS_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r#"{"emails": [{}]}"#,
    response: r#"{
  "status": "missing_data",
  "message": "Request has empty emails array"
}
"#,
};

pub const OFFICIAL_CREATE_LIST: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "contacts": [
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    },
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "success",
  "message": "created new list",
  "list": {
    "id": "4432f157-28b9-4721-8add-2b48d70e968e",
    "state": "open",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 04:03 pm",
    "expiration_date": null,
    "results_path": null
  }
}
"#,
};
pub const OFFICIAL_VERIFY_LIST: MockRequestResponse = MockRequestResponse {
    request: r#"{"directive":"start"}"#,
    response: r#"{
  "status": "success",
  "message": "list queued for processing",
  "list": {
    "id": "8448cf63-fdcc-4870-bd04-17d9f5d240fa",
    "state": "verifying",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 16,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 05:08 pm",
    "expiration_date": null,
    "results_path": null
  }
}
"#,
};
pub const OFFICIAL_APPEND_TO_LIST: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "contacts": [
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    },
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "success",
  "message": "updated existing list",
  "list": {
    "id": "33fa061d-c40f-4f59-afd1-5f5dccff54c0",
    "state": "open",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 04:03 pm",
    "expiration_date": null,
    "results_path": null
  }
}
"#,
};
pub const OFFICIAL_TERMINATE_LIST: MockRequestResponse = MockRequestResponse {
    request: r#"{"directive":"terminate"}"#,
    response: r#"{
  "status": "success",
  "message": "terminated list",
  "list": {
    "id": "4a56548e-6c22-4119-b3c2-107985c6f29f",
    "state": "terminated",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 04:03 pm",
    "expiration_date": null,
    "results_path": null,
    "errors": [
      {
        "code": "import_error",
        "message": "user terminated at 08-10-2021 04:07PM"
      }
    ]
  }
}
"#,
};
pub const OFFICIAL_CREATE_VERIFY_LIST: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "directive": "start",
  "contacts": [
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    },
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "success",
  "message": "list queued for processing",
  "list": {
    "id": "44fdd8a1-c9dc-4049-bb67-b902185d0989",
    "state": "pending",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 03:58 pm",
    "expiration_date": null,
    "results_path": null
  }
}
"#,
};
pub const OFFICIAL_CREATE_LIST_WITH_EXTERNAL_ID: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "contacts": [
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    },
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "success",
  "message": "created new list",
  "list": {
    "id": "67d53139-1c4a-4c68-b83f-69569f9c5135",
    "state": "open",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 04:23 pm",
    "expiration_date": null,
    "results_path": null,
    "account_external_id": "12345"
  }
}
"#,
};
pub const OFFICIAL_CREATE_VERIFY_LIST_WITH_EXTERNAL_ID: MockRequestResponse = MockRequestResponse {
    request: r#"{
  "directive": "start",
  "contacts": [
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    },
    {
      "email": "hello@example.com",
      "phone": "4444444444",
      "address": {
        "address1": "200 Clarendon St",
        "address2": "Unit 2200",
        "city": "Boston",
        "state": "MA",
        "zip": "02115"
      }
    }
  ]
}
"#,
    response: r#"{
  "status": "success",
  "message": "list queued for processing",
  "list": {
    "id": "64cc9e3b-1890-4952-8cc4-d435e3ac1bf0",
    "state": "pending",
    "total_verified": 0,
    "total_verified_emails": 0,
    "total_verified_phones": 0,
    "page_count": 0,
    "progress": 0,
    "created_at": "08-10-2021 04:25 pm",
    "expiration_date": null,
    "results_path": null,
    "account_external_id": "12345"
  }
}
"#,
};

// </editor-fold desc="// Create Bulk Verification List ...">

// <editor-fold desc="// Get Bulk Verification Lists ...">

pub const OFFICIAL_GET_ALL_LISTS: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "lists": [
    {
      "id": "a2595a63-ae71-4dda-91d4-57bdb331aa3a",
      "state": "open",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": 0,
      "progress": 0,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": null,
      "results_path": null
    },
    {
      "id": "1433fe1c-cc4b-48fb-8989-d3ec83502c54",
      "state": "complete",
      "total_verified": 2,
      "total_verified_emails": 1,
      "total_verified_phones": 1,
      "page_count": 1,
      "progress": 100,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": "08-17-2021 04:26 pm",
      "results_path": "https://bulk-api.briteverify.com/api/v3/lists/1433fe1c-cc4b-48fb-8989-d3ec83502c54/export/1"
    },
    {
      "id": "288be984-2094-4925-8790-4ebfeab7d757",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}"#,
};
pub const OFFICIAL_LISTS_BY_DATE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "lists": [
    {
      "id": "9fda9ed3-2819-4ce0-9811-e2207d1b3da0",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}
"#,
};
pub const OFFICIAL_LISTS_BY_PAGE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "Page 2 of 2",
  "lists": [
    {
      "id": "97f01e89-0ae9-4f83-b70f-3ce9e0a1d87b",
      "state": "open",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": 0,
      "progress": 0,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": null,
      "results_path": null
    },
    {
      "id": "37312ed4-8af5-426f-99fb-17b3023c56cc",
      "state": "complete",
      "total_verified": 2,
      "total_verified_emails": 1,
      "total_verified_phones": 1,
      "page_count": 1,
      "progress": 100,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": "08-17-2021 04:26 pm",
      "results_path": "https://bulk-api.briteverify.com/api/v3/lists/37312ed4-8af5-426f-99fb-17b3023c56cc/export/1"
    },
    {
      "id": "303a74ea-2a03-40f7-b8b5-11ae88d13e3b",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}
"#,
};
pub const OFFICIAL_LISTS_BY_STATE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "lists": [
    {
      "id": "fb6c70e4-d6e9-43c1-84a4-790b4e090b00",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}
"#,
};
pub const OFFICIAL_NO_LISTS_FOUND: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{"lists":[]}"#,
};
pub const OFFICIAL_MULTIPLE_LIST_PAGES: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "Page 1 of 2",
  "lists": [
    {
      "id": "e0ba1b3f-dc6d-440e-ad3c-ab36a7885796",
      "state": "open",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": 0,
      "progress": 0,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": null,
      "results_path": null
    },
    {
      "id": "a3daba0e-e1d3-497f-b295-dfa4fe504710",
      "state": "complete",
      "total_verified": 2,
      "total_verified_emails": 1,
      "total_verified_phones": 1,
      "page_count": 1,
      "progress": 100,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": "08-17-2021 04:26 pm",
      "results_path": "https://bulk-api.briteverify.com/api/v3/lists/a3daba0e-e1d3-497f-b295-dfa4fe504710/export/1"
    },
    {
      "id": "accbf8b7-49f6-4f55-9677-707732be269a",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}
"#,
};
pub const OFFICIAL_LISTS_BY_EXTERNAL_ID: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "lists": [
    {
      "id": "b9b963c2-4248-4309-aef9-afbc90969cc4",
      "state": "open",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": 0,
      "progress": 0,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": null,
      "results_path": null,
      "account_external_id": 12345
    },
    {
      "id": "aad5327e-a4fa-4dac-8f24-4f622320a58a",
      "state": "complete",
      "total_verified": 2,
      "total_verified_emails": 1,
      "total_verified_phones": 1,
      "page_count": 1,
      "progress": 100,
      "created_at": "08-10-2021 04:26 pm",
      "expiration_date": "08-17-2021 04:26 pm",
      "results_path": "https://bulk-api.briteverify.com/api/v3/accounts/12345/lists/aad5327e-a4fa-4dac-8f24-4f622320a58a/export/1",
      "account_external_id": 12345
    },
    {
      "id": "0666d920-adc4-42a1-b8e8-d1a87e695cee",
      "state": "terminated",
      "total_verified": 0,
      "total_verified_emails": 0,
      "total_verified_phones": 0,
      "page_count": null,
      "progress": 0,
      "created_at": "08-10-2021 04:03 pm",
      "expiration_date": null,
      "results_path": null,
      "account_external_id": 12345,
      "errors": [
        {
          "code": "import_error",
          "message": "user terminated at 08-10-2021 04:07PM"
        }
      ]
    }
  ]
}
"#,
};

// </editor-fold desc="// Get Bulk Verification Lists ...">

// <editor-fold desc="// Get Verification List State ...">

pub const ERROR_LIST_STATE_NOT_FOUND: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "No matching list found",
  "status": "not_found"
}
"#,
};

pub const OFFICIAL_LIST_STATE_EXPIRED: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "created_at": "08-10-2021 05:08 pm",
  "errors": [
    {
      "code": "expired",
      "message": "Expired. Download Unavailable."
    }
  ],
  "expiration_date": "08-17-2021 05:08 pm",
  "id": "eda3acb3-099e-4a39-8563-3dcdde5a4411",
  "page_count": 0,
  "progress": 0,
  "results_path": null,
  "state": "complete",
  "total_verified": 10,
  "total_verified_emails": 10,
  "total_verified_phones": 16
}
"#,
};
pub const OFFICIAL_LIST_STATE_COMPLETE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "created_at": "08-10-2021 05:06 pm",
  "expiration_date": "08-17-2021 05:07 pm",
  "id": "52233c90-3dbe-47d4-910b-1fa9d1e8829c",
  "page_count": 1,
  "progress": 100,
  "results_path": "https://bulk-api.briteverify.com/api/v3/lists/52233c90-3dbe-47d4-910b-1fa9d1e8829c/export/1",
  "state": "complete",
  "total_verified": 32,
  "total_verified_emails": 16,
  "total_verified_phones": 16
}
"#,
};
pub const OFFICIAL_LIST_STATE_VERIFYING: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "created_at": "08-10-2021 05:08 pm",
  "expiration_date": null,
  "id": "d3b7e1c9-0bb3-4d93-9809-560921dc91b6",
  "page_count": 0,
  "progress": 0,
  "results_path": null,
  "state": "verifying",
  "total_verified": 0,
  "total_verified_emails": 0,
  "total_verified_phones": 16
}
"#,
};
pub const OFFICIAL_LIST_STATE_TERMINATED: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "created_at": "08-10-2021 04:03 pm",
  "errors": [
    {
      "code": "import_error",
      "message": "user terminated at 08-10-2021 04:07PM"
    }
  ],
  "expiration_date": null,
  "id": "2880123d-172d-477b-aea0-11ba417eb07f",
  "page_count": null,
  "progress": 0,
  "results_path": null,
  "state": "terminated",
  "total_verified": 0,
  "total_verified_emails": 0,
  "total_verified_phones": 0
}
"#,
};
pub const OFFICIAL_LIST_STATE_AUTO_TERMINATED: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "counts": 0,
  "created_at": "06-15-2020 12:00 pm",
  "errors": [
    {
      "code": "import_error",
      "message": "auto-terminated at 07-27-2020 01:00PM due to inactivity"
    }
  ],
  "id": "5cb2df8b-619d-4843-bf37-3d8b9565815f",
  "page_count": null,
  "progress": 0,
  "results_path": null,
  "state": "terminated"
}
"#,
};
pub const OFFICIAL_LIST_STATE_WITH_EXTERNAL_ID: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "account_external_id": "12345",
  "created_at": "08-10-2021 04:26 pm",
  "expiration_date": "08-17-2021 04:26 pm",
  "id": "c7995898-1368-4aa4-9427-236f25192b30",
  "page_count": 1,
  "progress": 100,
  "results_path": "https://bulk-api.briteverify.com/api/v3/accounts/12345/lists/c7995898-1368-4aa4-9427-236f25192b303/export/1",
  "state": "complete",
  "total_verified": 2,
  "total_verified_emails": 1,
  "total_verified_phones": 1
}
"#,
};

// </editor-fold desc="// Get Verification List State ...">

// <editor-fold desc="// Get Verification List Results ...">

pub const ERROR_PAGE_NOT_FOUND: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "Page number 2 invalid.",
  "status": "invalid_page_number"
}
"#,
};
pub const ERROR_LIST_RESULTS_NOT_FOUND: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "No matching list found.",
  "status": "not_found"
}
"#,
};

pub const OFFICIAL_GET_LIST_RESULTS_EXPIRED: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "List with id 4abb46d3-99a0-439b-9695-c54919ab01ec has expired.",
  "status": "list_expired"
}
"#,
};
pub const OFFICIAL_GET_LIST_RESULTS_LAST_PAGE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "num_pages": 2,
  "results": [
    {
      "email": {
        "email": "john.doe@email.com",
        "secondary_status": null,
        "status": "valid"
      },
      "phone": {
        "phone": "4444444444",
        "phone_location": null,
        "phone_service_type": "mobile",
        "secondary_status": null,
        "status": "valid"
      }
    },
    {
      "email": {
        "email": "jane.doe@email.com",
        "secondary_status": null,
        "status": "valid"
      },
      "phone": {
        "phone": "5555555555",
        "phone_location": null,
        "phone_service_type": "mobile",
        "secondary_status": null,
        "status": "valid"
      }
    },
    {
      "email": {
        "email": "jim.doe@email.com",
        "secondary_status": null,
        "status": "valid"
      },
      "phone": {
        "phone": "6666666666",
        "phone_location": null,
        "phone_service_type": "mobile",
        "secondary_status": null,
        "status": "valid"
      }
    }
  ],
  "status": "success"
}
"#,
};
pub const OFFICIAL_GET_LIST_RESULTS_EMAILS_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "num_pages": "2",
  "results": [
    {
      "email": "invalid@test.com",
      "secondary_status": "email_account_invalid",
      "status": "invalid"
    },
    {
      "email": "unknown@test.com",
      "secondary_status": null,
      "status": "unknown"
    },
    {
      "email": "valid@test.com",
      "secondary_status": "role_address",
      "status": "valid"
    },
    {
      "email": "accept_all@test.com",
      "secondary_status": "role_address",
      "status": "accept_all"
    }
  ],
  "status": "success"
}
"#,
};
pub const OFFICIAL_GET_LIST_RESULTS_CONTACTS_PARAMETER: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "num_pages": 1,
  "results": [
    {
      "address": {
        "address1": "200 Clarendon St Ste 2200",
        "address2": null,
        "city": "Boston",
        "corrected": "true",
        "secondary_status": null,
        "state": "MA",
        "status": "valid",
        "zip": "02116-5051"
      },
      "email": {
        "email": "sales@validity.com",
        "secondary_status": "role_address",
        "status": "valid"
      },
      "phone": {
        "phone": "18009618205",
        "phone_location": null,
        "phone_service_type": "land",
        "secondary_status": null,
        "status": "valid"
      }
    },
    {
      "email": {
        "email": "goodbye@example.com",
        "secondary_status": "email_domain_invalid",
        "status": "invalid"
      },
      "phone": {
        "phone": "5555555555",
        "phone_location": null,
        "phone_service_type": null,
        "secondary_status": "invalid_phone_number",
        "status": "invalid"
      }
    }
  ],
  "status": "success"
}
"#,
};

// </editor-fold desc="// Get Verification List Results ...">

// <editor-fold desc="// Delete Bulk Verification List ...">

pub const ERROR_INVALID_LIST_STATE: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "message": "The provided list is in an invalid state -- state is verifying and needs to be one of the following to continue: complete, delivered, prepped, import_error.",
  "status": "invalid_state"
}
"#,
};

pub const OFFICIAL_DELETE_PREPPED_LIST: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "list": {
    "created_at": "07-18-2022 12:42 pm",
    "expiration_date": null,
    "id": "ec137d51-cbad-4924-9fcb-57d7566b031d",
    "page_count": 1,
    "progress": 100,
    "results_path": null,
    "state": "deleted",
    "total_verified": 32,
    "total_verified_emails": 16,
    "total_verified_phones": 16
  },
  "status": "success"
}"#,
};
pub const OFFICIAL_DELETE_COMPLETED_LIST: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "list": {
    "created_at": "07-18-2022 12:42 pm",
    "expiration_date": null,
    "id": "13ae1f20-9483-4e0e-857d-58d83f371859",
    "page_count": 1,
    "progress": 100,
    "results_path": null,
    "state": "deleted",
    "total_verified": 32,
    "total_verified_emails": 16,
    "total_verified_phones": 16
  },
  "status": "success"
}"#,
};
pub const OFFICIAL_DELETE_DELIVERED_LIST: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "list": {
    "created_at": "07-18-2022 12:42 pm",
    "expiration_date": null,
    "id": "6fcd86e6-e197-4b3f-a6d6-f531f1990206",
    "page_count": 1,
    "progress": 100,
    "results_path": null,
    "state": "deleted",
    "total_verified": 32,
    "total_verified_emails": 16,
    "total_verified_phones": 16
  },
  "status": "success"
}"#,
};
pub const OFFICIAL_DELETE_IMPORT_ERRORED_LIST: MockRequestResponse = MockRequestResponse {
    request: r"",
    response: r#"{
  "list": {
    "created_at": "07-18-2022 12:42 pm",
    "expiration_date": null,
    "id": "9984e0f5-420c-4d5f-b8ff-867d96192d8e",
    "page_count": 1,
    "progress": 100,
    "results_path": null,
    "state": "deleted",
    "total_verified": 32,
    "total_verified_emails": 16,
    "total_verified_phones": 16
  },
  "status": "success"
}"#,
};

// </editor-fold desc="// Delete Bulk Verification List ...">
