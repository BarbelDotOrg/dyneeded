# Maintainer: Barbel <barbel@barbel.org>
pkgname=dyneeded
pkgver=0.5.2
pkgrel=1
pkgdesc="A better, fancier, cross platform LDD"
arch=('x86_64' 'aarch64')
url="https://github.com/barbeldotorg/dyneeded"
license=('AGPL-3')
depends=('gcc-libs')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('e139b154a5e537da95ddbab4b7fa447ca97124b0e7a8ab4be98ee21fb6965091')

prepare() {
    cd "$pkgname-$pkgver"
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

check() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --release
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
    install -Dm0644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md" 2>/dev/null || true
}
