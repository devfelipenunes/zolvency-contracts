set -e
# Download pre-built binary for Linux x86_64
curl -L https://github.com/stellar/stellar-cli/releases/download/v22.1.0/stellar-cli-x86_64-unknown-linux-gnu.tar.gz -o stellar-cli.tar.gz
tar -xzf stellar-cli.tar.gz
mv stellar .bin/stellar
rm stellar-cli.tar.gz
./.bin/stellar --version
