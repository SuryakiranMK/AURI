use crate::cache::get_cache_dir;
use crate::ytm;
use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureId, TextureOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime};

const WORKERS: usize = 2;
const DEFAULT_BUDGET_BYTES: usize = 4 * 1024 * 1024;
const MAX_DOWNLOAD_BYTES: u64 = 6 * 1024 * 1024;
const MAX_ART_DISK_BYTES: u64 = 64 * 1024 * 1024;
const RETRY_AFTER: Duration = Duration::from_secs(20);
// ponytail: the 8 px size is the blur, stretched with linear filtering
const SIZES: [u32; 5] = [8, 64, 128, 256, 512];

type Sized = (String, u32);
type Jobs = Arc<(Mutex<Vec<Sized>>, Condvar)>;
type Loaded = (Sized, Option<(ColorImage, Color32)>);

#[derive(Clone, Copy)]
pub struct Picture {
    pub id: TextureId,
    pub tint: Color32,
}

enum Slot {
    Loading,
    Failed(Instant),
    Ready {
        texture: TextureHandle,
        tint: Color32,
        used: u64,
        bytes: usize,
    },
}

#[derive(Default)]
struct Cache {
    slots: HashMap<Sized, Slot>,
    frame: u64,
    bytes: usize,
}

pub struct Art {
    cache: RefCell<Cache>,
    jobs: Jobs,
    loaded: Receiver<Loaded>,
    budget: Arc<AtomicUsize>,
}

impl Art {
    pub fn new(ctx: egui::Context, agent: ureq::Agent) -> Self {
        let jobs = Jobs::default();
        let (sender, loaded) = mpsc::channel();
        for _ in 0..WORKERS {
            let (ctx, agent, jobs, sender) = (ctx.clone(), agent.clone(), jobs.clone(), sender.clone());
            std::thread::spawn(move || work(ctx, agent, jobs, sender));
        }
        Self {
            cache: RefCell::default(),
            jobs,
            loaded,
            budget: Arc::new(AtomicUsize::new(DEFAULT_BUDGET_BYTES)),
        }
    }

    pub fn set_budget_bytes(&self, bytes: usize) {
        self.budget.store(bytes, Relaxed);
    }

    pub fn begin_frame(&self, ctx: &egui::Context) {
        let mut cache = self.cache.borrow_mut();
        cache.frame += 1;
        let frame = cache.frame;
        while let Ok((key, result)) = self.loaded.try_recv() {
            let slot = match result {
                Some((image, tint)) => {
                    let bytes = image.width() * image.height() * 4;
                    cache.bytes += bytes;
                    Slot::Ready {
                        texture: ctx.load_texture(&key.0, image, TextureOptions::LINEAR),
                        tint,
                        used: frame,
                        bytes,
                    }
                }
                None => Slot::Failed(Instant::now()),
            };
            cache.slots.insert(key, slot);
        }
        let budget = self.budget.load(Relaxed);
        if cache.bytes > budget {
            let mut idle: Vec<(u64, usize, Sized)> = cache
                .slots
                .iter()
                .filter_map(|(key, slot)| match slot {
                    Slot::Ready { used, bytes, .. } if used + 1 < frame => Some((*used, *bytes, key.clone())),
                    _ => None,
                })
                .collect();
            idle.sort_unstable();
            for (_, bytes, key) in idle {
                if cache.bytes <= budget * 3 / 4 {
                    break;
                }
                cache.slots.remove(&key);
                cache.bytes -= bytes;
            }
        }
    }

    pub fn get(&self, ctx: &egui::Context, url: &str, points: f32) -> Option<Picture> {
        if url.is_empty() {
            return None;
        }
        let wanted = (points * ctx.pixels_per_point()).ceil() as u32;
        let px = SIZES.into_iter().find(|size| *size >= wanted).unwrap_or(SIZES[SIZES.len() - 1]);
        let key = (ytm::sized(url, px), px);
        let mut cache = self.cache.borrow_mut();
        let frame = cache.frame;
        match cache.slots.get_mut(&key) {
            Some(Slot::Ready { texture, tint, used, .. }) => {
                *used = frame;
                return Some(Picture {
                    id: texture.id(),
                    tint: *tint,
                });
            }
            Some(Slot::Loading) => return None,
            Some(Slot::Failed(at)) if at.elapsed() < RETRY_AFTER => return None,
            _ => {}
        }
        cache.slots.insert(key.clone(), Slot::Loading);
        self.jobs.0.lock().unwrap().push(key);
        self.jobs.1.notify_one();
        None
    }
}

