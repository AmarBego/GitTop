#!/bin/bash
# Helper script to test PKGBUILD locally by simulating a release artifact
set -e

# 1. Build release
echo "Building release..."
cargo build --release

# 2. Create fake release tarball structure
VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)
DIR_NAME="gittop-linux-gnu-x86_64"
rm -rf "$DIR_NAME" "$DIR_NAME.tar.gz"
mkdir -p "$DIR_NAME"

echo "Creating tarball for version $VERSION..."
cp target/release/gittop "$DIR_NAME/"
cp LICENSE.md "$DIR_NAME/"
cp README.md "$DIR_NAME/"
cp src/platform/resources/gittop.desktop "$DIR_NAME/"
cp assets/images/GitTop-256x256.png "$DIR_NAME/gittop.png"

# 3. Compress
tar -czf "gittop-linux-gnu-x86_64-$VERSION.tar.gz" "$DIR_NAME"

# 4. Update PKGBUILD checksum
echo "Updating PKGBUILD checksum..."
SHA=$(sha256sum "gittop-linux-gnu-x86_64-$VERSION.tar.gz" | awk '{print $1}')
sed -i "s/sha256sums=('.*')/sha256sums=('$SHA')/" packaging/aur/PKGBUILD
sed -i "s/pkgver=.*/pkgver=$VERSION/" packaging/aur/PKGBUILD

# 5. Copy tarball to where makepkg expects it (or just run makepkg pointing to file)
# Best way for local test: put tarball in same dir as PKGBUILD, and change source to file:// or just name
mv "gittop-linux-gnu-x86_64-$VERSION.tar.gz" packaging/aur/

echo "Ready to test!"
echo "Run: cd packaging/aur && makepkg -fp"
echo ""
echo "Note: I updated sha256sums automatically for you."
