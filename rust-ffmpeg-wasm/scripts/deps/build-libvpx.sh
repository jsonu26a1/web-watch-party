# script adapted from https://github.com/ffmpegwasm/ffmpeg.wasm/blob/main/build/libvpx.sh

set -e

source ./scripts/vars.sh

OPTS=(
  --prefix=$INSTALL_DIR                             # install library in a build directory for FFmpeg to include
  --target=generic-gnu                               # target with miminal features
  --disable-install-bins                             # not to install bins
  --disable-examples                                 # not to build examples
  --disable-tools                                    # not to build tools
  --disable-docs                                     # not to build docs
  --disable-unit-tests                               # not to do unit tests
  --disable-dependency-tracking                      # speed up one-time build

  --disable-multithread
)

# janky test to see if source var is defined
if [ ! -f "$LIBVPX_SOURCE/configure" ]; then
  echo "LIBVPX_SOURCE is not a valid path"
  exit 1
fi

DEST="$BUILD_DIR/libvpx"

mkdir -p $DEST
cd $DEST

emconfigure "$LIBVPX_SOURCE/configure" "${OPTS[@]}"
emmake make install -j
