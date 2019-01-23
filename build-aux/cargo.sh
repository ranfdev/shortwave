#!/bin/sh

export CARGO_HOME=$1/target/cargo-home
export LOCALEDIR="$3"
export APP_ID="$4"
export NAME_SUFFIX="$5"
export VERSION="$6"
export PROFILE="$7"

if [ "$PROFILE" = "Devel" ]
then	
    echo "DEBUG MODE"
    cargo build --manifest-path $1/Cargo.toml -p shortwave && cp $1/target/debug/shortwave $2
else
    echo "RELEASE MODE"
    cargo build --manifest-path $1/Cargo.toml --release -p shortwave && cp $1/target/release/shortwave $2
fi
