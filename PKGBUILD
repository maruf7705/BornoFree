pkgname=bornofree-fcitx5
pkgver=1.0.1
pkgrel=1
pkgdesc='Headless BornoFree Bengali Unicode input method for Fcitx5 and Hyprland'
arch=('x86_64')
url='https://github.com/maruf7705/BornoFree'
license=('MIT')
depends=('fcitx5' 'fcitx5-gtk' 'fcitx5-qt' 'waybar' 'python')
makedepends=('cargo' 'cmake' 'ninja' 'rust')
provides=('fcitx5-borno')
source=()
sha256sums=()

build() {
  cmake -S linux/fcitx5 -B build-linux -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr
  cmake --build build-linux
}

check() {
  cargo test --workspace
  ctest --test-dir build-linux --output-on-failure
}

package() {
  DESTDIR="$pkgdir" cmake --install build-linux
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
