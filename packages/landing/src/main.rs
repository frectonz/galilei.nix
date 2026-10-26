use std::{
    fs,
    io::IsTerminal,
    path::Path,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use procfs::{Current, CurrentSI, KernelStats, LoadAverage, Meminfo, Uptime, net::dev_status};
use serde_json::Value;
use tokio::{
    process::Command,
    task::spawn_blocking,
    time::{sleep, timeout},
};
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    icon::{IconData, icon, iconify},
    router::{module_router, page},
    runtime::{RouterBuilderRuntimeExt, connected},
    tailwind,
    view::{Child, View, class, component, emit, live, view},
};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

iconify::include!("feather");
iconify::include!("brands");

const DOMAIN: &str = "frectonz.cloud";
const LIBRARY: &str = "/data/media/library";
const DATA: &str = "/data";
const JELLYFIN: &str = "127.0.0.1:8096";
const QBITTORRENT: &str = "127.0.0.1:8080";
const VIDEO: &[&str] = &["mkv", "mp4", "avi", "webm", "m4v", "mov"];
const REFRESH: Duration = Duration::from_secs(5);

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("http client")
});

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_ansi(std::io::stdout().is_terminal())
        .init();
    info!("starting landing");
    tokio::spawn(sample_rates());
    let assets = AssetBundle::load().expect("asset bundle next to the binary");
    let router = module_router!().assets(assets).runtime().build();
    topcoat::start(router).await.unwrap();
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(DOMAIN)</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::runtime::script()
            </head>
            <body
                class="min-h-screen bg-zinc-100 font-sans text-zinc-900 antialiased dark:bg-zinc-950 dark:text-zinc-100"
            >
                <main class="mx-auto max-w-3xl px-4 py-8">
                    (live! {
                        loop {
                            let (jellyfin, qbittorrent, system) = tokio::join!(
                                jellyfin(),
                                qbittorrent(),
                                system(),
                            );
                            let notices = notices(&jellyfin, &qbittorrent, &system);
                            debug!(?jellyfin, ?qbittorrent, ?system, "rendering dashboard");
                            let token = emit! {
                                dashboard(
                                    jellyfin: &jellyfin,
                                    qbittorrent: &qbittorrent,
                                    system: &system,
                                    notices: &notices
                                )
                            }?;
                            if !connected(cx) {
                                break Ok(token);
                            }
                            sleep(REFRESH).await;
                        }
                    })
                </main>
            </body>
        </html>
    })
}

#[component]
async fn dashboard(
    jellyfin: &Jellyfin,
    qbittorrent: &Qbittorrent,
    system: &System,
    notices: &[String],
) -> Result<impl View> {
    Ok(view! {
        <header>
            <h1 class="text-3xl font-bold">(DOMAIN)</h1>
            subtle(
                "galilei · up "
                (system.uptime.as_str())
                " · load "
                (system.load.as_str())
            )
        </header>

        <section class="my-6 grid gap-4 sm:grid-cols-2">
            jellyfin_card(info: jellyfin)
            qbittorrent_card(info: qbittorrent)
        </section>

        <section class="my-6 grid gap-3">
            gauge(
                data: feather::HARD_DRIVE,
                label: "Disk",
                used: system.disk_used,
                total: system.disk_total
            )
            gauge(
                data: feather::DATABASE,
                label: "Memory",
                used: system.mem_used,
                total: system.mem_total
            )
            swap_gauge(used: system.swap_used, total: system.swap_total)
            stat(
                data: feather::CPU,
                label: "CPU",
                bar(percent: system.cpu)
                <span>
                    (system.cpu)
                    "%"
                </span>
            )
            stat(
                data: feather::ACTIVITY,
                label: "Network",
                speed(down: system.rx_rate, up: system.tx_rate)
                <span class="text-zinc-500 dark:text-zinc-400">
                    (bytes(system.rx_total))
                    " in · "
                    (bytes(system.tx_total))
                    " out"
                </span>
            )
        </section>

        <section>
            card_title(data: feather::BELL, "Notices")
            if notices.is_empty() {
                subtle(
                    <span class="inline-flex items-center gap-1">
                        icon(data: feather::CHECK_CIRCLE)
                        "Nothing needs your attention."
                    </span>
                )
            } else {
                <ul class="list-disc pl-5 text-sm">
                    for notice in notices {
                        <li class="my-1">(notice)</li>
                    }
                </ul>
            }
        </section>
    })
}

