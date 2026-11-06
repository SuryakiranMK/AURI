// AURI YouTube Music Innertube Client
// High-efficiency, zero-browser YouTube Music API client with multi-criteria stream scoring

use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use ureq::Agent;

use super::TrackItem;

const PLAYER_API: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
const SEARCH_API: &str = "https://music.youtube.com/youtubei/v1/search?prettyPrint=false";
const VISITOR_API: &str = "https://www.youtube.com/youtubei/v1/visitor_id?prettyPrint=false";

const WEB_REMIX_VERSION: &str = "2.20260707.00.00";
const ANDROID_CLIENT_VERSION: &str = "19.29.35";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const ANDROID_USER_AGENT: &str = "com.google.android.youtube/19.29.35 (Linux; U; Android 14) gzip";

const OPUS_ITAG: u64 = 251; // WebM Opus ~160kbps
const AAC_ITAG: u64 = 140;  // MP4 AAC ~128kbps

#[derive(Debug, Clone)]
pub struct ResolvedStream {
    pub url: String,
    pub content_length: u64,
    pub duration_secs: u64,
    pub loudness_db: Option<f64>,
    pub is_opus: bool,
    pub extension_hint: &'static str,
    pub user_agent: String,
}

pub struct InnertubeClient {
    agent: Agent,
    visitor_id: Mutex<String>,
}

impl Default for InnertubeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl InnertubeClient {
    pub fn new() -> Self {
        let agent = Agent::new_with_config(
            ureq::config::Config::builder()
                .timeout_global(Some(Duration::from_secs(12)))
                .build(),
        );

        Self {
            agent,
            visitor_id: Mutex::new(String::new()),
        }
    }

