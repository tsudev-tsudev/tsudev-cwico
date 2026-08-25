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
#   ./scripts/sync-standards.sh --check --ref main   # kiểm xem đã có bản mới hơn chưa
#   ./scripts/sync-standards.sh --dir .standards
#
# Sau khi chạy: thư mục đích chứa bản sao CHỈ-ĐỌC của bộ quy ước, và file
# .standards-version ghi lại đã đồng bộ từ đâu, lúc nào.
#
# QUAN TRỌNG - hai hành vi dễ bất ngờ, cả hai đều có chủ đích:
#
# 1. `--check` KHÔNG mặc định đối chiếu với `main`, mà với đúng NHÃN repo này
#    đang ghim trong .standards-version. Cổng kiểm phải trả lời "bản sao của tôi
#    có bị sửa trộm không", chứ không phải "tôi đã chạy theo bản mới nhất chưa" -
#    nếu không thì việc ghim theo nhãn vô nghĩa: mỗi lần trung tâm phát hành là
#    CI của mọi repo con đỏ dù chúng không làm gì sai. Muốn hỏi câu thứ hai thì
#    truyền `--ref main`, và đó là việc của PR nâng cấp định kỳ, không phải của
#    cổng chặn merge. Chi tiết: docs/SYNC.md mục 5.
#
# 2. Đồng bộ CÓ ghi đè `scripts/check-standards.sh` của repo con. Cổng kiểm là
#    một phần của bộ quy ước, không phải file riêng của repo con. Trước bản
#    v3.1.0 nó nằm ngoài gói đồng bộ, nên repo con đồng bộ xong vẫn chạy cổng
#    kiểm cũ, thấy xanh và tưởng mình đạt chuẩn.
#
# QUAN TRỌNG: thư mục đích PHẢI được commit vào repo con, để lập trình viên và
# agent AI đọc được quy ước khi không có mạng, và để CI kiểm được mà không cần
# gọi ra ngoài.
set -euo pipefail

REPO="tsudev-tsudev/tsudev-standards"
REF="main"
REF_EXPLICIT=0
DEST=".standards"
CHECK_ONLY=0
VERSION_FILE=".standards-version"

# Bộ file thuộc gói đồng bộ. Dùng chung cho cả bước chép lẫn bước --check, để
# hai bước không bao giờ lệch nhau - lệch một lần là đẻ ra đúng loại lỗ hổng mà
# v3.1.0 đang vá.
SYNC_DIRS=(AGENTS.md VERSION SECURITY.md docs tokens templates)
SYNC_SCRIPTS=(check-standards.sh sync-standards.sh)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)   REF="$2"; REF_EXPLICIT=1; shift 2 ;;
    --dir)   DEST="$2"; shift 2 ;;
    --check) CHECK_ONLY=1; shift ;;
    --repo)  REPO="$2"; shift 2 ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "Tham số không hiểu: $1" >&2; exit 2 ;;
  esac
done

# TS-17: `--check` đối chiếu với đúng nhãn repo này đang ghim, không phải với
# `main`. Truyền `--ref` tay thì tôn trọng lựa chọn đó.
if [[ "$CHECK_ONLY" == "1" && "$REF_EXPLICIT" == "0" && -f "$VERSION_FILE" ]]; then
  PINNED_REF="$(sed -n 's/^ref=//p' "$VERSION_FILE" | head -1)"
  [[ -n "$PINNED_REF" ]] && REF="$PINNED_REF"
fi

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
  CHECK_PATHS=("${SYNC_DIRS[@]}")
  for sc in "${SYNC_SCRIPTS[@]}"; do
    # Chỉ đối chiếu script trong $DEST khi bản đang ghim thật sự có mang nó.
    # Gói đồng bộ chỉ chứa scripts/ từ v3.1.0; repo còn ghim bản cũ hơn mà bị
    # bắt lỗi thiếu file thì đó là cổng kiểm đỏ oan, không phải phát hiện lỗi.
    if [[ -e "$SRC/scripts/$sc" && -e "$DEST/scripts/$sc" ]]; then
      CHECK_PATHS+=("scripts/$sc")
    elif [[ -e "$SRC/scripts/$sc" && ! -e "$DEST/scripts/$sc" ]]; then
      echo "LƯU Ý: $DEST/scripts/$sc chưa có - bản đang ghim cũ hơn v3.1.0." >&2
      echo "       Chạy ./scripts/sync-standards.sh để lấy cổng kiểm vào gói đồng bộ." >&2
    fi
  done
  while IFS= read -r rel; do
    if ! cmp -s "$SRC/$rel" "$DEST/$rel"; then
      echo "LỆCH: $DEST/$rel khác bản trung tâm." >&2
      status=1
    fi
  done < <(cd "$SRC" && find "${CHECK_PATHS[@]}" -type f | sort)

  # Cổng kiểm mà repo con thật sự chạy nằm ở scripts/, không phải trong $DEST.
  # Không soi chỗ này thì nó âm thầm cũ đi và cổng kiểm mất tác dụng.
  if [[ -f "$SRC/scripts/check-standards.sh" ]] \
     && ! cmp -s "$SRC/scripts/check-standards.sh" "scripts/check-standards.sh"; then
    echo "LỆCH: scripts/check-standards.sh khác bản trung tâm $UP_VERSION." >&2
    echo "     Chạy: ./scripts/sync-standards.sh" >&2
    status=1
  fi

  if [[ "$status" == "0" ]]; then
    echo "OK: bộ quy ước khớp bản trung tâm $UP_VERSION (ref=$REF)."
  fi
  exit "$status"
