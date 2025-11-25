use std::env;
use std::process::{exit, Command};

const VERSION: &str = "2025.11.09-rust";
const PROGRAM_NAME: &str = "wcurl";

#[derive(Debug)]
struct Config {
    curl_options: Vec<String>,
    urls: Vec<String>,
    output_path: Option<String>,
    decode_filename: bool,
    dry_run: bool,
}

impl Config {
    fn new() -> Self {
        Config {
            curl_options: Vec::new(),
            urls: Vec::new(),
            output_path: None,
            decode_filename: true,
            dry_run: false,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();

        return Err("No arguments provided".to_string());
    }

    let config = parse_args(args)?;
    exec_curl(config)
}

fn parse_args(args: Vec<String>) -> Result<Config, String> {
    let mut config = Config::new();

    let mut iter = args.into_iter().skip(1).peekable();
    let mut reading_urls = false;

    while let Some(arg) = iter.next() {
        if reading_urls {
            config.urls.push(encode_whitespace(&arg));
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
            "--dry-run" => config.dry_run = true,
            "--no-decode-filename" => config.decode_filename = false,
            "--" => reading_urls = true,

            "--curl-options" => {
                let opt = iter.next().ok_or("--curl-options requires an argument")?;
                config.curl_options.push(opt);
            }
            "-o" | "-O" | "--output" => {
                let opt = iter.next().ok_or(format!("{} requires an argument", arg))?;
                config.output_path = Some(opt);
            }

            x if x.starts_with("--curl-options=") => {
                let val = x.strip_prefix("--curl-options=").unwrap();
                config.curl_options.push(val.to_string());
            }
            x if x.starts_with("--output=") => {
                let val = x.strip_prefix("--output=").unwrap();
                config.output_path = Some(val.to_string());
            }
            x if x.starts_with("-") => {
                if x.starts_with("-o") || x.starts_with("-O") {
                    if x.len() > 2 {
                        config.output_path = Some(x[2..].to_string());
                    } else {
                        let opt = iter.next().ok_or(format!("{} requires an argument", x))?;
                        config.output_path = Some(opt);
                    }
                } else {
                    return Err(format!("Unknown option: '{}'", x));
                }
            }

            url => {
                config.urls.push(encode_whitespace(url));
            }
        }
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
    let (major_str, minor_str) = version.split_once('.').ok_or("Invalid version format")?;

    let major = major_str
        .parse::<u32>()
        .map_err(|_| "Invalid major version")?;

    let minor = minor_str
        .split('.')
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .map_err(|_| "Invalid minor version")?;

    Ok((major, minor))
}

fn get_url_filename(url: &str, decode: bool) -> String {
    let url_path = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);

    let path_no_query = url_path.split(&['?', '#'][..]).next().unwrap_or(url_path);

    let filename = path_no_query.rsplit('/').next().unwrap_or("");

    if filename.is_empty() {
        return "index.html".to_string();
    }

    if decode {
        percent_decode(filename)
    } else {
        filename.to_string()
    }
}

fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let mut lookahead = chars.clone();
            let h1 = lookahead.next();
            let h2 = lookahead.next();

            if let (Some(d1), Some(d2)) = (h1, h2) {
                let hex_str = format!("{}{}", d1, d2);
                if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                    if byte >= 0x20 && !is_unsafe_char(byte) {
                        result.push(byte as char);

                        chars.next();
                        chars.next();
                        continue;
                    }
                }
            }
        }
        result.push(c);
    }

    result
}

fn is_unsafe_char(byte: u8) -> bool {
    byte == 0x2F || byte == 0x5C
}

fn exec_curl(config: Config) -> Result<(), String> {
    let (major, minor) = get_curl_version()?;

    let mut command = Command::new("curl");

    if config.urls.len() >= 2 {
        if major >= 8 || (major == 7 && minor >= 66) {
            command.arg("--parallel");
            if major >= 8 && minor >= 16 {
                command.args(["--parallel-max-host", "5"]);
            }
        }
    }

    let per_url_params = [
        "--fail",
        "--globoff",
        "--location",
        "--proto-default",
        "https",
        "--remote-time",
        "--retry",
        "5",
    ];

    let use_no_clobber = major >= 8 || (major == 7 && minor >= 83);

    for (idx, url) in config.urls.iter().enumerate() {
        if idx > 0 {
            command.arg("--next");
        }

        command.args(&per_url_params);

        if use_no_clobber {
            command.arg("--no-clobber");
        }

        let output = if let Some(ref path) = config.output_path {
            path.clone()
        } else {
            get_url_filename(url, config.decode_filename)
        };

        command.arg("--output").arg(output);

        command.args(&config.curl_options);

        command.arg(url);
    }

    if config.dry_run {
        print!("curl");
        for arg in command.get_args() {
            print!(" {}", arg.to_string_lossy());
        }
        println!();
        Ok(())
    } else {
        let status = command
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
    println!(
        "{} -- a simple wrapper around curl to easily download files.\n",
        PROGRAM_NAME
    );
    println!("Usage: {} <URL>...", PROGRAM_NAME);
    println!("       {} [--curl-options <CURL_OPTIONS>]... [--no-decode-filename] [-o|-O|--output <PATH>] [--dry-run] [--] <URL>...", PROGRAM_NAME);
    println!("       {} [--curl-options=<CURL_OPTIONS>]... [--no-decode-filename] [--output=<PATH>] [--dry-run] [--] <URL>...", PROGRAM_NAME);
    println!("       {} -h|--help", PROGRAM_NAME);
    println!("       {} -V|--version\n", PROGRAM_NAME);
    println!("Options:\n");
    println!(
        "  --curl-options <CURL_OPTIONS>: Specify extra options to be passed when invoking curl."
    );
    println!("                                 May be specified more than once.\n");
    println!("  -o, -O, --output <PATH>: Use the provided output path instead of getting it from the URL.");
    println!("                           If multiple URLs are provided, resulting files share the same name");
    println!("                           (curl behavior depends on version).\n");
    println!("  --no-decode-filename: Don't percent-decode the output filename.\n");
    println!("  --dry-run: Don't actually execute curl, just print what would be invoked.\n");
    println!("  -V, --version: Print version information.\n");
    println!("  -h, --help: Print this usage message.\n");
}
