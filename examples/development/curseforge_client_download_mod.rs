use minepack::api::curseforge::CurseforgeClient;

#[tokio::main]
async fn main() {
    let client = CurseforgeClient::new().unwrap();
    let data = client.download_mod_file(1030830, 6332315).await.unwrap();
    assert!(!data.is_empty());
    println!("Downloaded mod file successfully");
}
