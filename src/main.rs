use anyhow::{Context, Result};
use clap::Parser;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::process::Command;
use webbrowser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// system clipboard
    #[arg(short, long)]
    clipboard: bool,

    /// LLM provider
    #[arg(short, long, default_value = "claude")]
    provider: String,

    /// query
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

fn get_clipboard_content() -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .context("Failed to run pbpaste")?;
        Ok(String::from_utf8(output.stdout)?)
    }
    #[cfg(target_os = "linux")]
    {
        let wayland_output = Command::new("wl-paste").output();
        match wayland_output {
            Ok(output) => return Ok(String::from_utf8(output.stdout)?),
            Err(_) => {
                let output = Command::new("xclip")
                    .args(["-selection", "clipboard", "-o"])
                    .output()
                    .context("Failed to run clipboard command. Install xclip or wl-paste")?;
                Ok(String::from_utf8(output.stdout)?)
            }
        }
    }
}

fn format_content(content: &str, query: &[String]) -> String {
    let formatted = if content.contains("```") {
        format!("<paste>\n{}\n</paste>", content)
    } else {
        format!("```paste\n{}\n```", content)
    };

    if !query.is_empty() {
        format!("{}\n{}", formatted, query.join(" "))
    } else {
        formatted
    }
}

fn get_provider_url(provider: &str, query: &str) -> Result<String> {
    let encoded_query = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();

    let url = match provider {
        "claude" => format!("https://claude.ai/new?q={}", encoded_query),
        "chatgpt" => format!("https://chatgpt.com/?q={}", encoded_query),
        _ => anyhow::bail!("Unsupported provider: {}", provider),
    };

    Ok(url)
}

fn run_search(input: &str, provider: &str) -> Result<()> {
    let url = get_provider_url(provider, input)?;
    webbrowser::open(&url).context("Failed to open browser")?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.clipboard {
        let content = get_clipboard_content()?;
        if content.trim().is_empty() {
            anyhow::bail!("Clipboard is empty");
        }
        let formatted = format_content(&content, &args.query);
        run_search(&formatted, &args.provider)?;
    } else {
        let query = args.query.join(" ");
        run_search(&query, &args.provider)?;
    }

    Ok(())
}
