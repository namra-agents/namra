# Homebrew formula for Namra
# Repository: namra-agents/homebrew-tap
# Path: Formula/namra.rb
#
# After first release, update SHA256 values from the .sha256 files in the release

class Namra < Formula
  desc "Enterprise AI agent framework with Rust core and YAML configuration"
  homepage "https://github.com/namra-agents/namra"
  version "0.1.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/namra-agents/namra/releases/download/v#{version}/namra-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_DARWIN"
    end
    on_intel do
      url "https://github.com/namra-agents/namra/releases/download/v#{version}/namra-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_DARWIN"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/namra-agents/namra/releases/download/v#{version}/namra-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_LINUX"
    end
    on_intel do
      url "https://github.com/namra-agents/namra/releases/download/v#{version}/namra-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX"
    end
  end

  def install
    bin.install "namra"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/namra --version")
  end
end
