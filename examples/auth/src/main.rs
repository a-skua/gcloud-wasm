wit_bindgen::generate!({
    world: "app",
    path: "wit",
    generate_all,
});

use gcloud::auth::token_source::get_token;

fn main() {
    let scopes = vec!["https://www.googleapis.com/auth/cloud-platform".to_string()];

    match get_token(&scopes) {
        Ok(token) => {
            let masked = if token.access_token.len() > 12 {
                format!("{}...{}", &token.access_token[..6], &token.access_token[token.access_token.len()-6..])
            } else {
                "***".to_string()
            };
            println!("Access Token: {masked}");
            println!("Token Type: {}", token.token_type);
            println!("Expires In: {}s", token.expires_in);
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
        }
    }
}