#[component]
async fn jellyfin_card(info: &Jellyfin) -> Result<impl View> {
    let href = format!("https://jellyfin.{DOMAIN}");
    Ok(view! {
        card(
            href: &href,
            card_title(data: brands::JELLYFIN, "Jellyfin")
            big((plural(info.movies, "movie")))
            <p>
                (plural(info.shows, "show"))
                " · "
                (plural(info.episodes, "episode"))
            </p>
            subtle(
                match &info.version {
                    Some(version) => {
                        "Jellyfin "
                        (version)
                    }
                    None => {
                        "not responding"
                    }
                }
            )
        )
    })
}

#[component]
async fn qbittorrent_card(info: &Qbittorrent) -> Result<impl View> {
    let href = format!("https://qbittorrent.{DOMAIN}");
    Ok(view! {
        card(
            href: &href,
            card_title(data: brands::QBITTORRENT, "qBittorrent")
            big(
                (info.downloading)
                " downloading"
            )
            <p>
                (info.seeding)
                " seeding · "
                (info.errored)
                " errored"
            </p>
            subtle(
                if info.online {
                    speed(down: info.down_speed, up: info.up_speed)
                } else {
                    "not responding"
                }
            )
        )
    })
}

#[component]
async fn card(href: &str, child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <a
            class="block rounded-xl border border-zinc-200 bg-white p-5 hover:border-zinc-400 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-600"
            href=(href)
            rel="noreferrer"
        >
            (child)
        </a>
    })
}

#[component]
async fn card_title(data: IconData, child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <h2 class="mb-2 flex items-center gap-2 text-base font-semibold">
            icon(data: data)
            (child)
        </h2>
    })
}

#[component]
async fn big(child: Child<'_>) -> Result<impl View> {
    Ok(view! { <p class="text-3xl font-semibold">(child)</p> })
}

#[component]
async fn subtle(child: Child<'_>) -> Result<impl View> {
    Ok(view! { <p class="text-sm text-zinc-500 dark:text-zinc-400">(child)</p> })
}

#[component]
async fn speed(down: u64, up: u64) -> Result<impl View> {
    Ok(view! {
        <span class="inline-flex items-center gap-1">
            icon(data: feather::ARROW_DOWN)
            (bytes(down))
            "/s · "
            icon(data: feather::ARROW_UP)
            (bytes(up))
            "/s"
        </span>
    })
}

#[component]
async fn stat(data: IconData, label: &str, child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div
            class="grid grid-cols-[7rem_1fr_11rem] items-center gap-3 text-sm [&>:last-child]:text-right"
        >
            <span class="flex items-center gap-2">
                icon(data: data)
                (label)
            </span>
            (child)
        </div>
    })
}

#[component]
async fn bar(percent: u64) -> Result<impl View> {
    let width = format!("width: {percent}%");
    Ok(view! {
        <div class="h-2 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
            <div
                class=(class!(
                    "h-full",
                    "bg-red-600 dark:bg-red-400" if percent >= 80 else "bg-blue-600 dark:bg-blue-400",
                ))
                style=(width)
            ></div>
        </div>
    })
}

#[component]
async fn gauge(data: IconData, label: &str, used: u64, total: u64) -> Result<impl View> {
    let text = format!("{} of {}", bytes(used), bytes(total));
    Ok(view! {
        stat(
            data: data,
            label: label,
            bar(percent: percent(used, total))
            <span>(text)</span>
        )
    })
}

