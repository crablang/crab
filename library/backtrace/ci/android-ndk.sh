set -ex

ANDROID_NDK_URL=https://dl.google.com/android/repository
ANDROID_NDK_ARCHIVE=android-ndk-r25b-linux.zip

mkdir /android-toolchain
cd /android-toolchain
curl -fO $ANDROID_NDK_URL/$ANDROID_NDK_ARCHIVE
unzip -q $ANDROID_NDK_ARCHIVE
rm $ANDROID_NDK_ARCHIVE
mv android-ndk-* ndk

cd /tmp
rm -rf android
