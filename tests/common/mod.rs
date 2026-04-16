#![allow(dead_code)]

use std::sync::{Arc, OnceLock, Weak};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, header},
};
use reqwest::{Client, Url};
use rstest::fixture;
use rust_hack_template::{build_router, build_state, config::Config, connect_migrated_pgpool, run};
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use uuid::Uuid;

static POSTGRES: OnceLock<Mutex<Weak<TestPostgres>>> = OnceLock::new();
const POSTGRES_IMAGE_TAG: &str = "18-alpine";

struct TestPostgres {
    _container: ContainerAsync<Postgres>,
    host: String,
    port: u16,
}

fn postgres_slot() -> &'static Mutex<Weak<TestPostgres>> {
    POSTGRES.get_or_init(|| Mutex::new(Weak::new()))
}

async fn shared_postgres() -> Arc<TestPostgres> {
    let mut guard = postgres_slot().lock().await;

    if let Some(postgres) = guard.upgrade() {
        return postgres;
    }

    let container = Postgres::default()
        .with_tag(POSTGRES_IMAGE_TAG)
        .start()
        .await
        .expect("start postgres test container");
    let host = container
        .get_host()
        .await
        .expect("postgres container host")
        .to_string();
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres container port");

    let postgres = Arc::new(TestPostgres {
        _container: container,
        host,
        port,
    });

    *guard = Arc::downgrade(&postgres);
    postgres
}

fn build_database_url(host: &str, port: u16, database: &str) -> String {
    format!("postgres://postgres:postgres@{host}:{port}/{database}")
}

async fn create_test_database(postgres: &TestPostgres) -> anyhow::Result<String> {
    let admin_url = build_database_url(&postgres.host, postgres.port, "postgres");
    let mut admin = PgConnection::connect(&admin_url).await?;

    let database_name = format!("test_{}", Uuid::new_v4().simple());
    admin
        .execute(format!(r#"CREATE DATABASE "{database_name}""#).as_str())
        .await?;

    Ok(build_database_url(
        &postgres.host,
        postgres.port,
        &database_name,
    ))
}

pub fn test_config(database_url: String) -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_url,
        session_cookie_name: "session".to_string(),
        session_secure_cookie: false,
        session_ttl_days: 7,
    }
}

pub struct TestApp {
    _postgres: Arc<TestPostgres>,
    pub database_url: String,
    pub config: Config,
    pub pool: PgPool,
}

impl TestApp {
    pub async fn new() -> anyhow::Result<Self> {
        let postgres = shared_postgres().await;
        let database_url = create_test_database(postgres.as_ref()).await?;
        let config = test_config(database_url.clone());
        let pool = connect_migrated_pgpool(&database_url).await?;

        Ok(Self {
            _postgres: postgres,
            database_url,
            config,
            pool,
        })
    }

    pub fn router(&self) -> Router {
        let state = build_state(self.config.clone(), self.pool.clone());
        build_router(state)
    }

    pub async fn spawn_server(self) -> anyhow::Result<TestServer> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let app = self.router();
        let handle = tokio::spawn(async move { run(listener, app).await });

        Ok(TestServer {
            _test_app: self,
            base_url: Url::parse(&format!("http://{address}"))?,
            handle,
        })
    }
}

pub struct TestServer {
    _test_app: TestApp,
    pub base_url: Url,
    handle: JoinHandle<anyhow::Result<()>>,
}

impl TestServer {
    pub fn endpoint(&self, path: &str) -> Url {
        self.base_url.join(path).expect("join server url")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[fixture]
pub async fn test_app() -> TestApp {
    TestApp::new().await.expect("test app")
}

#[fixture]
pub async fn test_server(#[future] test_app: TestApp) -> TestServer {
    test_app.await.spawn_server().await.expect("spawn server")
}

#[fixture]
pub fn http_client() -> Client {
    Client::builder()
        .cookie_store(true)
        .build()
        .expect("http client")
}

#[fixture]
pub fn email() -> String {
    unique_email()
}

#[fixture]
pub fn password() -> &'static str {
    "password123"
}

pub fn unique_email() -> String {
    format!("user-{}@example.com", Uuid::new_v4().simple())
}

pub fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("build json request")
}

pub fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("build empty request")
}

pub fn cookie_request(method: Method, uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .expect("build cookie request")
}

pub async fn response_json(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");

    if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).expect("parse response json")
    }
}

pub fn set_cookie_header(response: &Response<Body>) -> String {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("set-cookie header")
        .to_str()
        .expect("set-cookie as str")
        .to_string()
}

pub fn cookie_pair(response: &Response<Body>) -> String {
    set_cookie_header(response)
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string()
}
