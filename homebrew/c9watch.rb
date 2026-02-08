# Homebrew Cask for c9watch
#
# SETUP INSTRUCTIONS:
#
# 1. Create a new GitHub repo: <your-username>/homebrew-tap
#
# 2. Copy this file into that repo as:
#    Casks/c9watch.rb
#
# 3. After each release, update:
#    - version
#    - sha256 values (run: shasum -a 256 <file>.tar.gz)
#    - url paths if your GitHub username differs
#
# 4. Users install with:
#    brew tap <your-username>/tap
#    brew install --cask c9watch
#
cask "c9watch" do
  version "0.1.0"

  on_arm do
    url "https://github.com/anthropics/c9watch/releases/download/v#{version}/c9watch_v#{version}_aarch64.app.tar.gz"
    sha256 "REPLACE_WITH_AARCH64_SHA256"
  end

  on_intel do
    url "https://github.com/anthropics/c9watch/releases/download/v#{version}/c9watch_v#{version}_x86_64.app.tar.gz"
    sha256 "REPLACE_WITH_X86_64_SHA256"
  end

  name "c9watch"
  desc "Monitor and control all your Claude Code sessions from one place"
  homepage "https://github.com/anthropics/c9watch"

  app "c9watch.app"

  zap trash: [
    "~/Library/Application Support/com.minchenlee.c9watch",
    "~/Library/Caches/com.minchenlee.c9watch",
    "~/Library/Preferences/com.minchenlee.c9watch.plist",
  ]
end
