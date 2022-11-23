use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

pub fn create_builder() -> Result<SslAcceptorBuilder, Box<dyn std::error::Error>> {
    // dev: openssl req -newkey rsa:2048 -nodes -keyout key.pem -x509 -days 365 -out cert.pem -addext "subjectAltName = DNS:localhost"
    // dev: manually configure browser to trust the created cert
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert.pem")?;

    Ok(builder)
}
