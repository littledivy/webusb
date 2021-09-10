rm -rf coverage/ *.profraw
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="webusb-%p-%m.profraw" cargo test -- --nocapture
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --excl-br-start "mod tests \{" --excl-start "mod tests \{" -o ./coverage/