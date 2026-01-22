# set -e

source ./scripts/vars.sh

# if we build rusty_ffmpeg for the wasm32-unknown-emscripten target, we get this error:
# "fatal error: 'errno.h' file not found"
# in other words, clang cannot find headers for the standard C library on this target

# possible work arounds?
# - is there a way to just "install" them? or a way to point at emscripten's stdlib?
#   I don't know how to do either of these.
# - could we use the "use_prebuilt_binding" feature of rusty_ffmpeg?
#   build.rs says this should only be used when building for docs.rs. oh, and the feature
#   is only on the master branch, not the latest released version
# - we could first build for the host target triple, copy the binding.rs file to use when
#   building for wasm32-unknown-emscripten
#   this is what I'll go with for now.

# failed idea for overriding:
# cargo --config ./scripts/deps/binding-override.toml build --target "$(rustc --print host-tuple)"

# this is stupid, I hate cargo
mv ./.cargo/config.toml ./.cargo/config.toml-disabled

FFMPEG_LIBS_DIR=$INSTALL_DIR/lib \
FFMPEG_INCLUDE_DIR=$BUILD_DIR/ffmpeg/src \
  cargo check

# this is stupid, I hate cargo
mv ./.cargo/config.toml-disabled ./.cargo/config.toml

cp ./target/debug/build/rusty_ffmpeg-*/out/binding.rs ./deps/binding.rs
