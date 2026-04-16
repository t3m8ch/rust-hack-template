mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Response, StatusCode},
};
use rstest::rstest;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{
    TestApp, cookie_pair, cookie_request, email, empty_request, json_request, password,
    response_json, set_cookie_header, test_app,
};

async fn register(app: &Router, email: &str, password: &str) -> Response<Body> {
    app.clone()
        .oneshot(json_request(
            Method::POST,
            "/auth/register",
            json!({
                "email": email,
                "password": password,
            }),
        ))
        .await
        .expect("register response")
}

async fn login(app: &Router, email: &str, password: &str) -> Response<Body> {
    app.clone()
        .oneshot(json_request(
            Method::POST,
            "/auth/login",
            json!({
                "email": email,
                "password": password,
            }),
        ))
        .await
        .expect("login response")
}

#[rstest]
#[tokio::test]
async fn register_success(#[future] test_app: TestApp, email: String, password: &'static str) {
    let test_app = test_app.await;
    let app = test_app.router();
    let raw_email = email.to_uppercase();

    let response = register(&app, &raw_email, password).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    assert_eq!(body["email"], json!(email));
    assert!(body.get("id").is_some());
}

#[rstest]
#[tokio::test]
async fn register_duplicate_email_returns_conflict(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let first = register(&app, &email, password).await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = register(&app, &email, password).await;

    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    let body = response_json(duplicate).await;
    assert_eq!(body["error"]["code"], json!("email_already_exists"));
}

#[rstest]
#[tokio::test]
async fn login_success_sets_session_cookie(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let register_response = register(&app, &email, password).await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let response = login(&app, &email, password).await;

    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = set_cookie_header(&response);
    assert!(set_cookie.contains("session="));
    let body = response_json(response).await;
    assert_eq!(body["email"], json!(email));
}

#[rstest]
#[tokio::test]
async fn login_wrong_password_returns_unauthorized(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let register_response = register(&app, &email, password).await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let response = login(&app, &email, "nottherightpass").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], json!("invalid_credentials"));
}

#[rstest]
#[tokio::test]
async fn me_without_cookie_returns_unauthorized(#[future] test_app: TestApp) {
    let test_app = test_app.await;
    let app = test_app.router();

    let response = app
        .clone()
        .oneshot(empty_request(Method::GET, "/auth/me"))
        .await
        .expect("me response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], json!("unauthorized"));
}

#[rstest]
#[tokio::test]
async fn me_with_valid_cookie_returns_current_user(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let register_response = register(&app, &email, password).await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let login_response = login(&app, &email, password).await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = cookie_pair(&login_response);

    let me_response = app
        .clone()
        .oneshot(cookie_request(Method::GET, "/auth/me", &cookie))
        .await
        .expect("me response");

    assert_eq!(me_response.status(), StatusCode::OK);
    let body = response_json(me_response).await;
    assert_eq!(body["email"], json!(email));
}

#[rstest]
#[tokio::test]
async fn logout_deletes_session_and_clears_cookie(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let register_response = register(&app, &email, password).await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let login_response = login(&app, &email, password).await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = cookie_pair(&login_response);

    let logout_response = app
        .clone()
        .oneshot(cookie_request(Method::POST, "/auth/logout", &cookie))
        .await
        .expect("logout response");

    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);
    let set_cookie = set_cookie_header(&logout_response);
    assert!(set_cookie.contains("Max-Age=0"));
}

#[rstest]
#[tokio::test]
async fn me_after_logout_returns_unauthorized(
    #[future] test_app: TestApp,
    email: String,
    password: &'static str,
) {
    let test_app = test_app.await;
    let app = test_app.router();

    let register_response = register(&app, &email, password).await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    let login_response = login(&app, &email, password).await;
    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = cookie_pair(&login_response);

    let logout_response = app
        .clone()
        .oneshot(cookie_request(Method::POST, "/auth/logout", &cookie))
        .await
        .expect("logout response");
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

    let me_response = app
        .clone()
        .oneshot(cookie_request(Method::GET, "/auth/me", &cookie))
        .await
        .expect("me response after logout");

    assert_eq!(me_response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(me_response).await;
    assert_eq!(body["error"]["code"], json!("unauthorized"));
}
