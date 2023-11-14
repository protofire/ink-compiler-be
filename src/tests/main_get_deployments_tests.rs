#[cfg(test)]
mod get_deployments_test {
    use super::super::*;
    use crate::models::api_models::ServerResponse;
    use crate::MongoRepo;
    use mongodb::bson::doc;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn get_deployments_no_matching_routes_error() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/deployments")).dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert!(response
            .into_string()
            .unwrap()
            .contains("The requested resource could not be found."));
        client.terminate();
    }

    #[test]
    fn get_deployments_user_address_missing_error() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let url = "/deployments?network=".to_string();
        let response = client.get(url).dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert!(response
            .into_string()
            .unwrap()
            .contains("The requested resource could not be found."));
        client.terminate();
    }

    #[test]
    fn get_deployments_network_missing_and_empty_user_address_empty_result() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let url = "/deployments?user_address=".to_string();
        let response = client.get(url).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .unwrap()
            .contains("{\"data\":[],\"error\":null}"));
        client.terminate();
    }

    #[test]
    fn get_deployments_no_data() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let url = "/deployments?user_address=&network=".to_string();
        let response = client.get(url).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .unwrap()
            .contains("{\"data\":[],\"error\":null}"));
        client.terminate();
    }

    // Since the database is not mocked, this test will fail if the user has something already deployed
    #[test]
    fn get_deployments_no_user_address_leads_to_empty_data_no_error() {
        let unused = "5FrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let url = format!(
            "/deployments?user_address={}&network={}",
            unused, "some_network"
        );
        let response = client.get(url).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.status(), Status::Ok);
        assert!(response
            .into_string()
            .unwrap()
            .contains("{\"data\":[],\"error\":null}"));
        client.terminate();
    }

    #[test]
    fn get_deployments_matching_routes_error() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let db = client.rocket().state::<MongoRepo>().unwrap();
        let post_response = client.post(uri!("/deployments")).body(r#"{ "contract_address": "5CfkL1QXpnMoto87UYNER6B9dktRADjn1Vyrvzvc4ZziraFs", "network": "some_network", "code_id": "some_impossible_id", "user_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "date":"2021-03-03T15:00:00.000Z", "contract_type":"custom" }"#).dispatch();
        let json: ServerResponse<String> = post_response.into_json().unwrap();
        let deployment_id = json.data.unwrap();

        let url = format!(
            "/deployments?user_address={}&network={}",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "some_network"
        );
        let response = client.get(url).dispatch();

        assert_eq!(response.status(), Status::Ok);
        let original_json = r#"{"data":[{"_id":{"$oid":"OID_PLACEHOLDER"},"contract_name":null,"contract_address":"5CfkL1QXpnMoto87UYNER6B9dktRADjn1Vyrvzvc4ZziraFs","network":"some_network","code_id":"some_impossible_id","user_address":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY","tx_hash":null,"date":"2021-03-03T15:00:00.000Z","contract_type":"custom","external_abi":null,"hidden":false}],"error":null}"#;
        let final_json = original_json.replace("OID_PLACEHOLDER", &deployment_id);
        assert_eq!(response.into_string().unwrap(), final_json);
        let db_res = db.deployments.delete_one(
            doc! {"contract_address": "5CfkL1QXpnMoto87UYNER6B9dktRADjn1Vyrvzvc4ZziraFs","network": "some_network", "user_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"},
            None,
        );
        assert!(db_res.is_ok());
        client.terminate();
    }

    #[test]
    fn get_deployment_by_id() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let db = client.rocket().state::<MongoRepo>().unwrap();

        let post_response = client.post(uri!("/deployments")).body(r#"{ "contract_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "network": "some_network", "code_id": "some_id", "user_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "date":"2021-03-03T15:00:00.000Z", "contract_type":"custom" }"#).dispatch();
        // status ok means that the deployment was stored in the database
        assert_eq!(post_response.status(), Status::Ok);

        // The post-response has the following structure: Some("{\"data\":\"6529673f5bd94db34547d990\",\"error\":null}") We need to obtain just the data which is the id:
        let json: ServerResponse<String> = post_response.into_json().unwrap();
        let deployment_id = json.data.unwrap();

        let url = format!("/deployment?id={}", deployment_id);
        let get_response = client.get(url).dispatch();

        assert_eq!(get_response.status(), Status::Ok);
        std::mem::drop(get_response);

        // Cleanup
        let db_res = db.deployments.delete_one(
            doc! {"contract_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY","network": "some_network", "user_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"},
            None,
        );
        assert!(db_res.is_ok());
        client.terminate();
    }
}
