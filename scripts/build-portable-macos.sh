#!/usr/bin/env bash
# ==============================================================================
# Script đóng gói phiên bản ExamPanel Portable cho macOS
# Sử dụng:
#   ./scripts/build-portable-macos.sh              (Build mới rồi đóng gói)
#   ./scripts/build-portable-macos.sh --skip-build (Chỉ đóng gói từ bản release đã build sẵn)
# ==============================================================================

set -euo pipefail

SKIP_BUILD=false
for arg in "$@"; do
  case $arg in
    --skip-build|-SkipBuild)
      SKIP_BUILD=true
      shift
      ;;
  esac
done

echo "=========================================================="
echo "   BẮT ĐẦU ĐÓNG GÓI EXAMPANEL PHIÊN BẢN PORTABLE (macOS)"
echo "=========================================================="

# 1. Biên dịch ứng dụng nếu không bật --skip-build
if [ "$SKIP_BUILD" = false ]; then
  echo ""
  echo "[1/4] Đang biên dịch frontend và ứng dụng Tauri (Release cho macOS)..."
  pnpm tauri build
else
  echo ""
  echo "[1/4] Bỏ qua bước biên dịch (--skip-build)..."
fi

# 2. Tìm kiếm bundle ExamPanel.app đã được biên dịch
APP_SRC=""
CANDIDATES=(
  "target/release/bundle/macos/ExamPanel.app"
  "src-tauri/target/release/bundle/macos/ExamPanel.app"
  "target/universal-apple-darwin/release/bundle/macos/ExamPanel.app"
  "target/aarch64-apple-darwin/release/bundle/macos/ExamPanel.app"
  "target/x86_64-apple-darwin/release/bundle/macos/ExamPanel.app"
  "src-tauri/target/universal-apple-darwin/release/bundle/macos/ExamPanel.app"
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ExamPanel.app"
  "src-tauri/target/x86_64-apple-darwin/release/bundle/macos/ExamPanel.app"
)

for cand in "${CANDIDATES[@]}"; do
  if [ -d "$cand" ]; then
    APP_SRC="$cand"
    break
  fi
done

if [ -z "$APP_SRC" ]; then
  echo "Lỗi: Không tìm thấy ExamPanel.app trong các thư mục target/.../bundle/macos/!" >&2
  exit 1
fi

echo "  -> Đã tìm thấy ứng dụng nguồn: $APP_SRC"

# 3. Chuẩn bị thư mục target/portable-macos/
echo ""
echo "[2/4] Chuẩn bị thư mục đích target/portable-macos/..."
PORTABLE_DIR="target/portable-macos/ExamPanel-portable"
rm -rf "target/portable-macos"
mkdir -p "$PORTABLE_DIR"

# 4. Sao chép và cấu hình chế độ portable
echo ""
echo "[3/4] Sao chép các tệp thành phần vào $PORTABLE_DIR/..."

# Sao chép ExamPanel.app (dùng cp -R hoặc ditto để bảo toàn quyền thực thi và symlink)
if command -v ditto >/dev/null 2>&1; then
  ditto "$APP_SRC" "$PORTABLE_DIR/ExamPanel.app"
else
  cp -R "$APP_SRC" "$PORTABLE_DIR/ExamPanel.app"
fi
echo "  -> Đã sao chép: ExamPanel.app"

# Tạo marker ExamPanel.portable ở cả 2 vị trí để đảm bảo nhận diện 100%:
# 1) Bên cạnh ExamPanel.app (thư mục gốc portable)
# 2) Bên trong ExamPanel.app/Contents/MacOS/ (đề phòng chạy trực tiếp binary từ bundle)
touch "$PORTABLE_DIR/ExamPanel.portable"
if [ -d "$PORTABLE_DIR/ExamPanel.app/Contents/MacOS" ]; then
  touch "$PORTABLE_DIR/ExamPanel.app/Contents/MacOS/ExamPanel.portable"
fi
echo "  -> Đã tạo marker: ExamPanel.portable (kích hoạt chế độ di động)"

# Sao chép tài liệu hướng dẫn và giấy phép
if [ -f "DOC-TOI.txt" ]; then
  cp "DOC-TOI.txt" "$PORTABLE_DIR/DOC-TOI.txt"
  echo "  -> Đã sao chép: DOC-TOI.txt"
fi

if [ -f "THIRD_PARTY_NOTICES" ]; then
  cp "THIRD_PARTY_NOTICES" "$PORTABLE_DIR/THIRD_PARTY_NOTICES.txt"
  echo "  -> Đã sao chép: THIRD_PARTY_NOTICES.txt"
fi

# 5. Đóng gói thành file nén .tar.gz và .zip
echo ""
echo "[4/4] Nén thư mục portable..."

# Lấy version từ package.json
VERSION=$(node -p "require('./package.json').version" 2>/dev/null || echo "0.1.0")
ARCH=$(uname -m 2>/dev/null || echo "universal")
TAR_FILE="target/ExamPanel-${VERSION}-macos-${ARCH}-portable.tar.gz"
ZIP_FILE="target/ExamPanel-${VERSION}-macos-${ARCH}-portable.zip"

# Nén tar.gz (chuẩn nhất cho macOS / Unix để bảo toàn permissions)
(cd target/portable-macos && tar -czf "../../$TAR_FILE" ExamPanel-portable)
echo "  -> Đã tạo tệp nén TAR.GZ: $TAR_FILE"

# Nén zip (dùng ditto trên macOS nếu có, hoặc zip)
if command -v ditto >/dev/null 2>&1; then
  ditto -c -k --sequesterRsrc --keepParent "$PORTABLE_DIR" "$ZIP_FILE"
  echo "  -> Đã tạo tệp nén ZIP: $ZIP_FILE"
elif command -v zip >/dev/null 2>&1; then
  (cd target/portable-macos && zip -q -r -y "../../$ZIP_FILE" ExamPanel-portable)
  echo "  -> Đã tạo tệp nén ZIP: $ZIP_FILE"
fi

echo ""
echo "=========================================================="
echo " HOÀN TẤT! BẢN PORTABLE CHO MACOS ĐÃ SẴN SÀNG:"
echo " - Thư mục chạy thử: target/portable-macos/ExamPanel-portable/ExamPanel.app"
echo " - Tệp lưu hành:     $TAR_FILE"
echo "=========================================================="
