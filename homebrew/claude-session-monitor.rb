# Homebrew Cask for Claude Session Monitor
#
# SETUP INSTRUCTIONS:
#
# 1. Create a new GitHub repo: <your-username>/homebrew-tap
#
# 2. Copy this file into that repo as:
#    Casks/claude-session-monitor.rb
#
# 3. After each release, update:
#    - version
#    - sha256 values (run: shasum -a 256 <file>.tar.gz)
#    - url paths if your GitHub username differs
#
# 4. Users install with:
#    brew tap <your-username>/tap
#    brew install --cask claude-session-monitor
#
cask "claude-session-monitor" do
  version "0.1.0"

  on_arm do
    url "https://github.com/anthropics/claude-session-monitor/releases/download/v#{version}/Claude-Session-Monitor_v#{version}_aarch64.app.tar.gz"
    sha256 "REPLACE_WITH_AARCH64_SHA256"
  end

  on_intel do
    url "https://github.com/anthropics/claude-session-monitor/releases/download/v#{version}/Claude-Session-Monitor_v#{version}_x86_64.app.tar.gz"
    sha256 "REPLACE_WITH_X86_64_SHA256"
  end

  name "Claude Session Monitor"
  desc "macOS menu bar app to monitor and control Claude Code sessions"
  homepage "https://github.com/anthropics/claude-session-monitor"

  app "Claude Session Monitor.app"

  zap trash: [
    "~/Library/Application Support/com.minchenlee.claude-session-monitor",
    "~/Library/Caches/com.minchenlee.claude-session-monitor",
    "~/Library/Preferences/com.minchenlee.claude-session-monitor.plist",
  ]
end
