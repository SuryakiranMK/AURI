// Project AURI: YouTube Music Innertube API Client
// Native lightweight client with zero browser dependencies.
// Searches music metadata and resolves direct audio streams (WebM Opus / MP4 AAC).

use super::{AudioStreamFormat, MusicProvider, ProviderError, ResolvedStream, SearchResultItem};
use serde_json::{json, Value};

const INNERTUBE_SEARCH_URL: &str = "https://music.youtube.com/youtubei/v1/search";
const INNERTUBE_PLAYER_URL: &str = "https://music.youtube.com/youtubei/v1/player";

pub struct InnertubeProvider {
    client_version: &'static str,
}

impl InnertubeProvider {
    pub fn new() -> Self {
        Self {
            client_version: "1.20231204.01.00",
        }
    }

    /// Formats a duration string like "3:45" or "1:02:15" into seconds
    fn parse_duration(d: &str) -> f64 {
        let parts: Vec<&str> = d.split(':').collect();
        match parts.len() {
            2 => {
                let m: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let s: f64 = parts[1].trim().parse().unwrap_or(0.0);
                m * 60.0 + s
            }
            3 => {
                let h: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let m: f64 = parts[1].trim().parse().unwrap_or(0.0);
                let s: f64 = parts[2].trim().parse().unwrap_or(0.0);
                h * 3600.0 + m * 60.0 + s
            }
            _ => 0.0,
        }
    }

    /// Recursively extracts all musicResponsiveListItemRenderer items from arbitrary JSON
    fn find_shelf_items(val: &Value, results: &mut Vec<Value>) {
        match val {
            Value::Object(map) => {
                if let Some(item) = map.get("musicResponsiveListItemRenderer") {
                    results.push(item.clone());
                } else {
                    for v in map.values() {
                        Self::find_shelf_items(v, results);
                    }
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    Self::find_shelf_items(v, results);
                }
            }
            _ => {}
        }
    }
}