fi

# --- Ghi đè thư mục đích ---
# Lần đồng bộ trước đã đặt thư mục thành chỉ-đọc, nên phải mở quyền ghi lại
# trước khi xóa. Thiếu bước này thì lần đồng bộ THỨ HAI trở đi sẽ thất bại.
[[ -d "$DEST" ]] && chmod -R u+w "$DEST"   # gồm cả cây cũ bị đặt a-w toàn bộ
rm -rf "$DEST"
mkdir -p "$DEST"
for item in "${SYNC_DIRS[@]}" MANIFEST.sha256; do
  [[ -e "$SRC/$item" ]] && cp -R "$SRC/$item" "$DEST/"
done

# TS-16: cổng kiểm đi theo gói đồng bộ. Chỉ lấy hai script repo con thật sự cần;
# make-manifest.sh và build-tokens.mjs là việc của repo trung tâm, gửi xuống chỉ
# tạo cơ hội chạy nhầm.
mkdir -p "$DEST/scripts"
for sc in "${SYNC_SCRIPTS[@]}"; do
  [[ -e "$SRC/scripts/$sc" ]] && cp "$SRC/scripts/$sc" "$DEST/scripts/"
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

# --- Cập nhật cổng kiểm mà repo con thật sự chạy ---
#
# Bản trong $DEST là bản chỉ-đọc để đối chiếu; bản chạy được nằm ở scripts/.
# Ghi bằng mv chứ không cp đè: cp cắt cụt rồi ghi lại ĐÚNG inode đang mở, mà
# chính script này có thể đang chạy từ cùng thư mục - bash đọc file theo từng
# đoạn nên ghi đè inode giữa chừng là hỏng phiên chạy. mv thay inode, tiến trình
# đang chạy giữ nguyên inode cũ.
GATE_SRC="$DEST/scripts/check-standards.sh"
GATE_DST="scripts/check-standards.sh"
if [[ -f "$GATE_SRC" ]]; then
  if [[ ! -f "$GATE_DST" ]] || ! cmp -s "$GATE_SRC" "$GATE_DST"; then
    mkdir -p scripts
    cp "$GATE_SRC" "$GATE_DST.tmp.$$"
    chmod +x "$GATE_DST.tmp.$$"
    mv -f "$GATE_DST.tmp.$$" "$GATE_DST"
    echo "Đã cập nhật $GATE_DST theo bản trung tâm."
  fi
fi

# sync-standards.sh KHÔNG tự ghi đè chính nó. Đổi công cụ bootstrap ngay giữa
# lúc nó đang chạy là loại rủi ro không đáng đổi lấy chút tiện lợi; báo để người
# quyết.
SYNC_SRC="$DEST/scripts/sync-standards.sh"
if [[ -f "$SYNC_SRC" ]] && ! cmp -s "$SYNC_SRC" "scripts/sync-standards.sh"; then
  echo "CẢNH BÁO: scripts/sync-standards.sh khác bản trung tâm. Cập nhật bằng:" >&2
  echo "  cp $SYNC_SRC scripts/sync-standards.sh && chmod +x scripts/sync-standards.sh" >&2
fi

COMMIT="$("${FETCH[@]}" "https://api.github.com/repos/${REPO}/commits/${REF}" 2>/dev/null \
  | sed -n 's/.*"sha"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{40\}\)".*/\1/p' | head -1 || true)"

cat > "$VERSION_FILE" <<META
# Sinh tự động bởi scripts/sync-standards.sh - KHÔNG sửa tay.
repo=${REPO}
ref=${REF}
version=${UP_VERSION}
commit=${COMMIT:-chưa rõ}
synced_at=$(date +'%H:%M %d/%m/%Y')
META

echo "Đã đồng bộ bộ quy ước v${UP_VERSION} vào ./${DEST}"
echo "Nhớ commit ./${DEST}, ./${VERSION_FILE} và ./scripts/check-standards.sh."
