use minepack::api::curseforge::CurseforgeClient;

#[tokio::main]
async fn main() {
    let client = CurseforgeClient::new().unwrap();
    let query = "jei";
    let mods = client.search_mods(query, None).await.unwrap();

    for m in mods {
        println!("Found mod: {}", m.name);
    }
}
