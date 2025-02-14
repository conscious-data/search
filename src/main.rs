use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::{command, Parser};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use webbrowser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(help_template = "{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}

Examples:
    search \"what is an inframodel\"
    search --clipboard \"what's causing this error?\"
    search -x main.py \"walk me through this code\"
    search -p chatgpt \"how do I write a fish function\"
")]
struct Args {
    /// Invoke conscious-data/contextualize to load content from specified paths
    #[arg(short = 'x', long, num_args = 1.., value_delimiter = ' ')]
    context: Option<Vec<String>>,

    /// Inject clipboard content as context
    #[arg(short, long)]
    clipboard: bool,

    /// LLM provider to use
    #[arg(short, long, default_value = "chatgpt")]
    provider: String,

    /// prompt/query text
    #[arg(trailing_var_arg = true)]
    prompt: Vec<String>,
}
fn get_clipboard_content() -> Result<String> {
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    clipboard
        .get_text()
        .context("Failed to get clipboard content")
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

fn run_contextualize(files: &[String]) -> Result<()> {
    use std::process::Command;

    let mut cmd = Command::new("contextualize");
    cmd.arg("cat").arg("--output").arg("clipboard");
    cmd.args(files);

    let output = cmd
        .output()
        .context("Failed to run contextualize command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("contextualize command failed: {}", stderr);
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.clipboard && args.context.is_some() {
        anyhow::bail!("--clipboard and --context flags are not compatible");
    }

    let content = if let Some(files) = args.context {
        run_contextualize(&files)?;
        get_clipboard_content()?
    } else if args.clipboard {
        get_clipboard_content()?
    } else {
        String::new()
    };

    if !content.is_empty() {
        let formatted = format_content(&content, &args.prompt);
        run_search(&formatted, &args.provider)?;
    } else {
        let query = args.prompt.join(" ");
        run_search(&query, &args.provider)?;
    }

    Ok(())
}
