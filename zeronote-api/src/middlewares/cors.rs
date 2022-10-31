use actix_cors::Cors;
use actix_web::http::{header, Method};

// Restricts data exchange to strictly CLIENT_ORIGIN_URL

pub fn cors(client_origin_url: &str) -> Cors {
    Cors::default()
        .allowed_origin(client_origin_url)
        .allowed_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allowed_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(86400)
}
