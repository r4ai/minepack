use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Response from GET /v1/mods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModsResponse {
    pub data: Vec<Mod>,
}

/// Parameters for GET /v1/mods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModsByIdsListRequestBody {
    #[serde(rename = "modIds")]
    pub mod_ids: Vec<u32>,

    #[serde(rename = "filterPcOnly")]
    pub filter_pc_only: bool,
}

/// Response from GET /v1/mods/{modId}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModResponse {
    pub data: Mod,
}

/// Parameters for GET /v1/mods/search
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchModsRequestQuery {
    #[serde(rename = "gameId")]
    pub game_id: Option<u32>,
    #[serde(rename = "classId")]
    pub class_id: Option<u32>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<u32>,
    #[serde(rename = "categoryIds")]
    pub category_ids: Option<Vec<u32>>,
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
    #[serde(rename = "searchFilter")]
    pub search_filter: Option<String>,
    #[serde(rename = "searchFilterType")]
    pub sort_field: Option<ModsSearchSortField>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<SortOrder>,
    #[serde(rename = "modLoaderType")]
    pub mod_loader_type: Option<ModLoaderType>,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<u32>,
    #[serde(rename = "authorId")]
    pub author_id: Option<u32>,
    #[serde(rename = "primaryAuthorId")]
    pub primary_author_id: Option<u32>,
    pub slug: Option<String>,
    /// A zero based index of the first item to include in the response, the limit is: (index + pageSize <= 10,000).
    pub index: Option<u32>,
    /// The number of items to include in the response, the default/maximum value is 50.
    pub page_size: Option<u32>,
}

/// Sort order for the search results
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModsSearchSortField {
    Featured = 1,
    Popularity = 2,
    LastUpdated = 3,
    Name = 4,
    Author = 5,
    TotalDownloads = 6,
    Category = 7,
    GameVersion = 8,
    EarlyAccess = 9,
    FeaturedReleased = 10,
    ReleasedDate = 11,
    Rating = 12,
}

impl From<u32> for ModsSearchSortField {
    fn from(val: u32) -> Self {
        match val {
            1 => ModsSearchSortField::Featured,
            2 => ModsSearchSortField::Popularity,
            3 => ModsSearchSortField::LastUpdated,
            4 => ModsSearchSortField::Name,
            5 => ModsSearchSortField::Author,
            6 => ModsSearchSortField::TotalDownloads,
            7 => ModsSearchSortField::Category,
            8 => ModsSearchSortField::GameVersion,
            9 => ModsSearchSortField::EarlyAccess,
            10 => ModsSearchSortField::FeaturedReleased,
            11 => ModsSearchSortField::ReleasedDate,
            12 => ModsSearchSortField::Rating,
            _ => ModsSearchSortField::Featured, // Default case
        }
    }
}

impl From<ModsSearchSortField> for u32 {
    fn from(order: ModsSearchSortField) -> Self {
        order as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

impl FromStr for SortOrder {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(SortOrder::Ascending),
            "desc" => Ok(SortOrder::Descending),
            _ => Err(()),
        }
    }
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::Ascending => "asc".to_string(),
            SortOrder::Descending => "desc".to_string(),
        }
    }
}

/// Response from GET /v1/mods/search
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchModsResponse {
    pub data: Vec<Mod>,
    pub pagination: Pagination,
}

