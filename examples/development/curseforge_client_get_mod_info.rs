use minepack::api::curseforge::CurseforgeClient;

#[tokio::main]
async fn main() {
    let client = CurseforgeClient::new().unwrap();
    let data = client.get_mod_info(238222).await.unwrap();
    dbg!(data);
}