#[component]
async fn swap_gauge(used: u64, total: u64) -> Result<impl View> {
    let text = if total == 0 {
        "none".to_string()
    } else {
        format!("{} of {}", bytes(used), bytes(total))
    };
    Ok(view! {
        stat(
            data: feather::LAYERS,
            label: "Swap",
            bar(percent: percent(used, total))
            <span>(text)</span>
        )
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
    let Some(torrents) = torrents else {
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
    if let Some(transfer) = transfer {
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
    swap_used: u64,
    swap_total: u64,
    cpu: u64,
    rx_rate: u64,
    tx_rate: u64,
    rx_total: u64,
    tx_total: u64,
    failed_units: Vec<String>,
    reboot_needed: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct Rates {
    cpu: u64,
    rx_rate: u64,
    tx_rate: u64,
    rx_total: u64,
    tx_total: u64,
}

static RATES: Mutex<Rates> = Mutex::new(Rates {
    cpu: 0,
    rx_rate: 0,
    tx_rate: 0,
    rx_total: 0,
    tx_total: 0,
});

async fn sample_rates() {
    let mut prev = (Instant::now(), cpu_ticks(), net_bytes());
    loop {
        sleep(Duration::from_secs(2)).await;
        let now = (Instant::now(), cpu_ticks(), net_bytes());
        let elapsed = now.0.duration_since(prev.0).as_secs_f64().max(0.001);
        let (busy, total) = (
            now.1.0.saturating_sub(prev.1.0),
            now.1.1.saturating_sub(prev.1.1),
        );
        let rates = Rates {
            cpu: percent(busy, total),
            rx_rate: ((now.2.0.saturating_sub(prev.2.0)) as f64 / elapsed) as u64,
            tx_rate: ((now.2.1.saturating_sub(prev.2.1)) as f64 / elapsed) as u64,
            rx_total: now.2.0,
            tx_total: now.2.1,
        };
        *RATES.lock().unwrap_or_else(|e| e.into_inner()) = rates;
        prev = now;
    }
}

fn cpu_ticks() -> (u64, u64) {
    let Ok(stats) = KernelStats::current() else {
        warn!("could not read /proc/stat");
        return (0, 0);
    };
    let t = stats.total;
    let idle = t.idle + t.iowait.unwrap_or(0);
    let total = t.user
        + t.nice
        + t.system
        + idle
        + t.irq.unwrap_or(0)
        + t.softirq.unwrap_or(0)
        + t.steal.unwrap_or(0);
    (total - idle, total)
}

fn net_bytes() -> (u64, u64) {
    let Ok(devices) = dev_status() else {
        warn!("could not read /proc/net/dev");
        return (0, 0);
    };
    devices
        .values()
        .filter(|d| d.name != "lo")
        .fold((0, 0), |acc, d| {
            (acc.0 + d.recv_bytes, acc.1 + d.sent_bytes)
        })
}

async fn system() -> System {
    let failed_units = failed_units().await;
    let mut system = spawn_blocking(system_blocking).await.unwrap_or_default();
    let rates = *RATES.lock().unwrap_or_else(|e| e.into_inner());
    system.failed_units = failed_units;
    system.cpu = rates.cpu;
    system.rx_rate = rates.rx_rate;
    system.tx_rate = rates.tx_rate;
    system.rx_total = rates.rx_total;
    system.tx_total = rates.tx_total;
    system
}

fn system_blocking() -> System {
    let uptime_secs = Uptime::current().map(|u| u.uptime as u64).unwrap_or(0);
    let load = LoadAverage::current()
        .map(|l| format!("{:.2}", l.one))
        .unwrap_or_else(|_| "?".into());
    let mem = Meminfo::current().ok();
    let mem_total = mem.as_ref().map(|m| m.mem_total).unwrap_or(0);
    let mem_available = mem.as_ref().and_then(|m| m.mem_available).unwrap_or(0);
    let swap_total = mem.as_ref().map(|m| m.swap_total).unwrap_or(0);
    let swap_free = mem.as_ref().map(|m| m.swap_free).unwrap_or(0);
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
        mem_used: mem_total.saturating_sub(mem_available),
        mem_total,
        swap_used: swap_total.saturating_sub(swap_free),
        swap_total,
        reboot_needed,
        ..Default::default()
    }
}

fn statvfs(path: &str) -> (u64, u64) {
    let Ok(vfs) = rustix::fs::statvfs(path) else {
        warn!("statvfs failed for {path}");
        return (0, 0);
    };
    let total = vfs.f_blocks * vfs.f_frsize;
    let free = vfs.f_bfree * vfs.f_frsize;
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

async fn get(addr: &str, path: &str) -> Option<Value> {
    let url = format!("http://{addr}{path}");
    let response = match CLIENT
        .get(&url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(response) => response,
        Err(error) => {
            warn!(url, %error, "request failed");
            return None;
        }
    };
    match response.json().await {
        Ok(value) => Some(value),
        Err(error) => {
            warn!(url, %error, "invalid response");
            None
        }
    }
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