/// Response from GET /v1/mods/{modId}/files/{fileId}/download-url
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetDownloadUrlResponse {
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub links: ModLinks,
    pub summary: String,
    pub status: ModStatus,
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "isFeatured")]
    pub is_featured: bool,
    #[serde(rename = "primaryCategoryId")]
    pub primary_category_id: u32,
    pub categories: Vec<Category>,
    #[serde(rename = "classId")]
    pub class_id: Option<u32>,
    pub authors: Vec<ModAuthor>,
    pub logo: Option<ModAsset>,
    pub screenshots: Vec<ModAsset>,
    #[serde(rename = "mainFileId")]
    pub main_file_id: u32,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<File>,
    #[serde(rename = "latestFilesIndexes")]
    pub latest_files_indexes: Vec<FileIndex>,
    #[serde(rename = "dateCreated")]
    pub date_created: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "dateReleased")]
    pub date_released: String,
    #[serde(rename = "allowModDistribution")]
    pub allow_mod_distribution: Option<bool>,
    #[serde(rename = "gamePopularityRank")]
    pub game_popularity_rank: u32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLinks {
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "u32", into = "u32")]
pub enum ModStatus {
    New = 1,
    ChangesRequired = 2,
    UnderSoftReview = 3,
    Approved = 4,
    Rejected = 5,
    ChangesMade = 6,
    Inactive = 7,
    Abandoned = 8,
    Deleted = 9,
    UnderReview = 10,
}

impl From<u32> for ModStatus {
    fn from(val: u32) -> Self {
        match val {
            1 => ModStatus::New,
            2 => ModStatus::ChangesRequired,
            3 => ModStatus::UnderSoftReview,
            4 => ModStatus::Approved,
            5 => ModStatus::Rejected,
            6 => ModStatus::ChangesMade,
            7 => ModStatus::Inactive,
            8 => ModStatus::Abandoned,
            9 => ModStatus::Deleted,
            10 => ModStatus::UnderReview,
            _ => ModStatus::New, // Default case
        }
    }
}

impl From<ModStatus> for u32 {
    fn from(status: ModStatus) -> Self {
        status as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "isClass")]
    pub is_class: Option<bool>,
    #[serde(rename = "classId")]
    pub class_id: Option<u32>,
    #[serde(rename = "parentCategoryId")]
    pub parent_category_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAsset {
    pub id: u32,
    #[serde(rename = "modId")]
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileIndex {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "fileId")]
    pub file_id: u32,
    pub filename: String,
    #[serde(rename = "releaseType")]
    pub release_type: FileReleaseType,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<u32>,
    #[serde(rename = "modLoader")]
    pub mod_loader: Option<ModLoaderType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "u32", into = "u32")]
pub enum ModLoaderType {
    Forge = 1,
    Cauldron = 2,
    LiteLoader = 3,
    Fabric = 4,
    Quilt = 5,
    NeoForge = 6,
}

impl From<u32> for ModLoaderType {
    fn from(val: u32) -> Self {
        match val {
            1 => ModLoaderType::Forge,
            2 => ModLoaderType::Cauldron,
            3 => ModLoaderType::LiteLoader,
            4 => ModLoaderType::Fabric,
            5 => ModLoaderType::Quilt,
            6 => ModLoaderType::NeoForge,
            _ => ModLoaderType::Forge, // Default case
        }
    }
}

impl From<ModLoaderType> for u32 {
    fn from(loader_type: ModLoaderType) -> Self {
        loader_type as u32
    }
}

/// Response from GET /v1/mods/{modId}/files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetModFilesResponse {
    pub data: Vec<File>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub id: u32,
    #[serde(rename = "gameId")]
    pub game_id: u32,
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: FileReleaseType,
    #[serde(rename = "fileStatus")]
    pub file_status: FileStatus,
    pub hashes: Vec<FileHash>,
    #[serde(rename = "fileDate")]
    pub file_date: String,
    #[serde(rename = "fileLength")]
    pub file_length: u64,
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
    #[serde(rename = "sortableGameVersions")]
    pub sortable_game_versions: Vec<SortableGameVersion>,
    pub dependencies: Option<Vec<FileDependency>>,
    #[serde(rename = "exposeAsAlternative")]
    pub expose_as_alternative: Option<bool>,
    #[serde(rename = "parentProjectFileId")]
    pub parent_project_file_id: Option<u32>,
    #[serde(rename = "alternateFileId")]
    pub alternate_file_id: Option<u32>,
    #[serde(rename = "isServerPack")]
    pub is_server_pack: Option<bool>,
    #[serde(rename = "serverPackFileId")]
    pub server_pack_file_id: Option<u32>,
    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: u64,
    pub modules: Option<Vec<FileModule>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileHash {
    pub value: String,
    pub algo: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortableGameVersion {
    #[serde(rename = "gameVersionName")]
    pub game_version_name: String,
    #[serde(rename = "gameVersionPadded")]
    pub game_version_padded: String,
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "gameVersionReleaseDate")]
    pub game_version_release_date: String,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDependency {
    #[serde(rename = "modId")]
    pub mod_id: u32,
    #[serde(rename = "fileId")]
    pub file_id: Option<u32>,
    #[serde(rename = "relationType")]
    pub relation_type: FileRelationType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileModule {
    pub name: String,
    pub fingerprint: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "u32", into = "u32")]
