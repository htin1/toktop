<div align="center">

<pre>
   __                __          __                         
  |  \              |  \        |  \                        
 _| $$_     ______  | $$   __  _| $$_     ______    ______  
|   $$ \   /      \ | $$  /  \|   $$ \   /      \  /      \ 
 \$$$$$$  |  $$$$$$\| $$_/  $$ \$$$$$$  |  $$$$$$\|  $$$$$$\
  | $$ __ | $$  | $$| $$   $$   | $$ __ | $$  | $$| $$  | $$
  | $$|  \| $$__/ $$| $$$$$$\   | $$|  \| $$__/ $$| $$__/ $$
   \$$  $$ \$$    $$| $$  \$$\   \$$  $$ \$$    $$| $$    $$
    \$$$$   \$$$$$$  \$$   \$$    \$$$$   \$$$$$$ | $$$$$$$ 
                                                  | $$      
                                                  | $$      
                                                   \$$      
</pre>

<em>htop but for llm tokens</em>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![](https://shields.io/badge/-Rust-3776AB?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

<img src="https://vhs.charm.sh/vhs-2oaQsyOnZGZSgoxN13DrQS.gif" alt="Made with VHS">
<a href="https://vhs.charm.sh">
   <img src="https://stuff.charm.sh/vhs/badge.svg">
</a>

</div>

toktop is a terminal-based dashboard for OpenAI and Anthropic token usage and costs. It allows user (admin of their openai and anthropic accounts) to monitor their usage inside terminal without constantly jumping into different websites.

## Installation

```bash
cargo install toktop --locked
```

## Usage
```bash
# 1. Set Environment Variables
export OPENAI_ADMIN_KEY="your-openai-key"
export ANTHROPIC_ADMIN_KEY="your-anthropic-key"
toktop

# 2. Use env file
toktop -e .env

# 3. Don't set env var, toktop will prompt for api key
toktop
```


## Hotkeys

- `←/→` - Switch between options columns (Provider, Metrics, Date Range, Group By)
- `↑/↓` - Choosing options
- `h/l` - Scrolling charts if scroll bar is present
- `r` - Refresh data
- `q` - Quit the application

## API Keys

### OPENAI
It requires `$OPENAI_ADMIN_KEY` from https://platform.openai.com/settings/organization/admin-keys with READ permission to `Management API Scope` and `Usage API Scope`.

It tracks two data:
1. Cost: `GET /v1/organization/costs`
2. Usage:
   - `GET v1/organization/usage/completions`
   - `GET v1/organization/usage/embeddings`
   - `GET v1/organization/usage/images`

To group by API Keys, it also needs read access to the following endpoint:
1. Get project ids: `GET v1/organization/projects`
2. Get api key names from projects: `GET v1/organization/projects/{project_id}/api_keys`

### ANTHROPIC
It requires `$ANTHROPIC_ADMIN_KEY` from https://console.anthropic.com/settings/admin-keys.

It tracks two data:
1. Cost: `GET /v1/organizations/cost_report`
2. Usage: `GET /v1/organizations/usage_report/messages`

### Others
I would love to track my other LLM spends from Gemini and Cursor as well. Unfortunately, Cursor Admin API requires enterprise plan, which I don't have at the moment. Gemini does not seem to expose usage/cost in API in the same manner.