fn work(ctx: egui::Context, agent: ureq::Agent, jobs: Jobs, sender: Sender<Loaded>) {
    loop {
        let key = {
            let mut queue = jobs.0.lock().unwrap();
            loop {
                match queue.pop() {
                    Some(job) => break job,
                    None => queue = jobs.1.wait(queue).unwrap(),
                }
            }
        };
        let result = load(&agent, &key.0, key.1);
        if sender.send((key, result)).is_err() {
            return;
        }
        ctx.request_repaint();
    }
}

fn art_cache_dir() -> PathBuf {
    let dir = get_cache_dir().join("art");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn art_cache_path(url: &str) -> PathBuf {
    let digest = ring::digest::digest(&ring::digest::SHA256, url.as_bytes());
    let hex: String = digest.as_ref().iter().take(16).map(|b| format!("{b:02x}")).collect();
    art_cache_dir().join(format!("{hex}.img"))
}

fn trim_art_disk_cache_if_needed() {
    let dir = art_cache_dir();
    let mut files = Vec::new();
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let len = meta.len();
                    total += len;
                    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    files.push((entry.path(), len, mtime));
                }
            }
        }
    }
    if total > MAX_ART_DISK_BYTES {
        files.sort_by_key(|(_, _, mtime)| *mtime);
        let target = MAX_ART_DISK_BYTES * 3 / 4;
        for (path, len, _) in files {
            if total <= target {
                break;
            }
            if std::fs::remove_file(path).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }
}

fn fetch(agent: &ureq::Agent, url: &str) -> Option<image::DynamicImage> {
    let path = art_cache_path(url);
    if let Ok(bytes) = std::fs::read(&path) {
        if !bytes.is_empty() {
            if let Ok(img) = image::load_from_memory(&bytes) {
                return Some(img);
            }
        }
    }

    let mut bytes = Vec::new();
    agent
        .get(url)
        .call()
        .ok()?
        .into_reader()
        .take(MAX_DOWNLOAD_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;

    if let Ok(img) = image::load_from_memory(&bytes) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &bytes);
        trim_art_disk_cache_if_needed();
        Some(img)
    } else {
        None
    }
}

fn load(agent: &ureq::Agent, url: &str, px: u32) -> Option<(ColorImage, Color32)> {
    let sharp = ytm::sharper(url, px).and_then(|sharp| fetch(agent, &sharp));
    let decoded = sharp.or_else(|| fetch(agent, url))?;
    let side = decoded.width().min(decoded.height());
    let square = decoded.crop_imm((decoded.width() - side) / 2, (decoded.height() - side) / 2, side, side);
    let fitted = if side > px { square.thumbnail_exact(px, px) } else { square };
    let rgba = fitted.to_rgba8();
    let mut sum = [0u64; 3];
    let mut count = 0u64;
    for pixel in rgba.pixels().step_by(5) {
        for channel in 0..3 {
            sum[channel] += pixel[channel] as u64;
        }
        count += 1;
    }
    let mean = sum.map(|total| (total / count.max(1)) as u8);
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some((
        ColorImage::from_rgba_unmultiplied(size, &rgba),
        Color32::from_rgb(mean[0], mean[1], mean[2]),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_art_cache_path_deterministic() {
        let url = "https://lh3.googleusercontent.com/test_cover=w512-h512";
        let path1 = art_cache_path(url);
        let path2 = art_cache_path(url);
        assert_eq!(path1, path2);
        assert!(path1.to_string_lossy().contains("art"));
        assert_eq!(path1.extension().and_then(|e| e.to_str()), Some("img"));
    }

    #[test]
    fn test_art_disk_cache_fetch() {
        let url = "https://test.local/dummy_test_cover.png";
        let path = art_cache_path(url);
        let img = image::RgbaImage::new(4, 4);
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();

        let _ = std::fs::create_dir_all(path.parent().unwrap());
        std::fs::write(&path, &png_bytes).unwrap();

        let agent = ureq::Agent::new();
        let loaded = fetch(&agent, url);
        assert!(loaded.is_some());
        let _ = std::fs::remove_file(path);
    }
}