impl MusicProvider for InnertubeProvider {
    fn name(&self) -> &'static str {
        "YouTube Music (Innertube)"
    }

    fn search(&self, query: &str) -> Result<Vec<SearchResultItem>, ProviderError> {
        let payload = json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": self.client_version,
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query
        });

        let resp = ureq::post(INNERTUBE_SEARCH_URL)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Referer", "https://music.youtube.com/")
            .send_json(&payload)
            .map_err(|e| ProviderError::Network(format!("Search request failed: {}", e)))?;

        let mut body = resp.into_body();
        let json_val: Value = body
            .read_json()
            .map_err(|e| ProviderError::Parse(format!("Failed to parse search JSON: {}", e)))?;

        let mut raw_items = Vec::new();
        Self::find_shelf_items(&json_val, &mut raw_items);

        let mut items = Vec::new();
        for item in raw_items {
            let flex_cols = match item.get("flexColumns").and_then(|v| v.as_array()) {
                Some(cols) if cols.len() >= 2 => cols,
                _ => continue,
            };

            // Extract Video ID with multiple resilient strategies
            let video_id = item
                .get("playlistItemData")
                .and_then(|d| d.get("videoId"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    item.get("overlay")
                        .and_then(|o| o.get("musicItemThumbnailOverlayRenderer"))
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.get("musicPlayButtonRenderer"))
                        .and_then(|p| p.get("playNavigationEndpoint"))
                        .and_then(|ne| ne.get("watchEndpoint"))
                        .and_then(|w| w.get("videoId"))
                        .and_then(|v| v.as_str())
                })
                .or_else(|| {
                    flex_cols[0]
                        .get("musicResponsiveListItemFlexColumnRenderer")
                        .and_then(|r| r.get("text"))
                        .and_then(|t| t.get("runs"))
                        .and_then(|r| r.get(0))
                        .and_then(|run| run.get("navigationEndpoint"))
                        .and_then(|ne| ne.get("watchEndpoint"))
                        .and_then(|w| w.get("videoId"))
                        .and_then(|v| v.as_str())
                })
                .or_else(|| {
                    // Search all flexColumns runs for watchEndpoint
                    for col in flex_cols {
                        if let Some(runs) = col
                            .get("musicResponsiveListItemFlexColumnRenderer")
                            .and_then(|r| r.get("text"))
                            .and_then(|t| t.get("runs"))
                            .and_then(|r| r.as_array())
                        {
                            for run in runs {
                                if let Some(vid) = run
                                    .get("navigationEndpoint")
                                    .and_then(|ne| ne.get("watchEndpoint"))
                                    .and_then(|w| w.get("videoId"))
                                    .and_then(|v| v.as_str())
                                {
                                    if !vid.is_empty() {
                                        return Some(vid);
                                    }
                                }
                            }
                        }
                    }
                    None
                });

            let video_id = match video_id {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => continue,
            };

            // Title from Column 0
            let title = flex_cols[0]
                .get("musicResponsiveListItemFlexColumnRenderer")
                .and_then(|r| r.get("text"))
                .and_then(|t| t.get("runs"))
                .and_then(|r| r.get(0))
                .and_then(|run| run.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Title")
                .to_string();

            // Artist, Album, Duration from Column 1
            let col1_runs = flex_cols[1]
                .get("musicResponsiveListItemFlexColumnRenderer")
                .and_then(|r| r.get("text"))
                .and_then(|t| t.get("runs"))
                .and_then(|r| r.as_array());

            let mut artist = "Unknown Artist".to_string();
            let mut album = "Single / Unknown Album".to_string();
            let mut duration_str = "0:00".to_string();

            if let Some(runs) = col1_runs {
                let text_runs: Vec<&str> = runs
                    .iter()
                    .filter_map(|r| r.get("text").and_then(|v| v.as_str()))
                    .filter(|s| *s != " • " && !s.trim().is_empty())
                    .collect();

                // Check for duration (last run containing ':')
                for s in text_runs.iter().rev() {
                    if s.contains(':') && s.chars().all(|c| c.is_ascii_digit() || c == ':') {
                        duration_str = s.to_string();
                        break;
                    }
                }

                // Clean artist (first non-type text)
                for s in &text_runs {
                    if *s != "Song" && *s != "Video" && !s.contains("views") && !s.contains(':') {
                        artist = s.to_string();
                        break;
                    }
                }

                // Clean album if present
                if text_runs.len() >= 3 {
                    for s in &text_runs[1..] {
                        if *s != artist.as_str() && *s != "Song" && *s != "Video" && !s.contains("views") && !s.contains(':') {
                            album = s.to_string();
                            break;
                        }
                    }
                }
            }

            // Thumbnail
            let thumbnail_url = item
                .get("thumbnail")
                .and_then(|t| t.get("musicThumbnailRenderer"))
                .and_then(|r| r.get("thumbnail"))
                .and_then(|t| t.get("thumbnails"))
                .and_then(|arr| arr.get(0))
                .and_then(|th| th.get("url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let duration_secs = Self::parse_duration(&duration_str);

            items.push(SearchResultItem {
                id: video_id,
                title,
                artist,
                album,
                duration_secs,
                formatted_duration: duration_str,
                thumbnail_url,
            });
        }

        Ok(items)
    }

    fn resolve_stream(&self, track_id: &str) -> Result<ResolvedStream, ProviderError> {
        let payload = json!({
            "context": {
                "client": {
                    "clientName": "ANDROID_MUSIC",
                    "clientVersion": "6.42.52",
                    "hl": "en",
                    "gl": "US",
                    "androidSdkVersion": 34
                }
            },
            "videoId": track_id
        });

        let resp = ureq::post(INNERTUBE_PLAYER_URL)
            .header("Content-Type", "application/json")
            .header("User-Agent", "com.google.android.apps.youtube.music/6.42.52 (Linux; U; Android 14)")
            .send_json(&payload)
            .map_err(|e| ProviderError::Network(format!("Player resolution request failed: {}", e)))?;

        let mut body = resp.into_body();
        let json_val: Value = body
            .read_json()
            .map_err(|e| ProviderError::Parse(format!("Failed to parse player JSON: {}", e)))?;

        let adaptive_formats = json_val
            .get("streamingData")
            .and_then(|sd| sd.get("adaptiveFormats"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::Unavailable(format!("No streaming formats available for {}", track_id))
            })?;

        // Filter and score audio formats:
        // Prefer WebM Opus (high audio fidelity ~160kbps) > MP4 AAC (~128kbps)
        let mut candidates: Vec<(u32, AudioStreamFormat, String, f64)> = Vec::new();

        for fmt in adaptive_formats {
            let mime = match fmt.get("mimeType").and_then(|v| v.as_str()) {
                Some(m) if m.starts_with("audio/") => m,
                _ => continue,
            };

            let stream_url = match fmt.get("url").and_then(|v| v.as_str()) {
                Some(u) if !u.is_empty() => u.to_string(),
                _ => continue,
            };

            let bitrate = fmt
                .get("bitrate")
                .and_then(|v| v.as_u64())
                .unwrap_or(128000) as u32;

            let duration_ms = fmt
                .get("approxDurationMs")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let duration_secs = duration_ms / 1000.0;

            let format = if mime.contains("opus") {
                AudioStreamFormat::WebmOpus
            } else if mime.contains("mp4") || mime.contains("aac") {
                AudioStreamFormat::Mp4Aac
            } else {
                AudioStreamFormat::Mp3
            };

            candidates.push((bitrate, format, stream_url, duration_secs));
        }

        // Sort by bitrate descending
        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        let best = candidates.into_iter().next().ok_or_else(|| {
            ProviderError::Unavailable(format!(
                "No direct playable audio streams found for video ID: {}",
                track_id
            ))
        })?;

        Ok(ResolvedStream {
            video_id: track_id.to_string(),
            stream_url: best.2,
            format: best.1,
            bitrate_bps: best.0,
            duration_secs: best.3,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_parsing() {
        assert_eq!(InnertubeProvider::parse_duration("3:45"), 225.0);
        assert_eq!(InnertubeProvider::parse_duration("1:02:15"), 3735.0);
        assert_eq!(InnertubeProvider::parse_duration("0:30"), 30.0);
        assert_eq!(InnertubeProvider::parse_duration("invalid"), 0.0);
    }

    #[test]
    fn test_search_parser_mock() {
        let mock_json = json!({
            "contents": {
                "tabbedSearchResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [{
                                        "musicShelfRenderer": {
                                            "contents": [{
                                                "musicResponsiveListItemRenderer": {
                                                    "playlistItemData": {
                                                        "videoId": "mock_vid_123"
                                                    },
                                                    "flexColumns": [
                                                        {
                                                            "musicResponsiveListItemFlexColumnRenderer": {
                                                                "text": {
                                                                    "runs": [{"text": "Mock Track Title"}]
                                                                }
                                                            }
                                                        },
                                                        {
                                                            "musicResponsiveListItemFlexColumnRenderer": {
                                                                "text": {
                                                                    "runs": [
                                                                        {"text": "Mock Artist"},
                                                                        {"text": " • "},
                                                                        {"text": "Mock Album"},
                                                                        {"text": " • "},
                                                                        {"text": "3:24"}
                                                                    ]
                                                                }
                                                            }
                                                        }
                                                    ],
                                                    "thumbnail": {
                                                        "musicThumbnailRenderer": {
                                                            "thumbnail": {
                                                                "thumbnails": [{"url": "https://lh3.googleusercontent.com/mock.jpg"}]
                                                            }
                                                        }
                                                    }
                                                }
                                            }]
                                        }
                                    }]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let mut raw_items = Vec::new();
        InnertubeProvider::find_shelf_items(&mock_json, &mut raw_items);
        assert_eq!(raw_items.len(), 1);
    }
}

