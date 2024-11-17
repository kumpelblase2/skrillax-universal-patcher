use clap::Parser;
use pk2::{Pk2, SyncLock};
use speedy::{Readable, Writable};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io;
use tokio::net::TcpSocket;
use tokio_util::sync::CancellationToken;

const BLOWFISH_KEY: &str = "169841";

#[derive(Writable, Readable)]
struct Gateway {
    ip: String,
    null_byte: u8,
}

#[derive(Readable, Writable)]
struct Division {
    name: String,
    null_byte: u8,
    count: u8,
    #[speedy(length = count)]
    gateways: Vec<Gateway>,
}

#[derive(Readable, Writable)]
struct DivisionInfo {
    content: u8,
    count: u8,
    #[speedy(length = count)]
    divisions: Vec<Division>,
}

fn create_local_division_info(original: &DivisionInfo) -> DivisionInfo {
    DivisionInfo {
        content: original.content,
        count: 1,
        divisions: vec![Division {
            name: "DIV01".to_string(),
            null_byte: 0,
            count: 1,
            gateways: vec![Gateway {
                ip: "127.0.0.1".to_string(),
                null_byte: 0,
            }],
        }],
    }
}

fn create_port_from_patch(patch: u32) -> u32 {
    32000 + patch
}

#[derive(Debug, Parser)]
#[clap(name = "skrillax-client-patcher", version = env!("CARGO_PKG_VERSION"), author = "kumpelblase2", about = "Patch Silkroad to any version using a specialized patch server.")]
struct ClientParserArgs {
    #[clap(short, long, help = "The patch server to use, defaults to localhost")]
    server: Option<String>,
    #[clap(
        short,
        long,
        required = true,
        help = "The target patch version to patch to, e.g. 569."
    )]
    patch: u16,

    #[clap(
        help = "The directory of Silkroad Online, falls back to the current working directory."
    )]
    silkroad_dir: Option<PathBuf>,
}

fn main() {
    let args = ClientParserArgs::parse();

    let patch: u16 = args.patch;
    let base_path = args
        .silkroad_dir
        .unwrap_or_else(|| env::current_dir().unwrap());
    let server_addr = args.server.unwrap_or("127.0.0.1".to_string());
    println!("[*] Starting patching to version {patch}...");
    let media_path = base_path.join("Media.pk2");
    let original_division_info = load_division_info(&media_path);
    let new_division_info = create_local_division_info(&original_division_info);
    write_division_info(&new_division_info, &media_path).unwrap();
    println!("[*] Patched DivisionInfo to redirect to local proxy.");
    let done_token = CancellationToken::new();
    let thread_token = done_token.clone();
    std::thread::spawn(move || {
        run_proxy(
            thread_token,
            server_addr,
            create_port_from_patch(patch.into()),
        );
    });
    run_silkroad(base_path.join("Silkroad.exe"));
    done_token.cancel();
    write_division_info(&original_division_info, media_path).unwrap();
    println!("[*] Original DivisionInfo restored.");
    println!("[*] Patching complete, client should be on version {patch} now!");
}

fn run_silkroad<T: AsRef<OsStr>>(path: T) {
    println!("[*] Running Silkroad Online Launcher to start patching...");
    let mut silkroad = Command::new(path).spawn().unwrap();
    silkroad.wait().unwrap();
    println!("[*] Silkroad Online Launcher done.");
}

fn run_proxy(cancellation_token: CancellationToken, patcher_ip: String, port: u32) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async move {
        loop {
            let proxy_client = TcpSocket::new_v4().unwrap();
            proxy_client
                .bind("127.0.0.1:15779".parse().unwrap())
                .unwrap();
            println!("[*] Started proxy on port 15779, waiting for client connection...");
            let client_listener = proxy_client.listen(1).unwrap();
            let maybe_client = tokio::select! {
                accept = client_listener.accept() => Some(accept.unwrap().0),
                _ = cancellation_token.cancelled() => None,
            };
            let Some(client) = maybe_client else { return };
            println!("[*] Got client, connecting to patcher...");
            let (mut client_read, mut client_write) = client.into_split();
            let proxy_server = TcpSocket::new_v4().unwrap();
            let server = proxy_server
                .connect(format!("{}:{}", patcher_ip, port).parse().unwrap())
                .await
                .unwrap();
            println!("[*] Connected to patcher, waiting for patching to finish...");
            let (mut server_read, mut server_write) = server.into_split();
            let client_to_server =
                tokio::spawn(async move { io::copy(&mut client_read, &mut server_write).await });
            let server_to_client =
                tokio::spawn(async move { io::copy(&mut server_read, &mut client_write).await });

            let _ = futures::future::join(client_to_server, server_to_client).await;
            println!("[*] Connection between patcher closed.");
        }
    });
}

fn load_division_info<T: AsRef<Path>>(path: T) -> DivisionInfo {
    let pk2: Pk2<File, SyncLock> = Pk2::open(path, BLOWFISH_KEY).unwrap();
    let mut f = pk2.open_file("/DIVISIONINFO.TXT").unwrap();
    DivisionInfo::read_from_stream_buffered(&mut f).unwrap()
}

fn write_division_info<T: AsRef<Path>>(division_info: &DivisionInfo, path: T) -> io::Result<()> {
    let mut pk2: Pk2<File, SyncLock> = Pk2::open(path, BLOWFISH_KEY).unwrap();
    let mut f = pk2.open_file_mut("/DIVISIONINFO.TXT")?;
    division_info.write_to_stream(&mut f)?;
    Ok(())
}
