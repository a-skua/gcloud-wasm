wit_bindgen::generate!({
    world: "app",
    path: "wit",
    generate_all,
});

use gcloud::secretmanager::secrets::access;

fn main() {
    let name =
        std::env::var("SECRET_NAME").expect("SECRET_NAME is not set (e.g. projects/PROJECT/secrets/SECRET/versions/latest)");

    match access(&name) {
        Ok(payload) => match String::from_utf8(payload.data) {
            Ok(s) => println!("Secret: {s}"),
            Err(e) => println!("Secret: (binary data, {} bytes)", e.into_bytes().len()),
        },
        Err(e) => {
            eprintln!("Error: {e:?}");
        }
    }
}
