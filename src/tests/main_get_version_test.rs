#[cfg(test)]
mod get_contract_test {
    use super::super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn get_version_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/version")).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.into_string().unwrap().contains("v1.0.0"));
        client.terminate();
    }
}
