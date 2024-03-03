# please

please is an AI powered terminal command generator written in Rust. It uses OpenAI's text completion API in order to generate the right command from the user's input text prompt.

## Compile and install

```sh
$ git clone https://github.com/0lbap/please.git
$ cd please
$ echo "OPENAI_API_KEY=Your OpenAI API Key" >> .env
$ cargo build
```

## Usage

You can simply run the executable with `please <Your Text Prompt>`.

The target platform is automatically set to your platform, but you can change it with `-p <Your Platform>` (or `--platform`).

The default behavior is just displaying the generated terminal command on screen, but you can use `-c` (or `--copy`) to copy the command to the clipboard, or `-r` (or `--run`) to run the command.

See every options with `-h` (or `--help`).
