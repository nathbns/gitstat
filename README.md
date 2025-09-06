# GitStat 📊

A beautiful, terminal-adaptive CLI tool to display GitHub activity schemas for any user.

![GitStat Demo](https://via.placeholder.com/800x400?text=GitStat+Demo)

## Features ✨

- 🎨 **Beautiful dark blue theme** - Professional color scheme
- 📏 **Terminal-adaptive** - Automatically adjusts to your terminal size  
- 🌐 **Real GitHub data** - Uses GitHub's GraphQL API for accurate contributions
- 🚀 **Fast & lightweight** - Built in Rust for maximum performance
- 🎯 **Simple usage** - Just provide a username and token

## Installation 🔧

### From crates.io (Recommended)
```bash
cargo install gitstat
```

### From source
```bash
git clone https://github.com/nathbns/gitstat
cd gitstat
cargo install --path .
```

## Usage 💻

### Basic usage
```bash
# Set your GitHub token (get one at https://github.com/settings/tokens)
export GITHUB_TOKEN=your_token_here
gitstat username
```

### With token as argument
```bash
gitstat --token your_token_here username
```

### Examples
```bash
# View your own contributions
gitstat nathbns

# View any public user's contributions  
gitstat octocat
```

## GitHub Token 🔑

You need a GitHub personal access token to use this tool:

1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Select **only** the `read:user` permission
4. Copy the token and use it with the `--token` flag or `GITHUB_TOKEN` environment variable

## Output 📈

GitStat displays:
- User information (name, repos, followers, following)
- Beautiful contribution calendar with color-coded activity levels
- Statistics (active days, max contributions per day, averages)
- Terminal-adaptive layout that works on any screen size

## Color Scheme 🎨

- **Dark gray**: No contributions
- **Dark blue**: 1-2 contributions  
- **Medium blue**: 3-5 contributions
- **Bright blue**: 6-10 contributions
- **Very bright blue**: 11+ contributions

## Requirements 📋

- Rust 1.70+ (for installation from source)
- GitHub personal access token
- Terminal with color support

## License 📄

MIT License - see LICENSE file for details.

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request.

## Author 👨‍💻

Created by [Nathan Ben Soussan](https://github.com/nathbns)
