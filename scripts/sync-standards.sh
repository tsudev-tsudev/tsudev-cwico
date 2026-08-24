#!/usr/bin/env bash
#
# sync-standards.sh - đồng bộ bộ quy ước trung tâm tsudev về repo con.
#
# Repo nguồn là PUBLIC nên script này KHÔNG cần token, không cần đăng nhập.
#
# Cách dùng:
#   ./scripts/sync-standards.sh                 # kéo bản mới nhất của nhánh main
#   ./scripts/sync-standards.sh --ref v2.0.0    # ghim theo nhãn phát hành
#   ./scripts/sync-standards.sh --check         # chỉ kiểm tra, không ghi (dùng cho CI)
#   ./scripts/sync-standards.sh --dir .standards
#
# Sau khi chạy: thư mục đích chứa bản sao CHỈ-ĐỌC của bộ quy ước, và file
# .standards-version ghi lại đã đồng bộ từ đâu, lúc nào.
#
# QUAN TRỌNG: thư mục đích PHẢI được commit vào repo con, để lập trình viên và
# agent AI đọc được quy ước khi không có mạng, và để CI kiểm được mà không cần
# gọi ra ngoài.
set -euo pipefail

REPO="tsudev-tsudev/tsudev-standards"
REF="main"
DEST=".standards"
CHECK_ONLY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)   REF="$2"; shift 2 ;;
    --dir)   DEST="$2"; shift 2 ;;
    --check) CHECK_ONLY=1; shift ;;
    --repo)  REPO="$2"; shift 2 ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "Tham số không hiểu: $1" >&2; exit 2 ;;
  esac
done

command -v tar >/dev/null 2>&1 || { echo "Thiếu lệnh tar." >&2; exit 1; }
if command -v curl >/dev/null 2>&1; then
  FETCH=(curl -fsSL)
elif command -v wget >/dev/null 2>&1; then
  FETCH=(wget -qO-)
else
  echo "Cần curl hoặc wget." >&2; exit 1
fi

SHA_CMD="sha256sum"
command -v sha256sum >/dev/null 2>&1 || SHA_CMD="shasum -a 256"

TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

echo "Đang tải $REPO @ $REF ..."
"${FETCH[@]}" "https://codeload.github.com/${REPO}/tar.gz/${REF}" > "$TMP/src.tar.gz"
tar -xzf "$TMP/src.tar.gz" -C "$TMP"
SRC="$(find "$TMP" -maxdepth 1 -type d -name 'tsudev-standards-*' | head -1)"
[[ -n "$SRC" ]] || { echo "Không giải nén được bản tải về." >&2; exit 1; }

UP_VERSION="$(cat "$SRC/VERSION")"

# --- Xác minh tính toàn vẹn của bản tải về bằng MANIFEST.sha256 ---
if [[ -f "$SRC/MANIFEST.sha256" ]]; then
  if ! ( cd "$SRC" && $SHA_CMD --check MANIFEST.sha256 >/dev/null 2>&1 ); then
    echo "LỖI: bản tải về không khớp MANIFEST.sha256. Không dùng bản này." >&2
    ( cd "$SRC" && $SHA_CMD --check MANIFEST.sha256 2>&1 | grep -v ': OK$' >&2 ) || true
    exit 1
  fi
  echo "Đã xác minh toàn vẹn theo MANIFEST.sha256."
else
  echo "CẢNH BÁO: bản $REF không có MANIFEST.sha256, bỏ qua bước xác minh." >&2
fi

# --- Chế độ chỉ kiểm tra ---
if [[ "$CHECK_ONLY" == "1" ]]; then
  status=0
  if [[ ! -d "$DEST" ]]; then
    echo "LỖI: chưa có thư mục $DEST. Chạy ./scripts/sync-standards.sh để đồng bộ lần đầu." >&2
    exit 1
  fi
  LOCAL_VERSION="$(cat "$DEST/VERSION" 2>/dev/null || echo "chưa rõ")"
  if [[ "$LOCAL_VERSION" != "$UP_VERSION" ]]; then
    echo "LỖI: repo đang dùng quy ước $LOCAL_VERSION, bản mới nhất là $UP_VERSION." >&2
    echo "     Chạy: ./scripts/sync-standards.sh" >&2
    status=1
  fi
  while IFS= read -r rel; do
    if ! cmp -s "$SRC/$rel" "$DEST/$rel"; then
      echo "LỆCH: $DEST/$rel khác bản trung tâm." >&2
      status=1
    fi
  done < <(cd "$SRC" && find AGENTS.md VERSION SECURITY.md docs tokens templates -type f | sort)
  if [[ "$status" == "0" ]]; then
    echo "OK: bộ quy ước khớp bản trung tâm $UP_VERSION."
  fi
  exit "$status"
fi

# --- Ghi đè thư mục đích ---
# Lần đồng bộ trước đã đặt thư mục thành chỉ-đọc, nên phải mở quyền ghi lại
# trước khi xóa. Thiếu bước này thì lần đồng bộ THỨ HAI trở đi sẽ thất bại.
[[ -d "$DEST" ]] && chmod -R u+w "$DEST"   # gồm cả cây cũ bị đặt a-w toàn bộ
rm -rf "$DEST"
mkdir -p "$DEST"
for item in AGENTS.md VERSION SECURITY.md MANIFEST.sha256 docs tokens templates; do
  [[ -e "$SRC/$item" ]] && cp -R "$SRC/$item" "$DEST/"
done

# Đánh dấu chỉ-đọc để không ai sửa nhầm bản sao thay vì sửa ở trung tâm.
#
# CHỈ đặt cho FILE, KHÔNG đặt cho thư mục. Thư mục không có quyền ghi thì git
# không unlink được file bên trong: `git checkout` sang nhánh khác vẫn báo thành
# công nhưng bỏ sót file, để lại cây làm việc lệch với nhánh và rác chặn mọi lần
# checkout sau. `git clean` và `rm -rf` cũng hỏng theo.
#
# Lớp chặn này chỉ là rào chắn nhẹ, và git sẽ trả lại quyền ghi sau mỗi lần
# checkout. Cơ chế phát hiện sửa trộm thật sự là `sync-standards.sh --check`
# và `check-standards.sh`, không phải quyền file.
find "$DEST" -type f -exec chmod a-w {} + 2>/dev/null || true

COMMIT="$("${FETCH[@]}" "https://api.github.com/repos/${REPO}/commits/${REF}" 2>/dev/null \
  | sed -n 's/.*"sha"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{40\}\)".*/\1/p' | head -1 || true)"

cat > .standards-version <<META
# Sinh tự động bởi scripts/sync-standards.sh - KHÔNG sửa tay.
repo=${REPO}
ref=${REF}
version=${UP_VERSION}
commit=${COMMIT:-chưa rõ}
synced_at=$(date +'%H:%M %d/%m/%Y')
META

echo "Đã đồng bộ bộ quy ước v${UP_VERSION} vào ./${DEST}"
echo "Nhớ commit cả ./${DEST} và ./.standards-version."