    fn get_visitor_id(&self) -> String {
        let mut id = self.visitor_id.lock().unwrap();
        if !id.is_empty() {
            return id.clone();
        }

        let body = json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": WEB_REMIX_VERSION,
                    "hl": "en"
                }
            }
        });

        if let Ok(res) = self
            .agent
            .post(VISITOR_API)
            .header("User-Agent", USER_AGENT)
            .send_json(&body)
        {
            if let Ok(val) = res.into_body().read_json::<Value>() {
                if let Some(v) = val["responseContext"]["visitorData"].as_str() {
                    *id = v.to_string();
                    return v.to_string();
                }
            }
        }

        String::new()
    }

    /// Search YouTube Music for tracks matching query
    pub fn search(&self, query: &str) -> Result<Vec<TrackItem>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let visitor = self.get_visitor_id();
        let body = json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": WEB_REMIX_VERSION,
                    "hl": "en",
                    "visitorData": visitor
                }
            },
            "query": query,
            // Song filter params
            "params": "Eg-KAQwIABAAGAAgACgAMABqChAEEAMQCRAFEAo%3D"
        });

        let res = self
            .agent
            .post(SEARCH_API)
            .header("User-Agent", USER_AGENT)
            .header("Origin", "https://music.youtube.com")
            .header("X-YouTube-Client-Name", "67")
            .header("X-YouTube-Client-Version", WEB_REMIX_VERSION)
            .send_json(&body)
            .map_err(|e| format!("Search request failed: {}", e))?;

        let json_val = res
            .into_body()
            .read_json::<Value>()
            .map_err(|e| format!("Failed to parse search response: {}", e))?;

        let mut tracks = Vec::new();
        parse_search_results(&json_val, &mut tracks);
        Ok(tracks)
    }

    /// Resolve playable stream URL and audio metadata for a YouTube video ID
    pub fn resolve_stream(&self, video_id: &str) -> Result<ResolvedStream, String> {
        let visitor = self.get_visitor_id();

        // Query Android Client context (most reliable direct stream URLs without cipher)
        let body = json!({
            "context": {
                "client": {
                    "clientName": "ANDROID",
                    "clientVersion": ANDROID_CLIENT_VERSION,
                    "hl": "en",
                    "visitorData": visitor
                }
            },
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true
        });

        let res = self
            .agent
            .post(PLAYER_API)
            .header("User-Agent", ANDROID_USER_AGENT)
            .header("X-YouTube-Client-Name", "3")
            .header("X-YouTube-Client-Version", ANDROID_CLIENT_VERSION)
            .send_json(&body)
            .map_err(|e| format!("Player API request failed: {}", e))?;

        let json_val = res
            .into_body()
            .read_json::<Value>()
            .map_err(|e| format!("Failed to parse player response: {}", e))?;

        // Extract loudnessDb metadata (YouTube Music standard loudness tag)
        let loudness_db = json_val["playerConfig"]["audioConfig"]["loudnessDb"]
            .as_f64()
            .or_else(|| json_val["videoDetails"]["loudnessDb"].as_f64());

        // Extract adaptiveFormats
        let formats = json_val["streamingData"]["adaptiveFormats"]
            .as_array()
            .ok_or_else(|| "No streaming adaptiveFormats available".to_string())?;

        // 1. Prioritize Opus ~160kbps (itag 251)
        if let Some(opus_fmt) = formats.iter().find(|f| f["itag"].as_u64() == Some(OPUS_ITAG)) {
            if let Some(url) = opus_fmt["url"].as_str() {
                let len = opus_fmt["contentLength"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let dur = opus_fmt["approxDurationMs"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1000;

                return Ok(ResolvedStream {
                    url: url.to_string(),
                    content_length: len,
                    duration_secs: dur,
                    loudness_db,
                    is_opus: true,
                    extension_hint: "webm",
                    user_agent: ANDROID_USER_AGENT.to_string(),
                });
            }
        }

        // 2. Fallback to AAC ~128kbps (itag 140)
        if let Some(aac_fmt) = formats.iter().find(|f| f["itag"].as_u64() == Some(AAC_ITAG)) {
            if let Some(url) = aac_fmt["url"].as_str() {
                let len = aac_fmt["contentLength"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let dur = aac_fmt["approxDurationMs"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1000;

                return Ok(ResolvedStream {
                    url: url.to_string(),
                    content_length: len,
                    duration_secs: dur,
                    loudness_db,
                    is_opus: false,
                    extension_hint: "m4a",
                    user_agent: ANDROID_USER_AGENT.to_string(),
                });
            }
        }

        Err("No compatible un-ciphered audio stream found for track".to_string())
    }
}

fn parse_search_results(val: &Value, tracks: &mut Vec<TrackItem>) {
    // Search responses have nested renderers: contents -> tabbedSearchResultsRenderer -> ...
    let search_contents = &val["contents"]["tabbedSearchResultsRenderer"]["tabs"][0]["tabRenderer"]
        ["content"]["sectionListRenderer"]["contents"];

    if let Some(sections) = search_contents.as_array() {
        for sec in sections {
            let shelf_contents = &sec["musicShelfRenderer"]["contents"];
            if let Some(items) = shelf_contents.as_array() {
                for item in items {
                    let r = &item["musicResponsiveListItemRenderer"];
                    if r.is_null() {
                        continue;
                    }

                    // Extract Video ID
                    let video_id = r["playlistItemData"]["videoId"]
                        .as_str()
                        .or_else(|| {
                            r["overlay"]["musicItemThumbnailOverlayRenderer"]["content"]
                                ["musicPlayButtonRenderer"]["playNavigationEndpoint"]
                                ["watchEndpoint"]["videoId"]
                                .as_str()
                        })
                        .unwrap_or_default()
                        .to_string();

                    if video_id.is_empty() {
                        continue;
                    }

                    // Extract Title
                    let title = r["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]
                        ["text"]["runs"][0]["text"]
                        .as_str()
                        .unwrap_or("Unknown Title")
                        .to_string();

                    // Extract Artist
                    let artist = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]
                        ["text"]["runs"][0]["text"]
                        .as_str()
                        .unwrap_or("Unknown Artist")
                        .to_string();

                    // Extract Album
                    let album = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]
                        ["text"]["runs"]
                        .as_array()
                        .and_then(|runs| runs.get(2))
                        .and_then(|r| r["text"].as_str())
                        .unwrap_or("YouTube Music")
                        .to_string();

                    // Extract Duration
                    let duration_str = r["flexColumns"][1]["musicResponsiveListItemFlexColumnRenderer"]
                        ["text"]["runs"]
                        .as_array()
                        .and_then(|runs| runs.last())
                        .and_then(|r| r["text"].as_str())
                        .unwrap_or("3:00");
                    let duration_secs = parse_duration_string(duration_str);

                    // Extract Thumbnail
                    let thumb_url = r["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
                        .as_array()
                        .and_then(|arr| arr.last())
                        .and_then(|t| t["url"].as_str())
                        .unwrap_or_default()
                        .to_string();

                    tracks.push(TrackItem {
                        id: video_id,
                        title,
                        artist,
                        album,
                        duration_secs,
                        thumbnail_url: thumb_url,
                        is_local: false,
                        file_path: None,
                        loudness_db: None,
                        format_badge: "Opus 160k".to_string(),
                    });
                }
            }
        }
    }
}

pub fn parse_duration_string(s: &str) -> u64 {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 2 {
        let mins = parts[0].parse::<u64>().unwrap_or(0);
        let secs = parts[1].parse::<u64>().unwrap_or(0);
        mins * 60 + secs
    } else if parts.len() == 3 {
        let hours = parts[0].parse::<u64>().unwrap_or(0);
        let mins = parts[1].parse::<u64>().unwrap_or(0);
        let secs = parts[2].parse::<u64>().unwrap_or(0);
        hours * 3600 + mins * 60 + secs
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration_string("3:45"), 225);
        assert_eq!(parse_duration_string("1:02:15"), 3735);
        assert_eq!(parse_duration_string("invalid"), 0);
    }
}

