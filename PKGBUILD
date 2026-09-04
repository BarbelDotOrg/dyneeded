# Maintainer: Barbel <you@example.com>
pkgname=dyneeded
pkgver=0.1.0
pkgrel=1
pkgdesc="Inspect and report a binary's needed dynamic dependencies"
arch=('x86_64' 'aarch64')
url="https://github.com/barbeldotorg/dyneeded"
license=('MIT')  # <- set to your actual license
depends=('gcc-libs')
makedepends=('cargo' 'cmake' 'clang')
source=("$pkgname-$pkgver.tar.gz::https://github.com/barbeldotorg/dyneeded/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')  # CI runs `updpkgsums` to fill this in per-release

prepare() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  [ -f LICENSE ] && install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  [ -f README.md ] && install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
