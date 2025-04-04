#![forbid(unsafe_code)]

use anyhow::{anyhow, Error};
use chrono::{Duration, Local, NaiveDate};
use clap::Parser;
use filebuffer::FileBuffer;
use indicatif::{ProgressBar, ProgressStyle};
use ring::digest;
use std::collections::HashSet;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, remove_file, File};
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use toml::Value;
use url::Url;

const RELEASE_CHANNELS: [&str; 3] = ["stable", "beta", "nightly"];

// rustc --print target-list | awk '{print "    \"" $1 "\","}'
const TARGETS: [&str; 271] = [
    "aarch64-apple-darwin",
    "aarch64-apple-ios",
    "aarch64-apple-ios-macabi",
    "aarch64-apple-ios-sim",
    "aarch64-apple-tvos",
    "aarch64-apple-tvos-sim",
    "aarch64-apple-visionos",
    "aarch64-apple-visionos-sim",
    "aarch64-apple-watchos",
    "aarch64-apple-watchos-sim",
    "aarch64-kmc-solid_asp3",
    "aarch64-linux-android",
    "aarch64-nintendo-switch-freestanding",
    "aarch64-pc-windows-gnullvm",
    "aarch64-pc-windows-msvc",
    "aarch64-unknown-freebsd",
    "aarch64-unknown-fuchsia",
    "aarch64-unknown-hermit",
    "aarch64-unknown-illumos",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu_ilp32",
    "aarch64-unknown-linux-musl",
    "aarch64-unknown-linux-ohos",
    "aarch64-unknown-netbsd",
    "aarch64-unknown-none",
    "aarch64-unknown-none-softfloat",
    "aarch64-unknown-nto-qnx700",
    "aarch64-unknown-nto-qnx710",
    "aarch64-unknown-openbsd",
    "aarch64-unknown-redox",
    "aarch64-unknown-teeos",
    "aarch64-unknown-trusty",
    "aarch64-unknown-uefi",
    "aarch64-uwp-windows-msvc",
    "aarch64-wrs-vxworks",
    "aarch64_be-unknown-linux-gnu",
    "aarch64_be-unknown-linux-gnu_ilp32",
    "aarch64_be-unknown-netbsd",
    "arm-linux-androideabi",
    "arm-unknown-linux-gnueabi",
    "arm-unknown-linux-gnueabihf",
    "arm-unknown-linux-musleabi",
    "arm-unknown-linux-musleabihf",
    "arm64_32-apple-watchos",
    "arm64e-apple-darwin",
    "arm64e-apple-ios",
    "arm64e-apple-tvos",
    "arm64ec-pc-windows-msvc",
    "armeb-unknown-linux-gnueabi",
    "armebv7r-none-eabi",
    "armebv7r-none-eabihf",
    "armv4t-none-eabi",
    "armv4t-unknown-linux-gnueabi",
    "armv5te-none-eabi",
    "armv5te-unknown-linux-gnueabi",
    "armv5te-unknown-linux-musleabi",
    "armv5te-unknown-linux-uclibceabi",
    "armv6-unknown-freebsd",
    "armv6-unknown-netbsd-eabihf",
    "armv6k-nintendo-3ds",
    "armv7-linux-androideabi",
    "armv7-rtems-eabihf",
    "armv7-sony-vita-newlibeabihf",
    "armv7-unknown-freebsd",
    "armv7-unknown-linux-gnueabi",
    "armv7-unknown-linux-gnueabihf",
    "armv7-unknown-linux-musleabi",
    "armv7-unknown-linux-musleabihf",
    "armv7-unknown-linux-ohos",
    "armv7-unknown-linux-uclibceabi",
    "armv7-unknown-linux-uclibceabihf",
    "armv7-unknown-netbsd-eabihf",
    "armv7-unknown-trusty",
    "armv7-wrs-vxworks-eabihf",
    "armv7a-kmc-solid_asp3-eabi",
    "armv7a-kmc-solid_asp3-eabihf",
    "armv7a-none-eabi",
    "armv7a-none-eabihf",
    "armv7k-apple-watchos",
    "armv7r-none-eabi",
    "armv7r-none-eabihf",
    "armv7s-apple-ios",
    "armv8r-none-eabihf",
    "avr-unknown-gnu-atmega328",
    "bpfeb-unknown-none",
    "bpfel-unknown-none",
    "csky-unknown-linux-gnuabiv2",
    "csky-unknown-linux-gnuabiv2hf",
    "hexagon-unknown-linux-musl",
    "hexagon-unknown-none-elf",
    "i386-apple-ios",
    "i586-pc-nto-qnx700",
    "i586-pc-windows-msvc",
    "i586-unknown-linux-gnu",
    "i586-unknown-linux-musl",
    "i586-unknown-netbsd",
    "i686-apple-darwin",
    "i686-linux-android",
    "i686-pc-windows-gnu",
    "i686-pc-windows-gnullvm",
    "i686-pc-windows-msvc",
    "i686-unknown-freebsd",
    "i686-unknown-haiku",
    "i686-unknown-hurd-gnu",
    "i686-unknown-linux-gnu",
    "i686-unknown-linux-musl",
    "i686-unknown-netbsd",
    "i686-unknown-openbsd",
    "i686-unknown-redox",
    "i686-unknown-uefi",
    "i686-uwp-windows-gnu",
    "i686-uwp-windows-msvc",
    "i686-win7-windows-msvc",
    "i686-wrs-vxworks",
    "loongarch64-unknown-linux-gnu",
    "loongarch64-unknown-linux-musl",
    "loongarch64-unknown-linux-ohos",
    "loongarch64-unknown-none",
    "loongarch64-unknown-none-softfloat",
    "m68k-unknown-linux-gnu",
    "mips-unknown-linux-gnu",
    "mips-unknown-linux-musl",
    "mips-unknown-linux-uclibc",
    "mips64-openwrt-linux-musl",
    "mips64-unknown-linux-gnuabi64",
    "mips64-unknown-linux-muslabi64",
    "mips64el-unknown-linux-gnuabi64",
    "mips64el-unknown-linux-muslabi64",
    "mipsel-sony-psp",
    "mipsel-sony-psx",
    "mipsel-unknown-linux-gnu",
    "mipsel-unknown-linux-musl",
    "mipsel-unknown-linux-uclibc",
    "mipsel-unknown-netbsd",
    "mipsel-unknown-none",
    "mipsisa32r6-unknown-linux-gnu",
    "mipsisa32r6el-unknown-linux-gnu",
    "mipsisa64r6-unknown-linux-gnuabi64",
    "mipsisa64r6el-unknown-linux-gnuabi64",
    "msp430-none-elf",
    "nvptx64-nvidia-cuda",
    "powerpc-unknown-freebsd",
    "powerpc-unknown-linux-gnu",
    "powerpc-unknown-linux-gnuspe",
    "powerpc-unknown-linux-musl",
    "powerpc-unknown-linux-muslspe",
    "powerpc-unknown-netbsd",
    "powerpc-unknown-openbsd",
    "powerpc-wrs-vxworks",
    "powerpc-wrs-vxworks-spe",
    "powerpc64-ibm-aix",
    "powerpc64-unknown-freebsd",
    "powerpc64-unknown-linux-gnu",
    "powerpc64-unknown-linux-musl",
    "powerpc64-unknown-openbsd",
    "powerpc64-wrs-vxworks",
    "powerpc64le-unknown-freebsd",
    "powerpc64le-unknown-linux-gnu",
    "powerpc64le-unknown-linux-musl",
    "riscv32-wrs-vxworks",
    "riscv32e-unknown-none-elf",
    "riscv32em-unknown-none-elf",
    "riscv32emc-unknown-none-elf",
    "riscv32gc-unknown-linux-gnu",
    "riscv32gc-unknown-linux-musl",
    "riscv32i-unknown-none-elf",
    "riscv32im-risc0-zkvm-elf",
    "riscv32im-unknown-none-elf",
    "riscv32ima-unknown-none-elf",
    "riscv32imac-esp-espidf",
    "riscv32imac-unknown-none-elf",
    "riscv32imac-unknown-nuttx-elf",
    "riscv32imac-unknown-xous-elf",
    "riscv32imafc-esp-espidf",
    "riscv32imafc-unknown-none-elf",
    "riscv32imafc-unknown-nuttx-elf",
    "riscv32imc-esp-espidf",
    "riscv32imc-unknown-none-elf",
    "riscv32imc-unknown-nuttx-elf",
    "riscv64-linux-android",
    "riscv64-wrs-vxworks",
    "riscv64gc-unknown-freebsd",
    "riscv64gc-unknown-fuchsia",
    "riscv64gc-unknown-hermit",
    "riscv64gc-unknown-linux-gnu",
    "riscv64gc-unknown-linux-musl",
    "riscv64gc-unknown-netbsd",
    "riscv64gc-unknown-none-elf",
    "riscv64gc-unknown-nuttx-elf",
    "riscv64gc-unknown-openbsd",
    "riscv64imac-unknown-none-elf",
    "riscv64imac-unknown-nuttx-elf",
    "s390x-unknown-linux-gnu",
    "s390x-unknown-linux-musl",
    "sparc-unknown-linux-gnu",
    "sparc-unknown-none-elf",
    "sparc64-unknown-linux-gnu",
    "sparc64-unknown-netbsd",
    "sparc64-unknown-openbsd",
    "sparcv9-sun-solaris",
    "thumbv4t-none-eabi",
    "thumbv5te-none-eabi",
    "thumbv6m-none-eabi",
    "thumbv6m-nuttx-eabi",
    "thumbv7a-pc-windows-msvc",
    "thumbv7a-uwp-windows-msvc",
    "thumbv7em-none-eabi",
    "thumbv7em-none-eabihf",
    "thumbv7em-nuttx-eabi",
    "thumbv7em-nuttx-eabihf",
    "thumbv7m-none-eabi",
    "thumbv7m-nuttx-eabi",
    "thumbv7neon-linux-androideabi",
    "thumbv7neon-unknown-linux-gnueabihf",
    "thumbv7neon-unknown-linux-musleabihf",
    "thumbv8m.base-none-eabi",
    "thumbv8m.base-nuttx-eabi",
    "thumbv8m.main-none-eabi",
    "thumbv8m.main-none-eabihf",
    "thumbv8m.main-nuttx-eabi",
    "thumbv8m.main-nuttx-eabihf",
    "wasm32-unknown-emscripten",
    "wasm32-unknown-unknown",
    "wasm32-wasip1",
    "wasm32-wasip1-threads",
    "wasm32-wasip2",
    "wasm32v1-none",
    "wasm64-unknown-unknown",
    "x86_64-apple-darwin",
    "x86_64-apple-ios",
    "x86_64-apple-ios-macabi",
    "x86_64-apple-tvos",
    "x86_64-apple-watchos-sim",
    "x86_64-fortanix-unknown-sgx",
    "x86_64-linux-android",
    "x86_64-pc-nto-qnx710",
    "x86_64-pc-solaris",
    "x86_64-pc-windows-gnu",
    "x86_64-pc-windows-gnullvm",
    "x86_64-pc-windows-msvc",
    "x86_64-unikraft-linux-musl",
    "x86_64-unknown-dragonfly",
    "x86_64-unknown-freebsd",
    "x86_64-unknown-fuchsia",
    "x86_64-unknown-haiku",
    "x86_64-unknown-hermit",
    "x86_64-unknown-hurd-gnu",
    "x86_64-unknown-illumos",
    "x86_64-unknown-l4re-uclibc",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnux32",
    "x86_64-unknown-linux-musl",
    "x86_64-unknown-linux-none",
    "x86_64-unknown-linux-ohos",
    "x86_64-unknown-netbsd",
    "x86_64-unknown-none",
    "x86_64-unknown-openbsd",
    "x86_64-unknown-redox",
    "x86_64-unknown-trusty",
    "x86_64-unknown-uefi",
    "x86_64-uwp-windows-gnu",
    "x86_64-uwp-windows-msvc",
    "x86_64-win7-windows-msvc",
    "x86_64-wrs-vxworks",
    "x86_64h-apple-darwin",
    "xtensa-esp32-espidf",
    "xtensa-esp32-none-elf",
    "xtensa-esp32s2-espidf",
    "xtensa-esp32s2-none-elf",
    "xtensa-esp32s3-espidf",
    "xtensa-esp32s3-none-elf",
];

