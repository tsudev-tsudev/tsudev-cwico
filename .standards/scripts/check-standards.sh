#!/usr/bin/env bash
#
# check-standards.sh - CỔNG KIỂM QUY ƯỚC.
#
# Chạy được ở hai nơi:
#   - Trong chính repo tsudev-standards (kiểm cả token và MANIFEST).
#   - Trong repo con đã đồng bộ (tự tìm bộ quy ước ở ./.standards).
#
# Cách dùng:
#   ./scripts/check-standards.sh
#   ./scripts/check-standards.sh --standards-dir .standards
#
# Mã thoát khác 0 = có vi phạm. CI PHẢI chặn merge khi script này thất bại.
set -uo pipefail

STD_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --standards-dir) STD_DIR="$2"; shift 2 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "Tham số không hiểu: $1" >&2; exit 2 ;;
  esac
done

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

# Tự nhận diện: đang ở repo trung tâm, hay ở repo con?
IS_CENTRAL=0
if [[ -f "templates/gitignore/base.gitignore" && -f "tokens/design-tokens.json" ]]; then
  IS_CENTRAL=1
  STD_DIR="."
elif [[ -z "$STD_DIR" ]]; then
  STD_DIR=".standards"
fi

FAIL=0
WARN=0
pass() { printf '  ĐẠT   %s\n' "$1"; }
fail() { printf '  TRƯỢT %s\n' "$1" >&2; FAIL=$((FAIL + 1)); }
warn() { printf '  LƯU Ý %s\n' "$1" >&2; WARN=$((WARN + 1)); }

echo "Cổng kiểm quy ước tsudev - $( [[ $IS_CENTRAL == 1 ]] && echo 'repo trung tâm' || echo 'repo con' )"
echo

# ------------------------------------------------------------------
echo "1. Bộ quy ước"
# ------------------------------------------------------------------
if [[ $IS_CENTRAL == 0 ]]; then
  if [[ ! -d "$STD_DIR" ]]; then
    fail "không tìm thấy $STD_DIR - chạy ./scripts/sync-standards.sh"
  else
    pass "có bản sao quy ước tại $STD_DIR (v$(cat "$STD_DIR/VERSION" 2>/dev/null || echo '?'))"
    if [[ -f .standards-version ]]; then
      pass "có .standards-version ghi nguồn đồng bộ"
    else
      fail "thiếu .standards-version - bản sao quy ước không truy nguyên được"
    fi
  fi
else
  pass "đây là nguồn chân lý (v$(cat VERSION))"
fi

# ------------------------------------------------------------------
echo
echo "2. .gitignore"
# ------------------------------------------------------------------
BASE="$STD_DIR/templates/gitignore/base.gitignore"
if [[ ! -f .gitignore ]]; then
  fail "repo không có .gitignore"
elif [[ ! -f "$BASE" ]]; then
  warn "không đối chiếu được với bản chuẩn (thiếu $BASE)"
