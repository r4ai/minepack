use minepack::api::curseforge::schema::SearchModsRequestQuery;
use minepack::api::curseforge::CurseforgeClient;

#[tokio::main]
async fn main() {
    let client = CurseforgeClient::new().unwrap();
    let mods = client
        .search_mods(&SearchModsRequestQuery {
            search_filter: Some("oritech".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    for mod_info in mods {
        println!("Found mod: {} (ID: {})", mod_info.name, mod_info.id);
    }
}
