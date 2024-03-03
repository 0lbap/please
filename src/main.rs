use structopt::StructOpt;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::sleep;
use tokio::time::Duration;
use tokio::task;
use std::io;
use std::io::Write;
use arboard::Clipboard;
use std::process::Command;
use dotenv::dotenv;
use std::env;

#[derive(Debug, StructOpt)]
#[structopt(name = "please", about = "Generate terminal commands from text prompts")]
struct Cli {
    #[structopt(help = "Text prompt to generate a terminal command")]
    prompt: Vec<String>,

    #[structopt(short = "c", long = "copy", help = "Copy the generated command to the clipboard")]
    copy: bool,

    #[structopt(short = "r", long = "run", help = "Run the generated command")]
    run: bool,

    #[structopt(short = "p", long = "platform", help = "Set the target platform (operating system) the command should run on. If not specified, defaults to the operating system of the current environment.")]
    platform: Option<String>,
}

async fn generate_command(prompt: String, platform: String, openai_api_key: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": format!("Generate a {} terminal command based on the user's input text. Always reply with one command, raw text, no formatting.", platform)
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await?;
    
    let body = response.text().await?;

    let json_response: Value = serde_json::from_str(&body)?;

    let command: String = json_response["choices"][0]["message"]["content"].as_str().expect("Error parsing json to str").to_string();

    Ok(command)
}

fn run_command(command_str: String) -> io::Result<()> {
    let command_parts: Vec<&str> = command_str.as_str().split_whitespace().collect();
    let mut command = Command::new(command_parts[0]);
    for arg in &command_parts[1..] {
        command.arg(arg);
    }
    let status = command.status()?;
    if status.success() {
        println!("Command executed successfully!");
    } else {
        println!("Command failed with exit code: {}", status.code().unwrap_or_default());
    }
    Ok(())
}

async fn cli_loader() {
    let frames = vec!['|', '/', '-', '\\'];
    let mut i = 0;
    loop {
        print!("\rGenerating a command {} ", frames[i]);
        io::stdout().flush().expect("Error flushing stdout");
        i = (i + 1) % frames.len();
        sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let openai_api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not found in environment variables");

    let args = Cli::from_args();
    let prompt = args.prompt.join(" ");
    let platform = args.platform.unwrap_or_else(|| std::env::consts::OS.to_string());

    println!("Prompt: `{}`", prompt);
    println!("Target platform: `{}`", platform);

    let loader_task = task::spawn(cli_loader());

    match generate_command(prompt, platform, openai_api_key).await {
        Ok(command_str) => {
            loader_task.abort();
            print!("\r");
            io::stdout().flush().expect("Error flushing stdout");

            println!("Generated command: `{}`", command_str);
            
            if args.copy {
                println!("Copying `{}` to clipboard...", command_str);
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(command_str.clone()).unwrap();
                println!("Command copied successfully!");
            }

            if args.run {
                println!("Running `{}`...", command_str);
                let _ = run_command(command_str);
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
