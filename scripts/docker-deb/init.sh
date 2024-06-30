# Install pre-requisites
apt-get update && apt-get install -y git gcc curl pkg-config dpkg-dev

# Install rustup
curl https://sh.rustup.rs -sSf | sh -s -- -y
source ~/.cargo/env

# Clone the repo
git clone $GITREPO ~/nyaa

# Install cargo-deb for building .deb package
cargo install cargo-deb

pushd ~/nyaa

# Build .deb package in github mode
cargo deb -- --profile github

# Export it to shared directory
cp ~/nyaa/target/debian/nyaa_${VERSION}-1_amd64.deb ~/docker/nyaa-${VERSION}-x86_64.deb
chown 1000:1000 ~/docker/*.deb

apt install ~/nyaa/target/debian/nyaa_${VERSION}-1_amd64.deb

popd
