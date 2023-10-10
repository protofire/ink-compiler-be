use crate::models::api_models::{ServerResponse, WizardMessage};
use crate::models::db_models::Contract;
use log::error;
use rocket::{http::Status, response::status::Custom, serde::json::Json};
use sp_core::crypto::{AccountId32, Ss58Codec};

pub const CONTRACTS: [&str; 3] = ["psp22", "psp34", "psp37"];

pub const ALLOWED_FEATURES: [&str; 6] = [
    "psp22",
    "psp34",
    "psp37",
    "pausable",
    "ownable",
    "access-control",
];

pub const MAX_SIZE_ALLOWED: usize = 49999;

pub fn sanity_check_wizard_message(
    wizard_message: &Json<WizardMessage>,
) -> Result<(), Custom<Json<ServerResponse<Contract>>>> {
    // Checks length of the code not passing the max allowed
    match check_code_len(&wizard_message.code) {
        Ok(_) => (),
        Err(msg) => {
            return Err(Custom(
                Status::InternalServerError,
                Json(ServerResponse::new_error(String::from(msg))),
            ))
        }
    }

    // Checks the address len is valid
    match check_address(&wizard_message.address) {
        Ok(_) => (),
        Err(msg) => {
            return Err(Custom(
                Status::InternalServerError,
                Json(ServerResponse::new_error(String::from(msg))),
            ))
        }
    }

    check_features(&wizard_message.features)?;

    Ok(())
}

pub fn check_code_len(code: &String) -> Result<(), String> {
    if code.len() > MAX_SIZE_ALLOWED {
        error!(target: "compiler", "Code size is too big");
        return Err("Code size too big.".to_string());
    }
    Ok(())
}

pub fn check_address(address: &String) -> Result<(), String> {
    let ss58_check = AccountId32::from_ss58check(address);
    if ss58_check.is_err() {
        let err_msg = format!("Address is not valid: {:?}", ss58_check.unwrap_err());
        error!(target: "compiler", "{}", err_msg);
        return Err(err_msg);
    }
    Ok(())
}

pub fn check_features(
    features: &Vec<String>,
) -> Result<(), Custom<Json<ServerResponse<Contract>>>> {
    // Checks features not to be empty
    if features.is_empty() {
        error!(target: "compiler", "Features are empty");
        return Err(Custom(
            Status::InternalServerError,
            Json(ServerResponse::new_error(String::from(
                "Features must not be empty.",
            ))),
        ));
    }

    // Checks all the features passed are allowed
    for feature in features {
        if !ALLOWED_FEATURES.contains(&feature.as_str()) {
            error!(target: "compiler", "Feature not allowed: {:?}", feature);
            return Err(Custom(
                Status::InternalServerError,
                Json(ServerResponse::new_error(String::from(
                    "Feature not allowed",
                ))),
            ));
        }
    }

    // sets the found flag
    let mut found = false;

    // found flag is used to check the contract has a single and allowed standard
    for feature in features {
        if CONTRACTS.contains(&feature.as_str()) {
            if !found {
                found = true;
            } else {
                error!(target: "compiler", "Feature contains ambiguous contract standard");
                return Err(Custom(
                    Status::InternalServerError,
                    Json(ServerResponse::new_error(String::from(
                        "Feature contains ambiguous contract standard",
                    ))),
                ));
            }
        }
    }
    // here it checks at least one standard was found
    if !found {
        error!(target: "compiler", "Features must contain at least one contract standard");
        return Err(Custom(
            Status::InternalServerError,
            Json(ServerResponse::new_error(String::from(
                "Features must contain at least one contract standard",
            ))),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/utils/sanity_check_tests.rs"]
mod sanity_check_tests;
