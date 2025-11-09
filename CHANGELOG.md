# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
