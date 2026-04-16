mod common;

use reqwest::{Client, StatusCode};
use rstest::rstest;
use serde_json::json;

use crate::common::{TestServer, email, http_client, password, test_server};

#[rstest]
#[tokio::test]
async fn auth_happy_path_over_http(
    #[future] test_server: TestServer,
    http_client: Client,
    email: String,
    password: &'static str,
) {
    let server = test_server.await;

    let register_response = http_client
        .post(server.endpoint("/auth/register"))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("register response");
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let login_response = http_client
        .post(server.endpoint("/auth/login"))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("login response");
    assert_eq!(login_response.status(), StatusCode::OK);

    let me_response = http_client
        .get(server.endpoint("/auth/me"))
        .send()
        .await
        .expect("me response");
    assert_eq!(me_response.status(), StatusCode::OK);
    let me_body = me_response
        .json::<serde_json::Value>()
        .await
        .expect("me json");
    assert_eq!(me_body["email"], json!(email));

    let logout_response = http_client
        .post(server.endpoint("/auth/logout"))
        .send()
        .await
        .expect("logout response");
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

    let me_after_logout = http_client
        .get(server.endpoint("/auth/me"))
        .send()
        .await
        .expect("me after logout response");
    assert_eq!(me_after_logout.status(), StatusCode::UNAUTHORIZED);
}
