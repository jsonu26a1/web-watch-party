set -e

# first, build ffmpeg deps
./scripts/deps/build-libvpx.sh
./scripts/deps/build-opus.sh

# last, build ffmpeg and generate binding.rs
./scripts/deps/build-ffmpeg.sh
./scripts/deps/generate-bindings.sh
