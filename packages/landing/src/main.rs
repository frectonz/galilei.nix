use std::{ffi::CString, fs, io::IsTerminal, path::Path, time::Duration};

use serde_json::Value;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::Command,
    task::spawn_blocking,
    time::timeout,
};
use topcoat::{
    Result,
    router::{module_router, page},
    view::{View, component, view},
};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

const STYLE: &str = include_str!("style.css");
const LIBRARY: &str = "/data/media/library";
const DATA: &str = "/data";
const JELLYFIN: &str = "127.0.0.1:8096";
const QBITTORRENT: &str = "127.0.0.1:8080";
const VIDEO: &[&str] = &["mkv", "mp4", "avi", "webm", "m4v", "mov"];

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_ansi(std::io::stdout().is_terminal())
        .init();
    info!("starting landing");
    topcoat::start(module_router!().build()).await.unwrap();
}

fn domain() -> String {
    std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".into())
}

#[page]
async fn home() -> Result<impl View> {
    let (jellyfin, qbittorrent, system) = tokio::join!(jellyfin(), qbittorrent(), system());
    let notices = notices(&jellyfin, &qbittorrent, &system);
    debug!(?jellyfin, ?qbittorrent, ?system, "rendering home");
    let domain = domain();

    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(domain.as_str())</title>
                <style>(STYLE)</style>
            </head>
            <body>
                <main>
                    <header>
                        <h1>(domain.as_str())</h1>
                        <p class="meta">
                            "galilei · up " (system.uptime) " · load " (system.load)
                        </p>
                    </header>

                    <section class="cards">
                        jellyfin_card(domain: &domain, info: &jellyfin)
                        qbittorrent_card(domain: &domain, info: &qbittorrent)
                    </section>

                    <section class="stats">
                        gauge(label: "Disk", used: system.disk_used, total: system.disk_total)
                        gauge(label: "Memory", used: system.mem_used, total: system.mem_total)
                    </section>

                    <section class="notices">
                        <h2>"Notices"</h2>
                        if notices.is_empty() {
                            <p class="meta">"Nothing needs your attention."</p>
                        } else {
                            <ul>
                                for notice in notices {
                                    <li>(notice)</li>
                                }
                            </ul>
                        }
                    </section>
                </main>
            </body>
        </html>
    })
}

#[component]
async fn jellyfin_card(domain: &str, info: &Jellyfin) -> Result<impl View> {
    let href = format!("https://jellyfin.{domain}");
    let status = match &info.version {
        Some(version) => format!("Jellyfin {version}"),
        None => "not responding".to_string(),
    };
    Ok(view! {
        <a class="card" href=(href)>
            <h2>"Jellyfin"</h2>
            <p class="big">(plural(info.movies, "movie"))</p>
            <p>(plural(info.shows, "show")) " · " (plural(info.episodes, "episode"))</p>
            <p class="meta">(status)</p>
        </a>
    })
}

#[component]
async fn qbittorrent_card(domain: &str, info: &Qbittorrent) -> Result<impl View> {
    let href = format!("https://qbittorrent.{domain}");
    let status = if info.online {
        format!(
            "↓ {}/s · ↑ {}/s",
            bytes(info.down_speed),
            bytes(info.up_speed)
        )
    } else {
        "not responding".to_string()
    };
    Ok(view! {
        <a class="card" href=(href)>
            <h2>"qBittorrent"</h2>
            <p class="big">(info.downloading.to_string()) " downloading"</p>
            <p>(info.seeding.to_string()) " seeding · " (info.errored.to_string()) " errored"</p>
            <p class="meta">(status)</p>
        </a>
    })
}

#[component]
async fn gauge(label: &str, used: u64, total: u64) -> Result<impl View> {
    let pct = percent(used, total);
    let width = format!("width: {pct}%");
    let text = format!("{} of {}", bytes(used), bytes(total));
    Ok(view! {
        <div class="stat">
            <span>(label)</span>
            <div class="bar">
                <div class=(if pct >= 80 { "fill warn" } else { "fill" }) style=(width)></div>
            </div>
            <span>(text)</span>
        </div>
    })
}

