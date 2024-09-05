# Xatu

A Discord bot that fullfills your thirst for github statistics.

## Features

- **Issue Statistics**: Statistics about open and closed issues.
- **Pull Request Statistics**: Statistics about open and closed pull requests.

*WIP*
- **Various Discord commands**: Show graphes and more detailed statistics

## Installation

### Prerequisites

- Rust stable or nightly

### Steps

1. **Clone the repository**:
   ```sh
   git clone https://github.com/Ninjdai1/xatu.git
   cd xatu
   ```

2. **Build**:
   ```sh
   cargo build --release
   ```

3. **Set up environment variables**:
   Create a `.env` file in the root directory of the project and add the following variables:
   ```env
   DISCORD_TOKEN="your_discord_bot_token" # Self-explanatory
   GITHUB_TOKEN="your_github_token" # A Github Personal Acce ss Token (PAT), to make authenticated requests to the Github API (thus increasing rate limits)
   GIST_ID="gist_id" # Optional, requires a PAT with the gists permission
   ```

4. **Start the bot**:
   ```sh
   ./target/release/xatu
   ```

## License

This project is licensed under the GPLv3 License. See [LICENSE](LICENSE).

## Contact

For any questions or support, please open an issue or contact me on Discord (@ninjdai).

## Acknowledgments

Uses the following Rust crates:
- chrono
- octocrab
- serenity
- tokio
- tokio-cron-scheduler
- rusqlite
- secrecy
- dotenvy

---

Made with ❤️ by [Ninjdai](https://github.com/Ninjdai1)
