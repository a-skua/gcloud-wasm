wit_bindgen::generate!({
    world: "app",
    path: "wit",
    generate_all,
});

use gcloud::storage::buckets::list_buckets;

fn main() {
    let project = std::env::var("GOOGLE_CLOUD_PROJECT").expect("GOOGLE_CLOUD_PROJECT is not set");

    match list_buckets(&project) {
        Ok(buckets) => {
            println!("Buckets in project '{project}':");
            for bucket in &buckets {
                println!("  - {}", bucket.name);
            }
            if buckets.is_empty() {
                println!("  (none)");
            }
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
        }
    }
}
