wit_bindgen::generate!({
    world: "token-provider",
    path: "wit",
});

mod adc;
mod authorized_user;
mod cache;
mod metadata;
mod service_account;

use adc::Adc;
use exports::gcloud::auth::token_source::{Error, Guest, Token};

struct Component;

export!(Component);

impl Guest for Component {
    fn get_token(scopes: Vec<String>) -> Result<Token, Error> {
        wstd::runtime::block_on(get_access_token(scopes))
    }
}

async fn get_access_token(scopes: Vec<String>) -> Result<Token, Error> {
    // 1. Check cache
    if let Some(token) = cache::get(&scopes) {
        return Ok(token);
    }

    // 2. Try ADC file, fallback to metadata server
    let token = match adc::read_adc() {
        Ok(Adc::AuthorizedUser {
            client_id,
            client_secret,
            refresh_token,
        }) => authorized_user::fetch_token(&client_id, &client_secret, &refresh_token).await?,
        Ok(Adc::ServiceAccount {
            client_email,
            private_key,
            private_key_id,
            token_uri,
        }) => {
            service_account::fetch_token(
                &client_email,
                &private_key,
                &private_key_id,
                &token_uri,
                &scopes,
            )
            .await?
        }
        Err(_) => metadata::fetch_token(&scopes).await?,
    };

    // 3. Cache the token
    cache::put(&scopes, &token);

    Ok(token)
}
