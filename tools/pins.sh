# Pinned external tools for experiments and the ADR-0006 slice.
# Sourced by tools/fetch.sh. Changing a version here changes experiment
# results: record the new version in the experiment report that uses it.
#
# None of these is a build dependency or part of the trusted base.

# cvc5: SMT solver producing Alethe and CPC certificates.
CVC5_VERSION=1.4.2
CVC5_URL=https://github.com/cvc5/cvc5/releases/download/cvc5-$CVC5_VERSION
CVC5_SHA256_macOS_arm64=24397f7d022e755876ca836193d4c5f9c1d4326f0f6c01f7d7fc1ddd6bc5096c
CVC5_SHA256_Linux_x86_64=7eb18f8c814c36a46c5a7d003983e9cfbc4b0a9828d6dd596239291bd4cc61d8
CVC5_SHA256_Linux_arm64=765ebee59d9e5efccb60a445fa66fbddfb07a8de9fe7a5b6d128e5c2aafec974

# CPC signature (Eunoia files that define cvc5's proof rules for Ethos),
# from the cvc5 source tree at the same tag.
CPC_URL=https://github.com/cvc5/cvc5/archive/refs/tags/cvc5-$CVC5_VERSION.tar.gz
CPC_SHA256=795099b277bfa79a3c9a11c8d3fec780452031dc89ddcc170003963aa3521343

# Ethos: checker for CPC proofs. 0.2.5 is the exact commit cvc5 1.4.2 pins
# in contrib/get-ethos-checker (08e4aa40c4f8a6e00833f10e8d8985777e424027).
ETHOS_VERSION=0.2.5
ETHOS_URL=https://github.com/cvc5/ethos/releases/download/ethos-$ETHOS_VERSION
ETHOS_SHA256_macOS_arm64=6eb6ddf21a5c9b129d07a2ad59e3b4bbd4ba3cbdd3066111a5cf1bd729094edc
ETHOS_SHA256_Linux_x86_64=5d9c41e53d97d918ccd682b0012032ba26994d964309328cddb4a20df116e296
ETHOS_SHA256_Linux_arm64=736eb758f69da12a72962ee0bf9203127f82267ec54668162b2b17f7fe207832

# Carcara: Alethe checker and elaborator. The last release (1.1.0, 2023)
# predates cvc5 1.4.2's Alethe output, so it is pinned to a commit and built
# from source with its own Cargo.lock (--locked).
CARCARA_REPO=https://github.com/ufmg-smite/carcara
CARCARA_REV=819f30914665019898cbad7e09d3962053b6f97a

# Dafny and Verus: baseline verifiers for experiment 0005 (ADR-0006 S6).
# Official release zips; each bundles its own Z3. Verus also needs the Rust
# toolchain it names (installed through rustup on first run).
DAFNY_VERSION=4.11.0
DAFNY_URL=https://github.com/dafny-lang/dafny/releases/download/v$DAFNY_VERSION
DAFNY_SHA256_macOS_arm64=c90c75e7d5db9c6ccbb7127840dfe43f0ac938b039a7ebed146d8ead383a572f
DAFNY_SHA256_Linux_x86_64=a46a9ff7cdd720f7955854c78e95df13f4cfe6b80691b05f8654fe19e8267179

VERUS_VERSION=0.2026.11.02.8bf68ee
VERUS_URL=https://github.com/verus-lang/verus/releases/download/release/$VERUS_VERSION
VERUS_SHA256_macOS_arm64=3086974d1afebf8b9f6fee2939b626486a12248c0dab528b9262ea652b863326
VERUS_SHA256_Linux_x86_64=f3a0119292df8855e4435da8130bee1873b0e6a60c306accf81a08ea687bba3c
VERUS_SHA256_Linux_arm64=bc28850dd5efd1679a4cf4ffd3309921c3523c69fa4757079e5d12d4122d400d