pub enum FileReleaseType {
    Release = 1,
    Beta = 2,
    Alpha = 3,
}

impl From<u32> for FileReleaseType {
    fn from(val: u32) -> Self {
        match val {
            1 => FileReleaseType::Release,
            2 => FileReleaseType::Beta,
            3 => FileReleaseType::Alpha,
            _ => FileReleaseType::Release, // Default case
        }
    }
}

impl From<FileReleaseType> for u32 {
    fn from(release_type: FileReleaseType) -> Self {
        release_type as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "u32", into = "u32")]
pub enum FileStatus {
    Processing = 1,
    ChangesRequired = 2,
    UnderReview = 3,
    Approved = 4,
    Rejected = 5,
    MalwareDetected = 6,
    Deleted = 7,
    Archived = 8,
    Testing = 9,
    Released = 10,
    ReadyForReview = 11,
    Deprecated = 12,
    Baking = 13,
    AwaitingPublishing = 14,
    FailedPublishing = 15,
}

impl From<u32> for FileStatus {
    fn from(val: u32) -> Self {
        match val {
            1 => FileStatus::Processing,
            2 => FileStatus::ChangesRequired,
            3 => FileStatus::UnderReview,
            4 => FileStatus::Approved,
            5 => FileStatus::Rejected,
            6 => FileStatus::MalwareDetected,
            7 => FileStatus::Deleted,
            8 => FileStatus::Archived,
            9 => FileStatus::Testing,
            10 => FileStatus::Released,
            11 => FileStatus::ReadyForReview,
            12 => FileStatus::Deprecated,
            13 => FileStatus::Baking,
            14 => FileStatus::AwaitingPublishing,
            15 => FileStatus::FailedPublishing,
            _ => FileStatus::Processing, // Default case
        }
    }
}

impl From<FileStatus> for u32 {
    fn from(status: FileStatus) -> Self {
        status as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "u32", into = "u32")]
pub enum FileRelationType {
    EmbeddedLibrary = 1,
    OptionalDependency = 2,
    RequiredDependency = 3,
    Tool = 4,
    Incompatible = 5,
    Include = 6,
}

impl From<u32> for FileRelationType {
    fn from(val: u32) -> Self {
        match val {
            1 => FileRelationType::EmbeddedLibrary,
            2 => FileRelationType::OptionalDependency,
            3 => FileRelationType::RequiredDependency,
            4 => FileRelationType::Tool,
            5 => FileRelationType::Incompatible,
            6 => FileRelationType::Include,
            _ => FileRelationType::RequiredDependency, // Default case
        }
    }
}

impl From<FileRelationType> for u32 {
    fn from(relation_type: FileRelationType) -> Self {
        relation_type as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub index: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "resultCount")]
    pub result_count: u32,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub minecraft: ManifestMinecraft,
    #[serde(rename = "manifestType")]
    pub manifest_type: String,
    #[serde(rename = "manifestVersion")]
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<ManifestFile>,
    pub overrides: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestMinecraft {
    pub version: String,
    #[serde(rename = "modLoaders")]
    pub mod_loaders: Vec<ManifestModLoader>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    pub required: bool,
}