fn notices(jellyfin: &Jellyfin, qbittorrent: &Qbittorrent, system: &System) -> Vec<String> {
    let mut out = Vec::new();
    let disk = percent(system.disk_used, system.disk_total);
    if disk >= 80 {
        out.push(format!(
            "Disk is {disk}% full, {} left.",
            bytes(system.disk_total - system.disk_used)
        ));
    }
    let mem = percent(system.mem_used, system.mem_total);
    if mem >= 90 {
        out.push(format!("Memory is {mem}% used."));
    }
    if !system.failed_units.is_empty() {
        out.push(format!("Failed units: {}.", system.failed_units.join(", ")));
    }
    if system.reboot_needed {
        out.push(
            "A reboot is pending: the running kernel is older than the current system.".into(),
        );
    }
    if jellyfin.version.is_none() {
        out.push("Jellyfin is not responding.".into());
    }
    if !qbittorrent.online {
        out.push("qBittorrent is not responding.".into());
    }
    if qbittorrent.errored > 0 {
        out.push(format!("{} torrents have errors.", qbittorrent.errored));
    }
    if qbittorrent.seeding > 0 {
        out.push(format!(
            "{} finished downloads are waiting to be moved into the library.",
            qbittorrent.seeding
        ));
    }
    out
}

#[derive(Debug, Default)]
struct Jellyfin {
    version: Option<String>,
    movies: usize,
    shows: usize,
    episodes: usize,
}

async fn jellyfin() -> Jellyfin {
    let version = get(JELLYFIN, "/System/Info/Public")
        .await
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .and_then(|v| v["Version"].as_str().map(str::to_string));
    if version.is_none() {
        warn!("jellyfin is not responding");
    }
    let (movies, shows, episodes) = spawn_blocking(library_counts).await.unwrap_or_default();
    Jellyfin {
        version,
        movies,
        shows,
        episodes,
    }
}

fn library_counts() -> (usize, usize, usize) {
    let movies = count_dirs(&Path::new(LIBRARY).join("movies"));
    let shows_dir = Path::new(LIBRARY).join("shows");
    let shows = count_dirs(&shows_dir);
    let episodes = count_videos(&shows_dir);
    (movies, shows, episodes)
}

fn count_dirs(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
        .unwrap_or(0)
}

fn count_videos(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| {
            let path = e.path();
            if path.is_dir() {
                count_videos(&path)
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                usize::from(VIDEO.contains(&ext.to_ascii_lowercase().as_str()))
            }
        })
        .sum()
}

#[derive(Debug, Default)]
struct Qbittorrent {
    online: bool,
    downloading: usize,
    seeding: usize,
    errored: usize,
    down_speed: u64,
    up_speed: u64,
}

async fn qbittorrent() -> Qbittorrent {
    let (torrents, transfer) = tokio::join!(
        get(QBITTORRENT, "/api/v2/torrents/info"),
        get(QBITTORRENT, "/api/v2/transfer/info")
    );
    let Some(torrents) = torrents.and_then(|b| serde_json::from_str::<Value>(&b).ok()) else {
        warn!("qbittorrent is not responding");
        return Qbittorrent::default();
    };
    let mut info = Qbittorrent {
        online: true,
        ..Default::default()
    };
    for torrent in torrents.as_array().into_iter().flatten() {
        match torrent["state"].as_str().unwrap_or("") {
            "downloading" | "metaDL" | "forcedMetaDL" | "stalledDL" | "queuedDL" | "forcedDL"
            | "checkingDL" | "allocating" => info.downloading += 1,
            "uploading" | "stalledUP" | "queuedUP" | "forcedUP" | "checkingUP" => info.seeding += 1,
            "error" | "missingFiles" => info.errored += 1,
            _ => {}
        }
    }
    if let Some(transfer) = transfer.and_then(|b| serde_json::from_str::<Value>(&b).ok()) {
        info.down_speed = transfer["dl_info_speed"].as_u64().unwrap_or(0);
        info.up_speed = transfer["up_info_speed"].as_u64().unwrap_or(0);
    }
    info
}

#[derive(Debug, Default)]
struct System {
    uptime: String,
    load: String,
    disk_used: u64,
    disk_total: u64,
    mem_used: u64,
    mem_total: u64,
    failed_units: Vec<String>,
    reboot_needed: bool,
}

