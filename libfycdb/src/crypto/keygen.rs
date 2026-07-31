use tracing::instrument;

#[instrument]
pub fn generate_keypair() -> (String, String) {
    let response = librage::generate_keypair();
    if response.success {
        let data = response.data.unwrap();
        (data.secret_key.to_string(), data.public_key)
    } else {
        panic!("Key generation failed unexpectedly");
    }
}