else
  missing=0
  while IFS= read -r line; do
    [[ -z "$line" || "$line" == \#* ]] && continue
    if ! grep -qxF -- "$line" .gitignore; then
      fail ".gitignore thiếu dòng chuẩn: $line"
      missing=$((missing + 1))
      [[ $missing -ge 10 ]] && { echo "  ... (còn nữa, dừng liệt kê)" >&2; break; }
    fi
  done < "$BASE"
  [[ $missing == 0 ]] && pass ".gitignore chứa đủ bản chuẩn"
fi

# ------------------------------------------------------------------
echo
echo "3. File nhạy cảm đang bị git theo dõi"
# ------------------------------------------------------------------
if git rev-parse --git-dir >/dev/null 2>&1; then
  # Mẫu bắt các file KHÔNG BAO GIỜ được nằm trong repo.
  LEAK_RE='(^|/)\.env$|(^|/)\.env\..*(?<!\.example)(?<!\.sample)$|\.pem$|\.pfx$|\.p12$|\.jks$|\.keystore$|(^|/)id_rsa|(^|/)id_ed25519|(^|/)\.npmrc$|(^|/)\.netrc$|(^|/)secrets\.(json|ya?ml)$|service-account.*\.json$|\.tfstate$|\.kdbx$|\.har$'
  leaked="$(git ls-files | grep -PI "$LEAK_RE" || true)"
  if [[ -n "$leaked" ]]; then
    blocked=0
    while IFS= read -r f; do
      # Miễn trừ có ghi chép: .standards-allow, mỗi dòng
      #   <đường dẫn> | <lý do> | <hết hiệu lực DD/MM/YYYY hoặc điều kiện>
      # Một lần awk duy nhất: tách cột rồi in "lý do<TAB>hạn".
      # KHÔNG được gsub lên $1 rồi print $0 - awk sẽ dựng lại $0 bằng OFS và
      # xóa mất chính dấu | đang dùng để phân cột.
      allow_info=""
      if [[ -f .standards-allow ]]; then
        allow_info="$(awk -F'|' -v p="$f" '
          /^[[:space:]]*#/ { next }
          NF >= 1 {
            k = $1; gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
            if (k != p) next
            r = (NF >= 2) ? $2 : ""; gsub(/^[[:space:]]+|[[:space:]]+$/, "", r)
            e = (NF >= 3) ? $3 : ""; gsub(/^[[:space:]]+|[[:space:]]+$/, "", e)
            print r "\t" e
            exit
          }' .standards-allow)"
      fi
      if [[ -n "$allow_info" ]]; then
        reason="${allow_info%%$'\t'*}"
        expiry="${allow_info#*$'\t'}"
        if [[ -z "$reason" || -z "$expiry" ]]; then
          fail "miễn trừ cho $f thiếu lý do hoặc hạn - .standards-allow cần đủ 3 cột"
          blocked=$((blocked + 1))
        else
          warn "$f được miễn trừ có chủ đích: $reason (hạn: $expiry)"
        fi
      else
        fail "file nhạy cảm đang được theo dõi: $f"
        blocked=$((blocked + 1))
      fi
    done <<< "$leaked"
    if [[ $blocked -gt 0 ]]; then
      echo "        Xử lý: git rm --cached <file>, rồi THU HỒI KHÓA theo" >&2
      echo "        docs/SECURITY_BASELINE.md mục 9.2 - khóa đã lộ là đã lộ." >&2
      echo "        Nếu file THẬT SỰ không chứa secret: khai vào .standards-allow" >&2
      echo "        kèm lý do và hạn (docs/GITIGNORE_POLICY.md mục 7)." >&2
    fi
  else
    pass "không có file nhạy cảm nào trong chỉ mục git"
  fi
else
  warn "không phải kho git - bỏ qua kiểm tra file bị theo dõi"
fi

# ------------------------------------------------------------------
echo
echo "3b. File BẮT BUỘC phải được commit"
# ------------------------------------------------------------------
# Quy tắc .gitignore riêng của repo có thể nuốt mất chính những file mà bộ quy
# ước bắt buộc phải commit - và nuốt hoàn toàn im lặng. Hay gặp nhất: repo có
# sẵn dòng `logs/` cho nhật ký chạy của ứng dụng, dòng đó chặn luôn thư mục
# điều phối phiên mà AGENT_PROTOCOL.md dựa vào.
if git rev-parse --git-dir >/dev/null 2>&1; then
  REQUIRED=(logs/STATE.md logs/LOCKS.md)
  [[ $IS_CENTRAL == 0 ]] && REQUIRED+=("$STD_DIR/AGENTS.md" "$STD_DIR/VERSION")
  req_ok=1
  for f in "${REQUIRED[@]}"; do
    if [[ ! -e "$f" ]]; then
      fail "thiếu file bắt buộc: $f"
      req_ok=0
    elif ! git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
      why="$(git check-ignore -v "$f" 2>/dev/null || true)"
      if [[ -n "$why" ]]; then
        fail "$f TỒN TẠI nhưng bị .gitignore chặn nên không được commit"
        echo "        Quy tắc chặn: $why" >&2
        echo "        Sửa quy tắc đó cho hẹp lại (ví dụ logs/*.log thay vì logs/)." >&2
      else
        fail "$f chưa được thêm vào git"
      fi
      req_ok=0
    fi
  done
  # Toàn bộ bản sao quy ước cũng phải commit được, không được khuyết file nào.
  if [[ $IS_CENTRAL == 0 && -d "$STD_DIR" ]]; then
    swallowed="$(find "$STD_DIR" -type f -print0 2>/dev/null \
      | xargs -0 -r git check-ignore 2>/dev/null || true)"
    if [[ -n "$swallowed" ]]; then
      n="$(printf '%s\n' "$swallowed" | grep -c . || true)"
      fail "$n file trong $STD_DIR bị .gitignore chặn - bản sao quy ước sẽ khuyết"
      printf '%s\n' "$swallowed" | head -5 | sed 's/^/        /' >&2
      req_ok=0
    fi
  fi
  [[ $req_ok == 1 ]] && pass "mọi file bắt buộc đều được commit"
fi

# ------------------------------------------------------------------
echo
echo "4. Quy ước gạch ngang (AGENTS.md mục 7.1)"
# ------------------------------------------------------------------
if git rev-parse --git-dir >/dev/null 2>&1; then
  # Dựng ký tự em-dash và en-dash từ mã byte thay vì viết literal, để chính
  # script này không vi phạm quy tắc mà nó đang canh.
  EM_DASH="$(printf '\xe2\x80\x94')"   # U+2014
  EN_DASH="$(printf '\xe2\x80\x93')"   # U+2013

  # Phạm vi quét: MỌI đuôi file văn bản và mã nguồn của hệ sinh thái, không chỉ
  # riêng web. Bản trước chỉ quét 10 đuôi nên em-dash lọt tự do vào .py, .cs,
  # .html, .sql, .toml - đúng những nơi tsudev-swico và tsudev-cwico sống.
  DASH_GLOBS=(
    '*.md' '*.mdx' '*.txt' '*.rst' '*.adoc'
    '*.json' '*.jsonc' '*.yml' '*.yaml' '*.toml' '*.ini' '*.cfg' '*.conf' '*.env.example'
    '*.css' '*.scss' '*.sass' '*.less' '*.html' '*.htm' '*.xml' '*.svg'
    '*.js' '*.mjs' '*.cjs' '*.jsx' '*.ts' '*.tsx' '*.vue' '*.svelte'
    '*.py' '*.rb' '*.php' '*.go' '*.rs' '*.java' '*.kt' '*.kts' '*.swift'
    '*.cs' '*.c' '*.h' '*.cpp' '*.hpp' '*.cc'
    '*.sh' '*.bash' '*.zsh' '*.ps1' '*.bat'
    '*.sql' '*.graphql' '*.proto' '*.gradle' '*.tf'
  )
  # Miễn trừ theo AGENTS.md mục 7.1:
  #   1. migrations/ - file đã áp dụng là bất biến (lệch checksum).
  #   2. dòng trích dẫn chính ký tự này để định nghĩa quy tắc, nhận diện bằng
  #      mã điểm "U+2014" hoặc "U+2013" ghi trên cùng dòng.
  DASH_SKIP='^(migrations/|.*/migrations/|assets/brand/)'

  scan_dash() {
    # $1 = ký tự cần bắt, $2 = mã điểm dùng làm dấu miễn trừ
    local ch="$1" code="$2" hits kept f
    hits="$(git ls-files -- "${DASH_GLOBS[@]}" 2>/dev/null \
      | grep -Ev "$DASH_SKIP" \
      | xargs -r grep -lF -- "$ch" 2>/dev/null || true)"
    [[ -z "$hits" ]] && return 0
    kept=""
    while IFS= read -r f; do
      [[ -z "$f" ]] && continue
      if grep -F -- "$ch" "$f" | grep -qvF "$code"; then
        kept+="$f"$'\n'
      fi
    done <<< "$hits"
    printf '%s' "$kept"
  }

  emdash="$(scan_dash "$EM_DASH" 'U+2014')"
  if [[ -n "$emdash" ]]; then
    while IFS= read -r f; do
      [[ -z "$f" ]] && continue
      fail "còn em-dash (U+2014) trong: $f"
      grep -nF -- "$EM_DASH" "$f" | grep -vF 'U+2014' | head -3 | sed 's/^/        /' >&2
    done <<< "$emdash"
    echo "        Thay bằng gạch ngang ngắn - theo AGENTS.md mục 7.1." >&2
  else
    pass "không còn em-dash trong file văn bản và mã nguồn"
  fi

  # En-dash ở mức SHOULD NOT: cảnh báo chứ không chặn merge.
  endash="$(scan_dash "$EN_DASH" 'U+2013')"
  if [[ -n "$endash" ]]; then
    n_en="$(printf '%s\n' "$endash" | grep -c . || true)"
    warn "có en-dash (U+2013) trong $n_en file - nên thay bằng - (AGENTS.md mục 7.1)"
    printf '%s' "$endash" | head -5 | sed 's/^/        /' >&2
  else
    pass "không có en-dash trong file văn bản và mã nguồn"
  fi
fi

# ------------------------------------------------------------------
echo
echo "5. Định dạng ngày (DESIGN_SYSTEM.md mục 4)"
# ------------------------------------------------------------------
if git rev-parse --git-dir >/dev/null 2>&1; then
  isodate="$(git ls-files -- '*.md' 2>/dev/null \
    | xargs -r grep -lE '(^|[^0-9/-])20[0-9]{2}-[01][0-9]-[0-3][0-9]([^0-9/-]|$)' 2>/dev/null || true)"
  if [[ -n "$isodate" ]]; then
    while IFS= read -r f; do
      warn "có ngày dạng YYYY-MM-DD (quy ước hiển thị là DD/MM/YYYY): $f"
    done <<< "$isodate"
  else
    pass "ngày trong tài liệu theo đúng DD/MM/YYYY"
  fi
fi

# ------------------------------------------------------------------
if [[ $IS_CENTRAL == 1 ]]; then
  echo
  echo "6. Design token (chỉ ở repo trung tâm)"
  if command -v node >/dev/null 2>&1; then
    if node scripts/build-tokens.mjs --check >/dev/null 2>&1; then
      pass "tokens.css khớp design-tokens.json"
    else
      fail "tokens.css lệch design-tokens.json - chạy: node scripts/build-tokens.mjs"
    fi
    if node scripts/check-contrast.mjs >/dev/null 2>&1; then
      pass "mọi cặp màu đạt ngưỡng WCAG"
    else
      fail "có cặp màu trượt WCAG - chạy: node scripts/check-contrast.mjs"
    fi
  else
    warn "thiếu node - bỏ qua kiểm tra token"
  fi

  echo
  echo "6b. Bản mẫu .gitignore không được chặn file bắt buộc"
  # Chính bản mẫu của bộ quy ước từng mắc lỗi này: dotnet.gitignore có dòng
  # [Ll]ogs/ chặn đúng thư mục điều phối phiên mà AGENT_PROTOCOL.md bắt buộc
  # phải commit. Đây là cổng canh để không tái diễn.
  tpl_bad=0
  TMPT="$(mktemp -d)"
  for tpl in templates/gitignore/*.gitignore; do
    name="$(basename "$tpl")"
    [[ "$name" == "base.gitignore" ]] && continue
    rm -rf "${TMPT:?}/t"; mkdir -p "$TMPT/t/logs" "$TMPT/t/.standards/templates/logs"
    ( cd "$TMPT/t" && git init -q . ) || continue
    cat templates/gitignore/base.gitignore "$tpl" > "$TMPT/t/.gitignore"
    touch "$TMPT/t/logs/STATE.md" "$TMPT/t/logs/LOCKS.md" \
          "$TMPT/t/.standards/templates/logs/STATE.md"
    blocked="$( cd "$TMPT/t" && for f in logs/STATE.md logs/LOCKS.md .standards/templates/logs/STATE.md; do
        git check-ignore -q "$f" && echo "$f"
      done || true )"
    if [[ -n "$blocked" ]]; then
      fail "$name chặn file bắt buộc: $(printf '%s' "$blocked" | tr '\n' ' ')"
      tpl_bad=$((tpl_bad + 1))
    fi
  done
  rm -rf "$TMPT"
  [[ $tpl_bad == 0 ]] && pass "không bản mẫu nào chặn file bắt buộc"

  echo
  echo "7. MANIFEST.sha256"
  SHA_CMD="sha256sum"
  command -v sha256sum >/dev/null 2>&1 || SHA_CMD="shasum -a 256"
  if [[ ! -f MANIFEST.sha256 ]]; then
    fail "thiếu MANIFEST.sha256 - chạy: ./scripts/make-manifest.sh"
  elif $SHA_CMD --check MANIFEST.sha256 >/dev/null 2>&1; then
    pass "MANIFEST.sha256 khớp toàn bộ file quy ước"
  else
    fail "MANIFEST.sha256 lệch - chạy: ./scripts/make-manifest.sh"
  fi
fi

# ------------------------------------------------------------------
echo
echo "----------------------------------------------------------"
if [[ $FAIL -gt 0 ]]; then
  echo "KẾT QUẢ: $FAIL vi phạm, $WARN lưu ý. Không đạt cổng kiểm." >&2
  exit 1
fi
echo "KẾT QUẢ: đạt cổng kiểm ($WARN lưu ý)."
exit 0
