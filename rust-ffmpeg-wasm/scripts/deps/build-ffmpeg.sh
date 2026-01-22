# script adapted from https://github.com/ffmpegwasm/ffmpeg.wasm/blob/main/build/ffmpeg.sh

set -e

source ./scripts/vars.sh

EM_PKG_CONFIG_PATH="$INSTALL_DIR/lib/pkgconfig"
export EM_PKG_CONFIG_PATH

OPTS=(
  --prefix=$INSTALL_DIR

  --enable-small
  --disable-runtime-cpudetect

  --disable-programs

  --disable-doc

  # rusty_ffmpeg doesn't like when we disable this
  # --disable-avdevice
  --disable-network

  --disable-encoders
  --enable-encoder=libvpx_vp9
  --enable-encoder=opus
  --disable-muxers
  --enable-muxer=webm
  --enable-muxer=mp4

  --disable-hwaccels
  --disable-bsfs
  --disable-protocols
  --disable-indevs
  --disable-outdevs
  --disable-devices

  --enable-libopus
  --enable-libvpx
  --disable-sdl2

  # --cpu=x86_32
  --disable-asm

  --disable-pthreads

  # from github/ffmpeg.wasm; some repeats here?
  --target-os=none              # disable target specific configs
  --arch=x86_32                 # use x86_32 arch
  --enable-cross-compile        # use cross compile configs
  --disable-stripping           # disable stripping as it won't work
  --disable-debug               # disable debug mode
  --disable-runtime-cpudetect   # disable cpu detection
  --disable-autodetect          # disable env auto detect
  # assign toolchains and extra flags
  --nm=emnm
  --ar=emar
  --ranlib=emranlib
  --cc=emcc
  --cxx=em++
  --objcc=emcc
  --dep-cc=emcc
  --extra-cflags="$CFLAGS"
  --extra-cxxflags="$CXXFLAGS"

)

# janky test to see if source var is defined
if [ ! -f "$FFMPEG_SOURCE/configure" ]; then
  echo "FFMPEG_SOURCE is not a valid path"
  exit 1
fi

DEST="$BUILD_DIR/ffmpeg"

mkdir -p $DEST
cd $DEST

# oops we don't need this
# echo $FFMPEG_SOURCE > "$BUILD_DIR/path-to-ffmpeg-source.txt"

emconfigure "$FFMPEG_SOURCE/configure" "${OPTS[@]}"
emmake make install -j
