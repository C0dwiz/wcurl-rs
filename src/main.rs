use std::env;
use std::process::{Command, exit};

const VERSION: &str = "2025.11.09-rust";
const PROGRAM_NAME: &str = "wcurl";

struct Config {
    curl_options: Vec<String>,
    urls: Vec<String>,
    output_path: Option<String>,
    has_user_set_output: bool,
    decode_filename: bool,
    dry_run: bool,
}

impl Config {
    fn new() -> Self {
        Config {
            curl_options: Vec::new(),
            urls: Vec::new(),
            output_path: None,
            has_user_set_output: false,
            decode_filename: true,
            dry_run: false,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    match parse_args(args) {
        Ok(config) => {
            if let Err(e) = exec_curl(config) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<Config, String> {
    let mut config = Config::new();
    let mut i = 1;
    let mut reading_urls = false;

    while i < args.len() {
        let arg = &args[i];

        if reading_urls {
            let encoded_url = encode_whitespace(arg);
            config.urls.push(encoded_url);
            i += 1;
            continue;
        }

        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                exit(0);
            }
            "-V" | "--version" => {
                println!("{}", VERSION);
                exit(0);
            }
            "--dry-run" => {
                config.dry_run = true;
            }
            "--no-decode-filename" => {
                config.decode_filename = false;
            }
            "--curl-options" => {
                i += 1;
                if i >= args.len() {
                    return Err("--curl-options requires an argument".to_string());
                }
                config.curl_options.push(args[i].clone());
            }
            "-o" | "-O" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("{} requires an argument", arg));
                }
                config.has_user_set_output = true;
                config.output_path = Some(args[i].clone());
            }
            "--" => {
                reading_urls = true;
            }
            _ => {
                if arg.starts_with("--curl-options=") {
                    let opt = arg.strip_prefix("--curl-options=").unwrap();
                    config.curl_options.push(opt.to_string());
                } else if arg.starts_with("--output=") {
                    let opt = arg.strip_prefix("--output=").unwrap();
                    config.has_user_set_output = true;
                    config.output_path = Some(opt.to_string());
                } else if arg.starts_with("-o") || arg.starts_with("-O") {
                    let opt = &arg[2..];
                    config.has_user_set_output = true;
                    config.output_path = Some(opt.to_string());
                } else if arg.starts_with("-") {
                    return Err(format!("Unknown option: '{}'", arg));
                } else {
                    let encoded_url = encode_whitespace(arg);
                    config.urls.push(encoded_url);
                }
            }
        }
        i += 1;
    }

    if config.urls.is_empty() {
        return Err("You must provide at least one URL to download.".to_string());
    }

    Ok(config)
}

fn encode_whitespace(url: &str) -> String {
    url.replace(' ', "%20")
}

fn get_curl_version() -> Result<(u32, u32), String> {
    let output = Command::new("curl")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    let version_str = String::from_utf8_lossy(&output.stdout);
    let first_line = version_str.lines().next().ok_or("No version output")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err("Could not parse curl version".to_string());
    }

    let version = parts[1];
    let version_parts: Vec<&str> = version.split('.').collect();
    
    if version_parts.len() < 2 {
        return Err("Invalid version format".to_string());
    }

    let major = version_parts[0].parse::<u32>()
        .map_err(|_| "Invalid major version")?;
    let minor = version_parts[1].parse::<u32>()
        .map_err(|_| "Invalid minor version")?;

    Ok((major, minor))
}

fn get_url_filename(url: &str, decode: bool) -> String {
    let url_without_protocol = url.split("//").nth(1).unwrap_or(url);
    let url_without_query = url_without_protocol.split('?').next().unwrap_or(url_without_protocol);
    
    if let Some(last_slash_pos) = url_without_query.rfind('/') {
        let filename = &url_without_query[last_slash_pos + 1..];
        if !filename.is_empty() {
            if decode {
                return percent_decode(filename);
            }
            return filename.to_string();
        }
    }
    
    "index.html".to_string()
}

fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex1 = chars.next();
            let hex2 = chars.next();
            
            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                let hex_str = format!("{}{}", h1, h2);
                if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                    // Пропускаем декодирование управляющих символов и небезопасных символов
                    if byte >= 0x20 && !is_unsafe_percent_encode(&hex_str) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push(h1);
                result.push(h2);
            } else {
                result.push('%');
                if let Some(h1) = hex1 {
                    result.push(h1);
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn is_unsafe_percent_encode(hex: &str) -> bool {
    let upper = hex.to_uppercase();
    upper == "2F" || upper == "5C" // / и \
}

fn exec_curl(config: Config) -> Result<(), String> {
    let (major, minor) = get_curl_version()?;
    
    let mut curl_args = Vec::new();

    // Флаги для параллельной загрузки
    if config.urls.len() >= 2 {
        if major >= 8 || (major == 7 && minor >= 66) {
            curl_args.push("--parallel".to_string());
            if major >= 8 && minor >= 16 {
                curl_args.push("--parallel-max-host".to_string());
                curl_args.push("5".to_string());
            }
        }
    }

    let per_url_params = vec![
        "--fail",
        "--globoff",
        "--location",
        "--proto-default", "https",
        "--remote-time",
        "--retry", "5",
    ];

    let use_no_clobber = major >= 8 || (major == 7 && minor >= 83);

    for (idx, url) in config.urls.iter().enumerate() {
        if idx > 0 {
            curl_args.push("--next".to_string());
        }

        for param in &per_url_params {
            curl_args.push(param.to_string());
        }

        if use_no_clobber {
            curl_args.push("--no-clobber".to_string());
        }

        let output = if config.has_user_set_output {
            config.output_path.clone().unwrap_or_else(|| "index.html".to_string())
        } else {
            get_url_filename(url, config.decode_filename)
        };

        curl_args.push("--output".to_string());
        curl_args.push(output);

        for opt in &config.curl_options {
            curl_args.push(opt.clone());
        }

        curl_args.push(url.clone());
    }

    if config.dry_run {
        print!("curl");
        for arg in &curl_args {
            print!(" {}", arg);
        }
        println!();
        Ok(())
    } else {
        let status = Command::new("curl")
            .args(&curl_args)
            .status()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("curl exited with status: {}", status))
        }
    }
}

fn print_usage() {
    println!("{} -- a simple wrapper around curl to easily download files.\n", PROGRAM_NAME);
    println!("Usage: {} <URL>...", PROGRAM_NAME);
    println!("       {} [--curl-options <CURL_OPTIONS>]... [--no-decode-filename] [-o|-O|--output <PATH>] [--dry-run] [--] <URL>...", PROGRAM_NAME);
    println!("       {} [--curl-options=<CURL_OPTIONS>]... [--no-decode-filename] [--output=<PATH>] [--dry-run] [--] <URL>...", PROGRAM_NAME);
    println!("       {} -h|--help", PROGRAM_NAME);
    println!("       {} -V|--version\n", PROGRAM_NAME);
    println!("Options:\n");
    println!("  --curl-options <CURL_OPTIONS>: Specify extra options to be passed when invoking curl. May be");
    println!("                                 specified more than once.\n");
    println!("  -o, -O, --output <PATH>: Use the provided output path instead of getting it from the URL. If");
    println!("                           multiple URLs are provided, resulting files share the same name with a");
    println!("                           number appended to the end (curl >= 7.83.0). If this option is provided");
    println!("                           multiple times, only the last value is considered.\n");
    println!("  --no-decode-filename: Don't percent-decode the output filename, even if the percent-encoding in");
    println!("                        the URL was done by wcurl, e.g.: The URL contained whitespace.\n");
    println!("  --dry-run: Don't actually execute curl, just print what would be invoked.\n");
    println!("  -V, --version: Print version information.\n");
    println!("  -h, --help: Print this usage message.\n");
    println!("  <CURL_OPTIONS>: Any option supported by curl can be set here. This is not used by wcurl; it is");
    println!("                 instead forwarded to the curl invocation.\n");
    println!("  <URL>: URL to be downloaded. Anything that is not a parameter is considered");
    println!("         an URL. Whitespace is percent-encoded and the URL is passed to curl, which");
    println!("         then performs the parsing. May be specified more than once.");

}
