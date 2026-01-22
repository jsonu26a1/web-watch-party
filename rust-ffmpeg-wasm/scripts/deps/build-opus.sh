# script adapted from https://github.com/ffmpegwasm/ffmpeg.wasm/blob/main/build/opus.sh

set -e

source ./scripts/vars.sh

OPTS=(
  --prefix=$INSTALL_DIR
  --host=i686-none                                    # use i686 unknown
  --enable-shared=no                                  # not to build shared library
  --disable-asm                                       # not to use asm
  --disable-rtcd                                      # not to detect cpu capabilities
  --disable-intrinsics                                # not to use intrinsics
  --disable-doc                                       # not to build docs
  --disable-extra-programs                            # not to build demo and tests
  --disable-stack-protector
)

# janky test to see if source var is defined
if [ ! -f "$OPUS_SOURCE/configure" ]; then
  echo "OPUS_SOURCE is not a valid path"
  exit 1
fi

DEST="$BUILD_DIR/opus"

mkdir -p $DEST
cd $DEST

emconfigure "$OPUS_SOURCE/configure" "${OPTS[@]}"
emmake make install -j