async fn system() -> System {
    let failed_units = failed_units().await;
    let mut system = spawn_blocking(system_blocking).await.unwrap_or_default();
    system.failed_units = failed_units;
    system
}

fn system_blocking() -> System {
    let uptime_secs = fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    let load = fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "?".into());
    let (mem_total, mem_used) = meminfo();
    let (disk_total, disk_used) = statvfs(DATA);
    let reboot_needed = match (
        fs::read_link("/run/booted-system/kernel"),
        fs::read_link("/run/current-system/kernel"),
    ) {
        (Ok(booted), Ok(current)) => booted != current,
        _ => false,
    };
    System {
        uptime: duration(uptime_secs),
        load,
        disk_used,
        disk_total,
        mem_used,
        mem_total,
        failed_units: Vec::new(),
        reboot_needed,
    }
}

fn meminfo() -> (u64, u64) {
    let text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let field = |name: &str| {
        text.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
            .unwrap_or(0)
            * 1024
    };
    let total = field("MemTotal:");
    let available = field("MemAvailable:");
    (total, total.saturating_sub(available))
}

fn statvfs(path: &str) -> (u64, u64) {
    let Ok(path) = CString::new(path) else {
        return (0, 0);
    };
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: statvfs writes into the provided buffer and we only read it on success.
    let stat = unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) != 0 {
            warn!("statvfs failed for {}", path.to_string_lossy());
            return (0, 0);
        }
        stat.assume_init()
    };
    let frsize = stat.f_frsize as u64;
    let total = stat.f_blocks as u64 * frsize;
    let free = stat.f_bfree as u64 * frsize;
    (total, total.saturating_sub(free))
}

async fn failed_units() -> Vec<String> {
    let output = timeout(
        Duration::from_secs(3),
        Command::new("systemctl")
            .args(["--failed", "--plain", "--no-legend"])
            .output(),
    )
    .await;
    match output {
        Ok(Ok(output)) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            .map(str::to_string)
            .collect(),
        other => {
            warn!(?other, "could not list failed units");
            Vec::new()
        }
    }
}

async fn get(addr: &str, path: &str) -> Option<String> {
    match timeout(Duration::from_secs(2), request(addr, path)).await {
        Ok(Some(body)) => Some(body),
        Ok(None) => {
            warn!(addr, path, "request failed");
            None
        }
        Err(_) => {
            warn!(addr, path, "request timed out");
            None
        }
    }
}

async fn request(addr: &str, path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(addr).await.ok()?;
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).await.ok()?;

    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
            continue;
        };
        let head = String::from_utf8_lossy(&buf[..end]).to_ascii_lowercase();
        let body_len = buf.len() - (end + 4);
        if let Some(len) = header(&head, "content-length").and_then(|v| v.parse::<usize>().ok()) {
            if body_len >= len {
                break;
            }
        } else if header(&head, "transfer-encoding").is_some_and(|v| v.contains("chunked"))
            && buf.ends_with(b"0\r\n\r\n")
        {
            break;
        }
    }

    let text = String::from_utf8_lossy(&buf);
    let (head, body) = text.split_once("\r\n\r\n")?;
    let status = head.split_whitespace().nth(1)?;
    if status != "200" {
        warn!(addr, path, status, "unexpected response");
        return None;
    }
    let chunked = header(&head.to_ascii_lowercase(), "transfer-encoding")
        .is_some_and(|v| v.contains("chunked"));
    Some(if chunked {
        dechunk(body)
    } else {
        body.to_string()
    })
}

fn header<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    head.lines()
        .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))
        .map(str::trim)
}

fn dechunk(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some((size, after)) = rest.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size.trim(), 16) else {
            break;
        };
        if size == 0 || after.len() < size {
            break;
        }
        out.push_str(&after[..size]);
        rest = after[size..].trim_start_matches("\r\n");
    }
    out
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn percent(used: u64, total: u64) -> u64 {
    (used * 100).checked_div(total).unwrap_or(0)
}

fn bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn duration(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3600;
    let minutes = (secs % 3600) / 60;
    match (days, hours) {
        (0, 0) => format!("{minutes}m"),
        (0, _) => format!("{hours}h {minutes}m"),
        _ => format!("{days}d {hours}h"),
    }
}
