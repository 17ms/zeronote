use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

pub fn create_builder() -> Result<SslAcceptorBuilder, Box<dyn std::error::Error>> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert.pem")?;

    Ok(builder)
}
