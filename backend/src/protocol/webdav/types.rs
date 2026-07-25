#[allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// WebDAV property names
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DavProperty {
    /// Resource creation date
    CreationDate,
    /// Display name
    DisplayName,
    /// Content length
    GetContentLength,
    /// Content type
    GetContentType,
    /// ETag
    GetEtag,
    /// Last modified date
    GetLastModified,
    /// Resource type (collection or resource)
    ResourceType,
    /// Supported lock types
    SupportedLock,
    /// Lock discovery
    LockDiscovery,
    /// Custom property
    Custom(String),
}

/// WebDAV lock scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DavLockScope {
    /// Exclusive lock
    Exclusive,
    /// Shared lock
    Shared,
}

/// WebDAV lock type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DavLockType {
    /// Write lock
    Write,
}

/// WebDAV lock entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavLockEntry {
    pub scope: DavLockScope,
    #[serde(rename = "type")]
    pub lock_type: DavLockType,
    pub depth: String,
    pub owner: Option<String>,
    pub timeout: Option<String>,
    pub lock_token: String,
}

/// WebDAV prop find request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropFind {
    #[serde(rename = "prop", default)]
    pub prop: Option<PropFindProp>,
    #[serde(rename = "allprop", default)]
    pub allprop: bool,
    #[serde(rename = "propname", default)]
    pub propname: bool,
}

/// Properties to find
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropFindProp {
    #[serde(rename = "$value", default)]
    pub properties: Vec<DavProperty>,
}

/// WebDAV multi-status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavMultiStatus {
    #[serde(rename = "response")]
    pub responses: Vec<DavResponse>,
}

/// WebDAV response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavResponse {
    #[serde(rename = "href")]
    pub href: String,
    #[serde(rename = "propstat")]
    pub propstat: Vec<DavPropStat>,
    #[serde(rename = "status", default)]
    pub status: Option<String>,
}

/// WebDAV prop stat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavPropStat {
    #[serde(rename = "prop")]
    pub prop: DavProp,
    #[serde(rename = "status")]
    pub status: String,
}

/// WebDAV properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavProp {
    #[serde(rename = "creationdate", default)]
    pub creation_date: Option<String>,
    #[serde(rename = "displayname", default)]
    pub display_name: Option<String>,
    #[serde(rename = "getcontentlength", default)]
    pub get_content_length: Option<u64>,
    #[serde(rename = "getcontenttype", default)]
    pub get_content_type: Option<String>,
    #[serde(rename = "getetag", default)]
    pub get_etag: Option<String>,
    #[serde(rename = "getlastmodified", default)]
    pub get_last_modified: Option<String>,
    #[serde(rename = "resourcetype", default)]
    pub resource_type: Option<DavResourceTypeElement>,
    #[serde(rename = "supportedlock", default)]
    pub supported_lock: Option<DavSupportedLock>,
    #[serde(rename = "lockdiscovery", default)]
    pub lock_discovery: Option<DavLockDiscovery>,
}

/// Resource type element for XML serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavResourceTypeElement {
    #[serde(rename = "collection", default)]
    pub collection: bool,
}

/// Supported lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavSupportedLock {
    #[serde(rename = "lockentry")]
    pub entries: Vec<DavLockEntry>,
}

/// Lock discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavLockDiscovery {
    #[serde(rename = "activelock")]
    pub active_locks: Vec<DavActiveLock>,
}

/// Active lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavActiveLock {
    #[serde(rename = "locktype")]
    pub lock_type: DavLockTypeElement,
    #[serde(rename = "lockscope")]
    pub lock_scope: DavLockScopeElement,
    pub depth: String,
    pub owner: Option<String>,
    pub timeout: Option<String>,
    #[serde(rename = "locktoken")]
    pub lock_token: DavLockToken,
}

/// Lock type element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavLockTypeElement {
    #[serde(rename = "write", default)]
    pub write: bool,
}

/// Lock scope element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavLockScopeElement {
    #[serde(rename = "exclusive", default)]
    pub exclusive: bool,
    #[serde(rename = "shared", default)]
    pub shared: bool,
}

/// Lock token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DavLockToken {
    pub href: String,
}

/// Depth header value
#[derive(Debug, Clone, PartialEq)]
pub enum Depth {
    /// Only the resource itself
    Zero,
    /// Resource and its direct children
    One,
    /// Resource and all descendants
    Infinity,
}

impl Depth {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "0" => Depth::Zero,
            "1" => Depth::One,
            _ => Depth::Infinity,
        }
    }
}

/// Convert filesystem metadata to WebDAV properties
pub fn metadata_to_prop(
    name: &str,
    _path: &str,
    is_dir: bool,
    size: u64,
    modified: DateTime<Utc>,
    created: Option<DateTime<Utc>>,
    mime_type: &str,
) -> DavProp {
    DavProp {
        creation_date: created.map(|d| d.to_rfc3339()),
        display_name: Some(name.to_string()),
        get_content_length: if is_dir { None } else { Some(size) },
        get_content_type: if is_dir {
            None
        } else {
            Some(mime_type.to_string())
        },
        get_etag: Some(format!(
            "\"{}-{}\"",
            modified.timestamp(),
            size
        )),
        get_last_modified: Some(modified.to_rfc2822()),
        resource_type: Some(DavResourceTypeElement {
            collection: is_dir,
        }),
        supported_lock: Some(DavSupportedLock {
            entries: vec![DavLockEntry {
                scope: DavLockScope::Exclusive,
                lock_type: DavLockType::Write,
                depth: "infinity".to_string(),
                owner: None,
                timeout: None,
                lock_token: String::new(),
            }],
        }),
        lock_discovery: None,
    }
}
