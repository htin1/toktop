# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/htin1/toktop/compare/v0.1.4...v0.1.5) - 2025-12-07

### Fixed

- fix cost api error
- fix encoding

### Other

- Enhance UI color palette and rendering styles. Update chart colors for better contrast, improve block styling in options and summary sections, and implement smart scaling for cost and usage charts to handle outliers effectively.

## [0.1.4](https://github.com/htin1/toktop/compare/v0.1.3...v0.1.4) - 2025-11-10

### Other

- Refactor summary rendering to support filtering and grouping by model or API keys. Update cost and usage calculations to incorporate selected filters, enhancing data presentation in the UI.
- default dont show segment value

## [0.1.3](https://github.com/htin1/toktop/compare/v0.1.2...v0.1.3) - 2025-11-10

### Other

- add a new hot key to toggle details - show values of each segment
- for cost chart, add threshold of $1 per day to reduce noise
- refactor cost and usage because they are repetitive. move more rendering logic to shared.rs
- clean up each file; rename header.rs to summary.rs. no functionality change, just removed some dead code/repetitive code
- update demo

## [0.1.2](https://github.com/htin1/toktop/compare/v0.1.1...v0.1.2) - 2025-11-09

### Other

- cargo fmt
- only shows filter that passes threshold
- update hotkey instruction
- add scroll bar

## [0.1.1](https://github.com/htin1/toktop/compare/v0.1.0...v0.1.1) - 2025-11-09

### Fixed

- fix yaml

### Other

- release-plz workflow
- add cargo.lock
- Add num_requests field to DailyUsageData and calculate total requests for OpenAI
- more metadata like cache rate, average, trend
- reset filter when changing providers + cargo fmt
- group by filters - can select individual model/api_key for the chart
- set a max width so if a filter only yields one bar, it wont look too big
- display each model cost in legend
- esc to quit when popping for api keys
- add option for 30 days range. we always fetch 30days results, then based on option 7d or 30d, adjust the rendering. also update to vertical bar, i think it makes more sense/align with the openai/anthropic dashboards
- shows summary if either cost or usage is fetched.