const DEFAULT_UPSTREAM_URL: &str = "https://static.rust-lang.org/";

const MAX_RETRIES: i32 = 3;

fn file_sha256(file_path: &Path) -> Option<String> {
    let file = Path::new(file_path);
    if file.exists() {
        let buffer = FileBuffer::open(&file).unwrap();
        Some(hex::encode(digest::digest(&digest::SHA256, &buffer)))
    } else {
        None
    }
}

fn download(upstream_url: &str, dir: &str, path: &str) -> Result<PathBuf, Error> {
    let manifest = format!("{}{}", upstream_url, path);
    let mut response;
    let mirror = Path::new(dir);
    let file_path = mirror.join(&path);
    create_dir_all(file_path.parent().unwrap())?;
    let mut dest = File::create(file_path)?;
    let mut attempts = 0;

    'outer: loop {
        attempts += 1;
        match reqwest::blocking::get(&manifest) {
            Ok(res) => {
                response = res;
            }
            Err(e) => {
                if attempts >= MAX_RETRIES {
                    return Err(anyhow!(
                        "Failed to download after {} attempts: {}",
                        MAX_RETRIES,
                        e.to_string()
                    ));
                }
                println!("Attempt {} failed: {}. Retrying...", attempts, e);
                continue 'outer;
            }
        }

        println!("File /{} downloading", path);
        let length = match response.content_length() {
            None => return Err(anyhow!("Not found")),
            Some(l) => l,
        };
        let pb = ProgressBar::new(length);
        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} (ETA {eta_precise})")?
        .progress_chars("#>-"));

        let mut buffer = [0u8; 4096];
        let mut read = 0;

        while read < length {
            match response.read(&mut buffer) {
                Ok(len) => {
                    dest.write_all(&buffer[..len])?;
                    read += len as u64;
                    pb.set_position(read);
                }
                Err(e) => {
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow!(
                            "Failed to read response after {} attempts: {}",
                            MAX_RETRIES,
                            e.to_string()
                        ));
                    }
                    println!(
                        "Attempt {} to read response failed: {}. Retrying...",
                        attempts, e
                    );
                    dest.rewind()?;
                    pb.finish_and_clear();
                    continue 'outer;
                }
            }
        }

        pb.finish_and_clear();
        println!("File /{} downloaded", path);
        break;
    }

    return Ok(mirror.join(path));
}

