# Homebrew Formula for Namra

This directory contains the Homebrew formula template for Namra.

## Setup Instructions

### 1. Create the Homebrew Tap Repository

Create a new repository: `namra-agents/homebrew-tap`

```bash
# On GitHub, create new repo: namra-agents/homebrew-tap
# Then clone it locally
git clone git@github.com:namra-agents/homebrew-tap.git
cd homebrew-tap
mkdir Formula
```

### 2. Copy the Formula

```bash
cp /path/to/namra/homebrew-formula/namra.rb Formula/namra.rb
```

### 3. After First Release

After running `git tag v0.1.0 && git push origin v0.1.0`, the release workflow will:
1. Build binaries for all platforms
2. Create `.sha256` files for each binary

Download the SHA256 files and update `Formula/namra.rb`:

```bash
# Download SHA256 files from release
curl -LO https://github.com/namra-agents/namra/releases/download/v0.1.0/namra-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
curl -LO https://github.com/namra-agents/namra/releases/download/v0.1.0/namra-v0.1.0-x86_64-apple-darwin.tar.gz.sha256
curl -LO https://github.com/namra-agents/namra/releases/download/v0.1.0/namra-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256

# View the hashes
cat *.sha256
```

Replace `PLACEHOLDER_*` values in `namra.rb` with actual SHA256 hashes.

### 4. Commit and Push

```bash
cd homebrew-tap
git add Formula/namra.rb
git commit -m "Add namra formula v0.1.0"
git push origin main
```

### 5. Test Installation

```bash
brew tap namra-agents/tap
brew install namra
namra --version
```

## Updating for New Releases

1. Update `version` in `namra.rb`
2. Update SHA256 hashes from new release
3. Commit and push to homebrew-tap repo

```bash
# In homebrew-tap repo
git commit -am "Update namra to v0.2.0"
git push origin main
```

Users can then upgrade:
```bash
brew update
brew upgrade namra
```
