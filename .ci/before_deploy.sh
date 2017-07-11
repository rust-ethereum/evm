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
    cd $src

    rm -rf $stage
}

main
