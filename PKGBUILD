# Maintainer: Sebastian Korotkiewicz <skorotkiewicz@gmail.com>
pkgname=pemdash
pkgver=0.1.0
pkgrel=1
pkgdesc="Fast GTK calculator overlay with proper operator precedence"
arch=('x86_64' 'aarch64')
url="https://github.com/skorotkiewicz/pemdash"
license=('MIT')
depends=('gcc-libs' 'gtk4')
makedepends=()
options=(!strip)

source_x86_64=("$pkgname-$pkgver-$pkgrel-x86_64.tar.gz::$url/releases/download/v$pkgver/pemdash-v$pkgver-x86_64-unknown-linux-gnu.tar.gz")
source_aarch64=("$pkgname-$pkgver-$pkgrel-aarch64.tar.gz::$url/releases/download/v$pkgver/pemdash-v$pkgver-aarch64-unknown-linux-gnu.tar.gz")

sha256sums_x86_64=('SKIP')
sha256sums_aarch64=('SKIP')

package() {
  install -Dm755 pemdash "$pkgdir/usr/bin/pemdash"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
