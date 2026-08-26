class JsonrpcStdioProxy < Formula
  desc "Transparent JSON-RPC stdio proxy with native OSLog tracing and metrics"
  homepage "https://github.com/paaloeye/jsonrpc-stdio-proxy"
  url "https://github.com/paaloeye/jsonrpc-stdio-proxy/releases/download/v0.1.0/jsonrpc-stdio-proxy-v0.1.0-macos-universal.tar.gz"
  sha256 "db93b503da0eeb639d0a85bc455b73a7ebe9d67b6cc5e05483492f14a27941a0"
  license "MIT"
  head "https://github.com/paaloeye/jsonrpc-stdio-proxy.git", branch: "main"

  depends_on "rust" => :build
  depends_on :macos

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "jsonrpc-stdio-proxy"
    end
  end

  test do
    assert_match "jsonrpc-stdio-proxy", shell_output("#{bin}/jsonrpc-stdio-proxy --version")
  end
end
