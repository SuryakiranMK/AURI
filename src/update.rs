use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const UPDATED_FLAG: &str = "--updated";
const DOWNLOADS: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/download");
const PROGRAM: &str = "auri.exe";
const CHECKSUMS: &str = "SHA256SUMS.txt";
const PROGRAM_BYTES: u64 = 64 * 1024 * 1024;
const SCOOP_FOLDER: &str = "\\scoop\\apps\\";
const SCOOP_COMMAND: &str = "scoop update auri";

#[derive(Clone, Debug)]
pub struct Progress {
    pub percent: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub stage: String,
    pub speed_bps: f64,
}

impl Progress {
    pub fn speed_string(&self) -> String {
        if self.speed_bps >= 1_048_576.0 {
            format!("{:.1} MB/s", self.speed_bps / 1_048_576.0)
        } else if self.speed_bps >= 1024.0 {
            format!("{:.0} KB/s", self.speed_bps / 1024.0)
        } else if self.speed_bps > 0.0 {
            format!("{:.0} B/s", self.speed_bps)
        } else {
            String::new()
        }
    }

    pub fn size_string(&self) -> String {
        let dl = self.downloaded_bytes as f64 / 1_048_576.0;
        if self.total_bytes > 0 {
            let tot = self.total_bytes as f64 / 1_048_576.0;
            format!("{:.1} / {:.1} MB", dl, tot)
        } else if self.downloaded_bytes > 0 {
            format!("{:.1} MB", dl)
        } else {
            String::new()
        }
    }
}

pub fn update_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent(&format!("AURI-Desktop/{}", env!("CARGO_PKG_VERSION")))
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(60))
        .build()
}

pub enum Plan {
    InPlace,
    PackageManager(&'static str),
    DownloadPage,
}

pub fn plan() -> Plan {
    let program = std::env::current_exe().ok().filter(|_| cfg!(windows));
    match program {
        Some(program) if program.to_string_lossy().to_lowercase().contains(SCOOP_FOLDER) => Plan::PackageManager(SCOOP_COMMAND),
        Some(_) => Plan::InPlace,
        None => Plan::DownloadPage,
    }
}

fn listed_checksum<'a>(checksums: &'a str, file: &str) -> Option<&'a str> {
    checksums.lines().find_map(|line| {
        let line = line.trim_start_matches('\u{feff}').trim();
        let (checksum, name) = line.split_once(char::is_whitespace)?;
        let checksum = checksum.trim();
        let complete = checksum.len() == 64 && checksum.chars().all(|c| c.is_ascii_hexdigit());
        (complete && name.trim().trim_start_matches('*') == file).then_some(checksum)
    })
}

fn download(
    agent: &ureq::Agent,
    release: &str,
    into: &Path,
    mut on_progress: impl FnMut(f32, u64, u64, &str, f64),
) -> Result<(), String> {
    let failed = |error: ureq::Error| format!("the download failed ({error})");

    on_progress(0.01, 0, 0, "Connecting to GitHub...", 0.0);

    // Concurrently fetch checksums so network latency is completely overlapped with binary download
    let checksum_agent = agent.clone();
    let checksum_url = format!("{release}/{CHECKSUMS}");
    let checksum_handle = std::thread::spawn(move || -> Result<String, String> {
        let resp = checksum_agent.get(&checksum_url).call().map_err(failed)?;
        resp.into_string().map_err(|e| e.to_string())
    });

    let program_url = format!("{release}/{PROGRAM}");
    let response = agent.get(&program_url).call().map_err(failed)?;
    let total_bytes = response
        .header("Content-Length")
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(0);

    let body = response.into_reader().take(PROGRAM_BYTES);
    let mut reader = BufReader::with_capacity(256 * 1024, body);
    let file = std::fs::File::create(into).map_err(|e| format!("this folder cannot be written to ({e})"))?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);
    let mut digest = ring::digest::Context::new(&ring::digest::SHA256);

    let mut block = [0u8; 128 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_progress_time = std::time::Instant::now();
    let mut last_bytes: u64 = 0;
    let mut speed_bps: f64 = 0.0;

    on_progress(0.02, 0, total_bytes, "Downloading update package...", 0.0);

    loop {
        let read = reader.read(&mut block).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&block[..read]);
        writer.write_all(&block[..read]).map_err(|e| e.to_string())?;
        downloaded += read as u64;

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_progress_time).as_secs_f64();
        if elapsed >= 0.05 || (total_bytes > 0 && downloaded >= total_bytes) {
            let instant_speed = (downloaded - last_bytes) as f64 / elapsed.max(0.001);
            speed_bps = if speed_bps == 0.0 {
                instant_speed
            } else {
                speed_bps * 0.7 + instant_speed * 0.3
            };
            last_progress_time = now;
            last_bytes = downloaded;

            let pct = if total_bytes > 0 {
                (downloaded as f32 / total_bytes as f32).clamp(0.0, 1.0) * 0.90 + 0.02
            } else {
                0.02 + (downloaded as f32 / (15.0 * 1024.0 * 1024.0)).clamp(0.0, 0.90)
            };
            on_progress(pct, downloaded, total_bytes, "Downloading update package...", speed_bps);
        }
    }
    writer.flush().map_err(|e| e.to_string())?;
    drop(writer);

    on_progress(0.93, downloaded, total_bytes, "Verifying SHA-256 checksum...", speed_bps);
    let checksums = checksum_handle
        .join()
        .map_err(|_| "checksum thread panicked".to_string())??;
    let expected = listed_checksum(&checksums, PROGRAM).ok_or("the release lists no checksum for the program")?;
    let actual: String = digest.finish().as_ref().iter().map(|byte| format!("{byte:02x}")).collect();
    if !actual.eq_ignore_ascii_case(expected) {
        let _ = std::fs::remove_file(into);
        return Err("the download does not match its published checksum".to_owned());
    }

    on_progress(0.97, downloaded, total_bytes, "Checksum verified", 0.0);
    Ok(())
}

