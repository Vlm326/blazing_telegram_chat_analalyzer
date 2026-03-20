# blazing_telegram_chat_analalyzer

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Serde](https://img.shields.io/badge/serde-json-green.svg)](https://serde.rs/)

Small Rust utility for reading a Telegram JSON export and writing chat statistics into separate files.

## Features

- Reads Telegram export JSON with a tolerant `serde` model.
- Detects a `.json` file in the current directory automatically.
- Falls back to reading a path from stdin if no JSON file is found.
- Writes separate statistics files next to the source JSON.

## Output Files

The program writes these files:

- `top_reactions.txt`
- `top_people_by_reactions.txt`
- `top_people_by_symbols.txt`
- `top_people_by_messages.txt`

## Requirements

- Rust toolchain with Cargo
- Valid Telegram export JSON containing a `messages` array

## Run

If the JSON export is in the project directory:

```bash
cargo run
```

If you want to pass the path manually:

```bash
printf '/path/to/result.json\n' | cargo run
```

## Notes

- If the JSON file is truncated or malformed, deserialization will fail.
- Statistics are generated only after the JSON is parsed successfully.

## License

This project is licensed under the Apache License 2.0. See [LICENSE](./LICENSE).