#[derive(Parser)]
#[command(
    version,
    about = "Make a mirror for rustup",
    author = "Jiajie Chen <c@jia.je>"
)]
struct Cli {
    /// Where to store original manifest
    #[arg(short, long, default_value = "./orig")]
    orig: String,

    /// Where to store mirror files
    #[arg(short, long, default_value = "./mirror")]
    mirror: String,

    /// Where mirror is served
    #[arg(short, long, default_value = "http://127.0.0.1:8000")]
    url: String,

    /// Keep how many days of nightly toolchains, e.g. 365
    #[arg(short, long)]
    gc: Option<i64>,

    /// Which release channel(s) to mirror, e.g. stable,nightly
    #[arg(short, long, value_delimiter = ',', default_values_t = RELEASE_CHANNELS.map(String::from))]
    channels: Vec<String>,

    /// Which targets to mirror, e.g. x86_64-unknown-linux-gnu,x86_64-apple-darwin
    #[arg(short, long, value_delimiter = ',', default_values_t = TARGETS.map(String::from))]
    targets: Vec<String>,

    /// Upstream url to sync from
    #[arg(short = 'U', long, default_value_t = DEFAULT_UPSTREAM_URL.to_string())]
    upstream_url: String,
}

fn main() {
    let args = Cli::parse();

    let orig_path = &args.orig;
    let mirror_path = &args.mirror;
    let mirror_url = &args.url;
    let upstream_url = &args.upstream_url;

    let parsed_gc_days = args.gc.map(|parsed_days| {
        let mut day = Local::now().date_naive();
        day -= Duration::days(parsed_days);
        println!("Nightly before {} will be deleted", day);
        day
    });

    let channels = args.channels;
    let filter_targets = args
        .targets
        .iter()
        .collect::<std::collections::HashSet<_>>();

    let mut all_targets = HashSet::new();

    // All referenced files
    let mut referenced = HashSet::new();

    // Fetch rust components
    for channel in channels.iter() {
        let name = format!("dist/channel-rust-{}.toml", channel);
        let file_path = download(upstream_url, orig_path, &name).unwrap();
        let sha256_name = format!("dist/channel-rust-{}.toml.sha256", channel);
        let sha256_file_path = download(upstream_url, orig_path, &sha256_name).unwrap();

        let mut file = File::open(file_path.clone()).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let mut sha256_file = File::open(sha256_file_path.clone()).unwrap();
        let mut sha256_data = String::new();
        sha256_file.read_to_string(&mut sha256_data).unwrap();
        assert_eq!(
            file_sha256(file_path.as_path()).unwrap(),
            &sha256_data[..64]
        );

        let mut value = data.parse::<Value>().unwrap();
        assert_eq!(value["manifest-version"].as_str(), Some("2"));
        println!(
            "Channel {} date {}",
            channel,
            value["date"].as_str().unwrap()
        );

        let pkgs = value["pkg"].as_table_mut().unwrap();
        let keys: Vec<String> = pkgs.keys().cloned().collect();
        for pkg_name in keys {
            let pkg = pkgs.get_mut(&pkg_name).unwrap().as_table_mut().unwrap();
            let pkg_targets = pkg.get_mut("target").unwrap().as_table_mut().unwrap();
            for (target, pkg_target) in pkg_targets {
                let pkg_target = pkg_target.as_table_mut().unwrap();

                // if we don't want to download this target
                // set available to false and do not download
                // but we will keep this table in the toml, which is required for newer version of
                // rustup
                if !(filter_targets.contains(target) || *target == "*") {
                    *pkg_target.get_mut("available").unwrap() = toml::Value::Boolean(false);
                    continue;
                }

                if pkg_target["available"].as_bool().unwrap() {
                    all_targets.insert(target.clone());

                    let prefixes = ["", "xz_"];
                    for prefix in prefixes.iter() {
                        let url =
                            Url::parse(pkg_target[&format!("{}url", prefix)].as_str().unwrap())
                                .unwrap();
                        let mirror = Path::new(mirror_path);
                        let file_name = url.path().replace("%20", " ");
                        let file = mirror.join(&file_name[1..]);

                        referenced.insert(normalize_path(&file));

                        let hash_file = mirror.join(format!("{}.sha256", &file_name[1..]));
                        let hash_file_cont =
                            File::open(hash_file.clone()).ok().and_then(|mut f| {
                                let mut cont = String::new();
                                f.read_to_string(&mut cont).ok().map(|_| cont)
                            });

                        let hash_file_missing = hash_file_cont.is_none();
                        let mut hash_file_cont =
                            hash_file_cont.or_else(|| file_sha256(file.as_path()));

                        let chksum_upstream =
                            pkg_target[&format!("{}hash", prefix)].as_str().unwrap();

                        let need_download = match hash_file_cont {
                            Some(ref chksum) => chksum_upstream != chksum,
                            None => true,
                        };

                        if need_download {
                            let mut attempts = 0;
                            loop {
                                attempts += 1;
                                download(upstream_url, mirror_path, &file_name[1..]).unwrap();
                                hash_file_cont = file_sha256(file.as_path());
                                if Some(chksum_upstream) == hash_file_cont.as_deref() {
                                    break;
                                }
                                if attempts >= MAX_RETRIES {
                                    panic!(
                                        "Failed to pass checksum after {} attempts",
                                        MAX_RETRIES
                                    );
                                }
                                println!("Checksum attempt {} failed. Retrying...", attempts);
                            }
                            // assert_eq!(Some(chksum_upstream), hash_file_cont.as_deref());
                        } else {
                            println!("File {} already downloaded, skipping", file_name);
                        }

                        if need_download || hash_file_missing {
                            File::create(hash_file)
                                .unwrap()
                                .write_all(hash_file_cont.unwrap().as_bytes())
                                .unwrap();
                            println!("Writing checksum for file {}", file_name);
                        }

                        pkg_target.insert(
                            format!("{}url", prefix),
                            Value::String(format!("{}{}", mirror_url, file_name)),
                        );
                    }
                }
            }
        }

        let output = toml::to_string(&value).unwrap();
        let path = Path::new(mirror_path).join(&name);
        create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = File::create(path.clone()).unwrap();
        println!("Producing /{}", name);
        file.write_all(output.as_bytes()).unwrap();

        let sha256_new_file = file_sha256(&path).unwrap();
        let sha256_new_file_path = Path::new(mirror_path).join(&sha256_name);
        let mut file = File::create(sha256_new_file_path.clone()).unwrap();
        println!("Producing /{}", sha256_name);
        file.write_all(format!("{}  channel-rust-{}.toml", sha256_new_file, channel).as_bytes())
            .unwrap();

        let date = value["date"].as_str().unwrap();

        let alt_name = format!("dist/{}/channel-rust-{}.toml", date, channel);
        let alt_path = Path::new(mirror_path).join(&alt_name);
        create_dir_all(alt_path.parent().unwrap()).unwrap();
        copy(path, alt_path).unwrap();
        println!("Producing /{}", alt_name);

        let alt_sha256_new_file_name =
            format!("dist/{}/channel-rust-{}.toml.sha256", date, channel);
        let alt_sha256_new_file_path = Path::new(mirror_path).join(&alt_sha256_new_file_name);
        copy(sha256_new_file_path, alt_sha256_new_file_path).unwrap();
        println!("Producing /{}", alt_sha256_new_file_name);
    }

    // Fetch latest binary of rustup
    println!("Downloading latest binary of rustup...");
    for target in &all_targets {
        if target == "*" {
            continue;
        }

        let is_windows = target.contains("windows");

        let ext = if is_windows { ".exe" } else { "" };

        if download(
            upstream_url,
            mirror_path,
            &format!("rustup/dist/{}/rustup-init{}", target, ext),
        )
        .is_err()
        {
            println!("Failed to fetch rustup-init for target {}, ignored", target);
        }
    }

    // Fetch rustup self update
    println!("Downloading rustup self update manifest...");
    let self_update_manifest_path =
        download(upstream_url, orig_path, "rustup/release-stable.toml").unwrap();

    let mut self_update_manifest = File::open(self_update_manifest_path.clone()).unwrap();
    let mut self_update_manifest_data = String::new();
    self_update_manifest
        .read_to_string(&mut self_update_manifest_data)
        .unwrap();

    let self_update_manifest_val = self_update_manifest_data.parse::<Value>().unwrap();
    assert_eq!(
        self_update_manifest_val["schema-version"].as_str(),
        Some("1")
    );

    let self_version = self_update_manifest_val["version"].as_str().unwrap();

    for target in all_targets {
        if target == "*" {
            continue;
        }

        let is_windows = target.contains("windows");

        let ext = if is_windows { ".exe" } else { "" };

        if download(
            upstream_url,
            mirror_path,
            &format!(
                "rustup/archive/{}/{}/rustup-init{}",
                self_version, target, ext
            ),
        )
        .is_err()
        {
            println!("Failed to fetch rustup-init for target {}, ignored", target);
        }
    }

    copy(
        self_update_manifest_path,
        Path::new(mirror_path).join("rustup/release-stable.toml"),
    )
    .unwrap();

    // Garbage collect old nightly builds, and unreferenced stable/beta builds
    for date_dir in read_dir(Path::new(mirror_path).join("dist")).expect("Unable to read dist dir")
    {
        let date_dir = date_dir.unwrap();
        if !date_dir.file_type().unwrap().is_dir() {
            // Is metadata
            continue;
        }

        let clear_nightly = if let Some(parsed_gc_days) = parsed_gc_days {
            let dir_name = date_dir.file_name().into_string().unwrap();
            let parsed_dir_name = NaiveDate::parse_from_str(&dir_name, "%Y-%m-%d").unwrap();
            parsed_dir_name < parsed_gc_days
        } else {
            false
        };

        // Is there anyone left?
        let mut perserve_dir = false;

        for file in read_dir(date_dir.path()).expect("inner dir") {
            let file = file.unwrap();
            let fname = file.file_name();
            let fname = fname.to_string_lossy();
            if fname.ends_with(".sha256") {
                // Is an hash, will be deleted alongside the hashed file
                continue;
            }

            let canonicalized = file.path().canonicalize().unwrap();
            let normalized = normalize_path(&file.path());

            // Filter referenced artifacts. Manifests will never be referenced
            let to_be_deleted = if referenced.contains(&normalized) {
                false
            } else if fname.find("nightly").is_some() {
                // Is nightly artifact or manifest
                clear_nightly
            } else {
                // Is stable/beta artifact or manifest, delete by default
                true
            };

            if to_be_deleted {
                // Delete artifact / manifest and its corresponding hash
                println!("Deleting file {}[.sha256]", canonicalized.display());
                remove_file(&canonicalized).unwrap();
                // Ignore error if the hash is not deleted (e.g. there is no hash present)
                let mut canonicalized = canonicalized;
                canonicalized.set_file_name((fname + ".sha256").as_ref());
                let _ = remove_file(canonicalized);
            } else {
                perserve_dir = true;
            }
        }

        if !perserve_dir {
            println!(
                "No useful file left in dir {}, removing the entire directory.",
                date_dir.path().display()
            );
            remove_dir_all(date_dir.path()).unwrap();
        }
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