fn swap(program: &Path, fresh: &Path) -> Result<(), String> {
    let previous = program.with_extension("old");
    let _ = std::fs::remove_file(&previous);
    std::fs::rename(program, &previous).map_err(|e| format!("the running program could not be moved aside ({e})"))?;
    std::fs::rename(fresh, program).map_err(|error| {
        let _ = std::fs::rename(&previous, program);
        format!("the new program could not be put in place ({error})")
    })
}

pub fn install(
    agent: &ureq::Agent,
    version: &str,
    mut on_progress: impl FnMut(f32, u64, u64, &str, f64),
) -> Result<PathBuf, String> {
    let program = std::env::current_exe().map_err(|e| e.to_string())?;
    let fresh = program.with_extension("new");
    download(agent, &format!("{DOWNLOADS}/v{version}"), &fresh, &mut on_progress)?;
    on_progress(0.98, 0, 0, "Applying update...", 0.0);
    swap(&program, &fresh)?;
    on_progress(1.0, 0, 0, "Complete! Restarting...", 0.0);
    Ok(program)
}

pub fn restart(program: &Path, arguments: &[String]) {
    let _ = Command::new(program).args(arguments).arg(UPDATED_FLAG).spawn();
}

pub fn clear_leftovers() {
    if let Ok(program) = std::env::current_exe() {
        let _ = std::fs::remove_file(program.with_extension("old"));
        let _ = std::fs::remove_file(program.with_extension("new"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    const NEW_PROGRAM: &[u8] = b"the next version of the program";
    const WRONG_CHECKSUM: &str = "0b8c9e6c1f1b6f0d3c0f2a3a4b5c6d7e8f90123456789abcdef0123456789abc";

    fn serve(checksum: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let checksum = std::sync::Arc::new(checksum);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let checksum = checksum.clone();
                std::thread::spawn(move || {
                    let mut stream = stream;
                    let mut request = String::new();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    if reader.read_line(&mut request).is_ok() {
                        while reader.read_line(&mut String::new()).unwrap_or(0) > 2 {}
                        let body = if request.contains(CHECKSUMS) {
                            format!("{}  other.zip\n{checksum}  {PROGRAM}\n", "a".repeat(64)).into_bytes()
                        } else {
                            NEW_PROGRAM.to_vec()
                        };
                        let head = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                        let _ = stream.write_all(head.as_bytes());
                        let _ = stream.write_all(&body);
                    }
                });
            }
        });
        address
    }

    fn sha256(bytes: &[u8]) -> String {
        let digest = ring::digest::digest(&ring::digest::SHA256, bytes);
        digest.as_ref().iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn reads_only_a_complete_checksum_for_the_named_file() {
        let good = "f".repeat(64);
        let list = format!(
            "{}  auri-windows-x64.zip\n{good} *auri.exe\nshort  auri.exe\n",
            "a".repeat(64)
        );
        assert_eq!(listed_checksum(&list, "auri.exe"), Some(good.as_str()));
        let bom_list = format!("\u{feff}{good}  auri.exe\n");
        assert_eq!(listed_checksum(&bom_list, "auri.exe"), Some(good.as_str()));
        assert_eq!(listed_checksum(&list, "missing.exe"), None);
        assert_eq!(listed_checksum("abc  auri.exe", "auri.exe"), None);
    }

    #[test]
    fn installs_a_verified_download_and_refuses_a_tampered_one() {
        let folder = std::env::temp_dir().join(format!("auri-update-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let (program, fresh) = (folder.join("Auri.exe"), folder.join("Auri.new"));
        std::fs::write(&program, b"the running version").unwrap();
        let agent = ureq::agent();

        let tampered = download(&agent, &serve(WRONG_CHECKSUM.to_owned()), &fresh, |_, _, _, _, _| ());
        assert!(tampered.unwrap_err().contains("checksum"));
        assert!(!fresh.exists());
        assert_eq!(std::fs::read(&program).unwrap(), b"the running version");

        download(&agent, &serve(sha256(NEW_PROGRAM)), &fresh, |_, _, _, _, _| ()).unwrap();
        swap(&program, &fresh).unwrap();
        assert_eq!(std::fs::read(&program).unwrap(), NEW_PROGRAM);
        assert_eq!(std::fs::read(folder.join("Auri.old")).unwrap(), b"the running version");
        assert!(!fresh.exists());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
