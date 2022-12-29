use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    test, Error,
};
use base64::encode;
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::{env, process::Command, str};
use zeronote::database::connection::{init_pool, run_migrations, Pool};

pub struct Context {
    pub db_name: String,
    pub psql_user: String,
    pub psql_pw: String,
}

impl Context {
    pub fn new(db_name: &str) -> Self {
        dotenv().ok();
        let psql_user =
            env::var("POSTGRES_USER").expect("POSTGRES_USER must be set for integration tests");
        let psql_pw = env::var("POSTGRES_PASSWORD")
            .expect("POSTGRES_PASSWORD must be set for integration tests");
        let database_url = format!(
            "postgres://{}:{}@localhost:5432/postgres",
            psql_user, psql_pw
        );
        let mut conn = PgConnection::establish(&database_url)
            .expect("Failed to connect to the database 'postgres'");

        let query = sql_query(format!("CREATE DATABASE {};", db_name));
        query
            .execute(&mut conn)
            .expect(format!("Couldn't create database {}", db_name).as_str());

        Self {
            db_name: db_name.to_string(),
            psql_user,
            psql_pw,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let database_url = format!(
            "postgres://{}:{}@localhost:5432/postgres",
            self.psql_user, self.psql_pw
        );
        let mut conn = PgConnection::establish(&database_url)
            .expect("Failed to connect to the database 'postgres'");

        let query = sql_query(format!("DROP DATABASE {};", self.db_name));
        query
            .execute(&mut conn)
            .expect(format!("Couldn't drop database {}", self.db_name).as_str());
    }
}

pub fn create_pool(ctx: &Context) -> Pool {
    let database_url = format!(
        "postgres://{}:{}@localhost:5432/{}",
        ctx.psql_user, ctx.psql_pw, ctx.db_name
    );
    // Pool is unnecessary for tests, but easier than completely changing Actix's web::Data type
    let pool = init_pool(database_url);
    let mut conn = pool.get().unwrap();

    let query = sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";");
    query
        .execute(&mut conn)
        .expect("Couldn't install postgres extension 'uuid-ossp'");
    run_migrations(&mut conn);

    pool
}

pub fn create_signature(username: &str, client_id: &str, client_secret: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(client_secret.as_bytes())
        .expect("Failed creating HMAC for AWS authentication");
    mac.update((username.to_owned() + client_id).as_bytes());
    let res_vec = mac.finalize().into_bytes().to_vec();

    encode(&res_vec)
}

pub fn fetch_jwt() -> String {
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set for integration tests");
    let client_secret =
        env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set for integration tests");
    let keyset_pool_id =
        env::var("KEYSET_POOL_ID").expect("KEYSET_POOL_ID must be set for integration tests");
    let username =
        env::var("OAUTH_USERNAME").expect("OAUTH_USERNAME must be set for integration tests");
    let password =
        env::var("OAUTH_PASSWORD").expect("OAUTH_PASSWORD must be set for integration tests");

    // Base64(HMAC_SHA256("Client Secret Key", "Username" + "Client Id"))
    let secret_hash = create_signature(&username, &client_id, &client_secret);

    // Requires AWS CLI
    let aws_command = format!(
        "aws cognito-idp admin-initiate-auth --user-pool-id '{}' --client-id '{}' --auth-flow 'ADMIN_USER_PASSWORD_AUTH' --auth-parameters USERNAME='{}',PASSWORD='{}',SECRET_HASH='{}' --output 'json'",
        keyset_pool_id, client_id, username, password, secret_hash
    );
    let raw_stdout = Command::new("sh")
        .arg("-c")
        .arg(aws_command)
        .output()
        .unwrap()
        .stdout;
    let res_data: Value = serde_json::from_str(str::from_utf8(&raw_stdout).unwrap()).unwrap();
    let access_token = res_data["AuthenticationResult"]["AccessToken"].clone();

    "Bearer ".to_owned() + access_token.as_str().unwrap()
}

pub async fn get_endpoint_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    bearer: &str,
    uri: &str,
) -> ServiceResponse {
    let req = test::TestRequest::get()
        .uri(uri)
        .insert_header(("Authorization", bearer))
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

pub async fn post_endpoint_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    req_body: Value,
    bearer: &str,
    uri: &str,
) -> ServiceResponse {
    let req = test::TestRequest::post()
        .uri(uri)
        .insert_header(("Authorization", bearer))
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

pub async fn put_endpoint_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    req_body: Value,
    bearer: &str,
    uri: &str,
) -> ServiceResponse {
    let req = test::TestRequest::put()
        .uri(uri)
        .insert_header(("Authorization", bearer))
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}

pub async fn delete_endpoint_res(
    app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    req_body: Value,
    bearer: &str,
    uri: &str,
) -> ServiceResponse {
    let req = test::TestRequest::delete()
        .uri(uri)
        .insert_header(("Authorization", bearer))
        .set_json(req_body)
        .to_request();
    let res = test::call_service(&app, req).await;

    res
}
