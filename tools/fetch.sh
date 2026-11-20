#!/bin/sh
# Install the pinned external tools into tools/bin (git-ignored).
#
#   tools/fetch.sh            cvc5, Ethos and the CPC signature
#   tools/fetch.sh carcara    also build Carcara from its pinned commit (needs cargo)
#   tools/fetch.sh all        everything
#
# Every download is checked against the SHA-256 in tools/pins.sh before it is
# unpacked. Re-running is cheap: a tool already at the pinned version is kept.
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/pins.sh"
BIN=$HERE/bin
CACHE=$HERE/.cache
mkdir -p "$BIN" "$CACHE"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) PLAT=macOS_arm64; CVC5_ASSET=cvc5-macOS-arm64-static; ETHOS_ASSET=ethos-macOS-arm64 ;;
  Linux-x86_64) PLAT=Linux_x86_64; CVC5_ASSET=cvc5-Linux-x86_64-static; ETHOS_ASSET=ethos-linux-x86_64 ;;
  Linux-aarch64 | Linux-arm64) PLAT=Linux_arm64; CVC5_ASSET=cvc5-Linux-arm64-static; ETHOS_ASSET=ethos-linux-arm64 ;;
  *) echo "fetch.sh: unsupported platform $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# download <url> <file> <sha256>
download() {
  if [ -f "$2" ] && [ "$(sha256 "$2")" = "$3" ]; then return; fi
  echo "downloading $1"
  curl -sSfL -o "$2.part" "$1"
  got=$(sha256 "$2.part")
  if [ "$got" != "$3" ]; then
    rm -f "$2.part"
    echo "fetch.sh: checksum mismatch for $1" >&2
    echo "  expected $3" >&2
    echo "  got      $got" >&2
    exit 1
  fi
  mv "$2.part" "$2"
}

# installed <stamp-name> <version>: true if that version is already installed.
installed() { [ "$(cat "$BIN/.$1" 2>/dev/null)" = "$2" ]; }
stamp() { echo "$2" >"$BIN/.$1"; }

fetch_cvc5() {
  installed cvc5 "$CVC5_VERSION" && return
  eval "want=\$CVC5_SHA256_$PLAT"
  download "$CVC5_URL/$CVC5_ASSET.zip" "$CACHE/cvc5.zip" "$want"
  rm -rf "$CACHE/cvc5" && mkdir -p "$CACHE/cvc5"
  unzip -q "$CACHE/cvc5.zip" -d "$CACHE/cvc5"
  install -m 755 "$CACHE/cvc5/$CVC5_ASSET/bin/cvc5" "$BIN/cvc5"
  stamp cvc5 "$CVC5_VERSION"
}

fetch_ethos() {
  installed ethos "$ETHOS_VERSION" && return
  eval "want=\$ETHOS_SHA256_$PLAT"
  download "$ETHOS_URL/$ETHOS_ASSET.zip" "$CACHE/ethos.zip" "$want"
  rm -rf "$CACHE/ethos" && mkdir -p "$CACHE/ethos"
  unzip -q "$CACHE/ethos.zip" -d "$CACHE/ethos"
  install -m 755 "$CACHE/ethos/ethos" "$BIN/ethos"
  stamp ethos "$ETHOS_VERSION"
}

fetch_cpc() {
  installed cpc "$CVC5_VERSION" && return
  download "$CPC_URL" "$CACHE/cvc5-src.tar.gz" "$CPC_SHA256"
  rm -rf "$BIN/cpc" "$CACHE/cpc" && mkdir -p "$CACHE/cpc"
  tar xzf "$CACHE/cvc5-src.tar.gz" -C "$CACHE/cpc" "cvc5-cvc5-$CVC5_VERSION/proofs/eo/cpc"
  mv "$CACHE/cpc/cvc5-cvc5-$CVC5_VERSION/proofs/eo/cpc" "$BIN/cpc"
  stamp cpc "$CVC5_VERSION"
}

fetch_carcara() {
  installed carcara "$CARCARA_REV" && return
  command -v cargo >/dev/null 2>&1 || { echo "fetch.sh: carcara needs cargo" >&2; exit 1; }
  cargo install --quiet --locked --git "$CARCARA_REPO" --rev "$CARCARA_REV" \
    --root "$CACHE/carcara" carcara
  install -m 755 "$CACHE/carcara/bin/carcara" "$BIN/carcara"
  stamp carcara "$CARCARA_REV"
}

case "${1:-default}" in
  default) fetch_cvc5; fetch_ethos; fetch_cpc ;;
  carcara) fetch_cvc5; fetch_ethos; fetch_cpc; fetch_carcara ;;
  all) fetch_cvc5; fetch_ethos; fetch_cpc; fetch_carcara ;;
  *) echo "usage: tools/fetch.sh [carcara|all]" >&2; exit 2 ;;
esac

echo "tools/bin:"
for t in cvc5 ethos cpc carcara; do
  if [ -e "$BIN/.$t" ]; then printf '  %-8s %s\n' "$t" "$(cat "$BIN/.$t")"; fi
done
