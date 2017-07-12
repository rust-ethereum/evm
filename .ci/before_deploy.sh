# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    cargo build --release --all

    cp target/release/sputnikvm $stage/

    cd $stage
    tar czf $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz *
    sha256sum $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz
    if [ "$TRAVIS_OS_NAME" -eq "osx" ]
    then
        shasum -a 256 $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz > $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz.sha256
    else
        sha256sum $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz > $src/sputnikvm-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz.sha256
    fi
    cd $src

    rm -rf $stage
}

main
