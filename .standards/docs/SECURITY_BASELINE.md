# CHUẨN BẢO MẬT BẮT BUỘC - mọi project/tool/phần mềm tsudev (v3.0.0)

> **Bất khả xâm phạm.** File này là chuẩn tối thiểu, không phải danh sách gợi ý.
> Repo con mang bản sao chỉ-đọc, đồng bộ xuống từ `tsudev-standards`, không sửa ngược.
>
> Ký hiệu mức độ (theo RFC 2119): `MUST` = bắt buộc, không đạt thì không được
> merge; `SHOULD` = nên làm, bỏ qua phải ghi lý do vào ADR/PR; `MAY` = tùy chọn.

---

## Mục lục

- [1. Sáu nguyên tắc nền](#1-sáu-nguyên-tắc-nền)
- [2. Phân loại dữ liệu](#2-phân-loại-dữ-liệu)
- [3. Quản lý secret và biến môi trường](#3-quản-lý-secret-và-biến-môi-trường)
- [4. Cổng kiểm mã nguồn](#4-cổng-kiểm-mã-nguồn)
- [5. Chuỗi cung ứng phần mềm](#5-chuỗi-cung-ứng-phần-mềm)
- [6. Xác thực, phiên và phân quyền](#6-xác-thực-phiên-và-phân-quyền)
- [7. Bảo mật tầng ứng dụng](#7-bảo-mật-tầng-ứng-dụng)
- [8. Hạ tầng, mạng và triển khai](#8-hạ-tầng-mạng-và-triển-khai)
- [9. Quy trình ứng phó sự cố](#9-quy-trình-ứng-phó-sự-cố)
- [10. Checklist theo giai đoạn](#10-checklist-theo-giai-đoạn)
- [11. Bộ công cụ 0 đồng](#11-bộ-công-cụ-0-đồng)
- [12. Yêu cầu riêng cho agent AI](#12-yêu-cầu-riêng-cho-agent-ai)

---

## 1. Sáu nguyên tắc nền

1. **Không tin đầu vào nào.** Mọi dữ liệu đến từ ngoài tiến trình (người dùng,
   API bên thứ ba, file tải lên, biến môi trường, thậm chí database của chính
   mình) `MUST` được kiểm tra bằng **danh sách trắng** trước khi dùng. Danh sách
   đen luôn thiếu sót vì kẻ tấn công chỉ cần tìm ra một trường hợp bạn quên.
2. **Phòng thủ nhiều lớp.** Một lớp bảo vệ hỏng thì lớp sau phải còn đứng. Ví dụ
   cụ thể: sanitize HTML ở backend khi lưu **và** sanitize lại khi render.
3. **Đặc quyền tối thiểu.** Mọi khóa, token, vai trò, quyền file, quyền workflow
   `MUST` cấp ở mức nhỏ nhất đủ dùng, và có thời hạn nếu nền tảng hỗ trợ.
4. **An toàn khi hỏng (fail secure).** Khi có lỗi/nghi ngờ, hệ thống `MUST` từ
   chối thay vì cho qua. `MUST NOT` viết `catch { return true }` ở luồng kiểm
   tra quyền.
5. **Mặc định là an toàn.** Cấu hình gốc phải an toàn ngay khi chưa ai chỉnh gì.
   Tính năng rủi ro (chạy mã, xem mã nguồn HTML, xuất dữ liệu hàng loạt) mặc
   định **tắt**, bật có chủ đích và có kiểm tra quyền.
6. **Ghi được, truy được.** Hành vi nhạy cảm (đăng nhập, đổi quyền, xóa dữ liệu,
   xuất dữ liệu) `MUST` được ghi nhật ký đủ để điều tra sau, nhưng `MUST NOT` ghi
   chính giá trị nhạy cảm ra log.

---

## 2. Phân loại dữ liệu

Mọi project `MUST` phân loại dữ liệu mình xử lý vào 4 mức, ghi vào
`docs/ARCHITECTURE.md` của repo. Mức càng cao, yêu cầu càng chặt.

| Mức | Tên | Ví dụ | Yêu cầu tối thiểu |
| --- | --- | --- | --- |
| D0 | Công khai | Nội dung bài viết đã xuất bản, tài liệu, design token | Không |
| D1 | Nội bộ | Bản nháp, log ứng dụng, thống kê tổng hợp | Kiểm soát truy cập theo vai trò |
| D2 | Cá nhân | Họ tên, email, số điện thoại, địa chỉ, ảnh đại diện | Mã hóa khi truyền + khi lưu; ghi nhật ký truy cập; có chính sách lưu trữ và xóa |
| D3 | Bí mật | Mật khẩu, khóa API, token phiên, dữ liệu tài chính, CCCD/số định danh | Như D2, thêm: băm hoặc mã hóa riêng từng bản ghi, hạn chế người truy cập, bắt buộc rà soát bảo mật khi đụng tới |

**Quy tắc bổ sung:**

- `MUST NOT` đưa dữ liệu D2/D3 thật vào môi trường development hoặc staging. Dùng
  dữ liệu giả sinh bằng script trong `scripts/`.
- `MUST NOT` ghi dữ liệu D3 ra log, ra thông báo lỗi, ra URL query string, hay ra
  hệ thống giám sát bên thứ ba.
- `MUST` khai báo thời hạn lưu trữ cho dữ liệu D2/D3 và có cơ chế xóa khi hết hạn.
- Với dữ liệu cá nhân của người dùng Việt Nam, `MUST` đối chiếu Nghị định
  13/2023/ND-CP về bảo vệ dữ liệu cá nhân trước khi thu thập trường mới.

---

## 3. Quản lý secret và biến môi trường

### 3.1. Nguyên tắc

- `MUST NOT` có bất kỳ secret nào trong mã nguồn, comment, test, tài liệu, ảnh
  chụp màn hình, hay lịch sử git. Không có ngoại lệ, kể cả secret của môi trường
  development, kể cả "chỉ tạm thời".
- Secret `MUST` chỉ tồn tại ở đúng một trong ba nơi: file `.env*` cục bộ (đã nằm
  trong `.gitignore`), kho secret của nền tảng (GitHub Actions Secrets, Cloudflare
  Workers Secrets, Vercel Environment Variables), hoặc file mã hóa bằng SOPS+age
  đã commit dạng đã mã hóa.
- Mọi biến môi trường được dùng `MUST` có mặt trong `.env.example` với **giá trị
  trống hoặc giá trị giả**, kèm một dòng chú thích nói rõ nó là gì.
- Ứng dụng `MUST` kiểm tra sự có mặt của biến bắt buộc **lúc khởi động** và
  dừng hẳn với thông báo rõ ràng nếu thiếu, thay vì chạy tiếp rồi hỏng giữa chừng.

### 3.2. Sinh secret

`MUST` sinh bằng nguồn ngẫu nhiên mật mã, tối thiểu 256 bit:

```bash
openssl rand -base64 48          # chuỗi bí mật chung (JWT, session)
openssl rand -hex 32             # khóa dạng hex
python3 -c "import secrets; print(secrets.token_urlsafe(48))"
```

`MUST NOT` dùng `Math.random()`, `rand()`, `uuid4()` làm nguồn sinh secret hay
token đặt lại mật khẩu.

### 3.3. Vòng đời khóa

| Loại khóa | Chu kỳ đổi tối đa | Ghi chú |
| --- | --- | --- |
| Token cá nhân (PAT), khóa triển khai | 90 ngày | Ưu tiên token có thời hạn và phạm vi hẹp |
| Khóa API dịch vụ ngoài | 12 tháng | Đổi ngay nếu người giữ khóa rời dự án |
| JWT/session secret | 12 tháng | Có cơ chế đổi luân phiên, chấp nhận 2 khóa trong thời gian chuyển |
| Chứng chỉ TLS | Theo nhà cung cấp | `MUST` tự động gia hạn |

### 3.4. Nếu bắt buộc phải commit secret dạng mã hóa

Chỉ chấp nhận **SOPS + age** (cả hai đều mã nguồn mở, 0 đồng):

```bash
age-keygen -o ~/.config/sops/age/keys.txt        # khóa riêng KHÔNG vào repo
sops --encrypt --age <public-key> .env > .env.enc  # .env.enc mới được commit
sops --decrypt .env.enc > .env                   # giải mã khi cần
```

`MUST` ghi rõ trong `README.md` của repo ai giữ khóa riêng và cách xin quyền.

---

## 4. Cổng kiểm mã nguồn

Không có cổng kiểm thì quy ước chỉ là lời khuyên. Mọi repo `MUST` bật đủ 4 lớp sau.

### 4.1. Lớp 1 - chặn trước khi commit (máy lập trình viên)

`MUST` cài `gitleaks` và chạy trước mỗi commit. Cách cài đặt hook không phụ thuộc
ngôn ngữ:

```bash
# Cài gitleaks: https://github.com/gitleaks/gitleaks/releases
mkdir -p .githooks && cat > .githooks/pre-commit <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks protect --staged --redact --config .gitleaks.toml || {
    echo "Phát hiện secret trong nội dung sắp commit. Xử lý xong mới commit lại."
    exit 1
  }
else
  echo "CẢNH BÁO: chưa cài gitleaks - cổng kiểm secret đang tắt." >&2
fi
HOOK
chmod +x .githooks/pre-commit
git config core.hooksPath .githooks
```

### 4.2. Lớp 2 - chặn khi push (GitHub)

- `MUST` bật **Secret Scanning** và **Push Protection** trong `Settings ->
  Code security`. Miễn phí với repo Public.
- `MUST` bật **Branch protection** cho nhánh `main`: cấm push thẳng, bắt buộc PR,
  bắt buộc cổng kiểm CI xanh, cấm force-push, cấm xóa nhánh.

### 4.3. Lớp 3 - kiểm trên CI (mỗi PR)

`MUST` có workflow chạy tối thiểu:

| Kiểm tra | Công cụ | Chặn merge |
| --- | --- | --- |
| Quét secret trên toàn diễn biến PR | gitleaks | Có |
| Phân tích tĩnh tìm lỗ hổng | CodeQL (Public) hoặc Semgrep OSS | Có với mức Cao trở lên |
| Lỗ hổng phụ thuộc | `npm audit` / `pip-audit` / `dotnet list package --vulnerable` / OSV-Scanner | Có với mức Cao trở lên |
| Cổng kiểm quy ước | `scripts/check-standards.sh` | Có |
| Lint + test | Theo ngôn ngữ | Có |

### 4.4. Lớp 4 - giám sát liên tục

- `MUST` bật **Dependabot** (`.github/dependabot.yml`) cho mọi hệ quản lý gói
  đang dùng, cập nhật hằng tuần.
- `MUST` xử lý cảnh báo Nghiêm trọng/Cao trong **7 ngày**, Trung bình trong
  **30 ngày**.
- `SHOULD` chạy quét lại toàn bộ repo hằng tháng theo lịch, không chỉ khi có PR.

---

## 5. Chuỗi cung ứng phần mềm

Phần lớn mã chạy trong sản phẩm là mã của người khác. Đối xử với nó tương xứng.

- `MUST` commit file khóa phiên bản (`package-lock.json`, `pnpm-lock.yaml`,
  `poetry.lock`, `requirements.txt` đã ghim, `packages.lock.json`, `Cargo.lock`).
  Build `MUST` dùng lệnh cài đặt tôn trọng file khóa (`npm ci`, không phải
  `npm install`).
- `MUST` ghim GitHub Action theo **SHA đầy đủ**, không theo nhãn phiên bản:

  ```yaml
  # ĐÚNG - nhãn có thể bị trỏ lại vào commit khác
  - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
  # SAI
  - uses: actions/checkout@v4
  ```

- `MUST` khai `permissions:` tối thiểu ở mỗi workflow. Mặc định của repo `MUST`
  đặt về `Read repository contents`.
- `MUST NOT` dùng `pull_request_target` với việc checkout mã của PR đến, trừ khi
  đã hiểu rõ và có rà soát riêng - đây là lỗ hổng chiếm quyền workflow phổ biến nhất.
- `MUST` rà soát trước khi thêm phụ thuộc mới: gói có được bảo trì không (commit
  gần nhất, số người bảo trì), giấy phép có tương thích không, có gói tương đương
  đã dùng trong project chưa, kích thước có xứng đáng không.
- `MUST NOT` cài gói có tên gần giống gói phổ biến mà chưa kiểm tra kỹ
  (typosquatting), và `MUST NOT` chạy script cài đặt của gói lạ với quyền cao.
- `SHOULD` sinh SBOM (`syft`, hoặc `npm sbom`) cho mỗi bản phát hành.

---

## 6. Xác thực, phiên và phân quyền

> Mục này là **ngưỡng kỹ thuật tối thiểu**. Kiến trúc tài khoản của hệ sinh thái
> (ba lối vào Google/GitHub/tsudev, gộp danh tính, cơ chế Xác minh tài khoản,
> trạng thái và ma trận quyền) nằm ở
> [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md). Làm chức năng đăng nhập thì
> `MUST` đọc cả hai: tài liệu này cho ngưỡng, tài liệu kia cho kiến trúc.

### 6.1. Mật khẩu

- `MUST` băm bằng **Argon2id** (ưu tiên) hoặc **bcrypt** hệ số công việc >= 12.
  `MUST NOT` dùng MD5, SHA-1, SHA-256 trần, hay tự nghĩ ra thuật toán băm.
- `MUST` yêu cầu độ dài tối thiểu 12 ký tự và đối chiếu với danh sách mật khẩu đã
  lộ (`Have I Been Pwned` range API, miễn phí). `MUST NOT` ép quy tắc kiểu "phải
  có ký tự đặc biệt" rồi giới hạn độ dài tối đa thấp.
- `MUST` giới hạn tần suất đăng nhập sai (rate limit theo tài khoản **và** theo IP),
  có độ trễ tăng dần.

### 6.2. Phiên và token

- `MUST` đặt cookie phiên với đủ `HttpOnly`, `Secure`, `SameSite=Lax` (hoặc
  `Strict` khi được).
- `MUST` sinh lại định danh phiên sau khi đăng nhập thành công và sau khi đổi
  quyền (chống session fixation).
- JWT: `MUST` chỉ định thuật toán ở phía máy chủ và từ chối `alg: none`; `MUST`
  kiểm tra `exp`, `iss`, `aud`; access token sống tối đa **15 phút**, refresh
  token có thể thu hồi và xoay vòng.
- `MUST NOT` lưu token trong `localStorage` nếu ứng dụng có bề mặt XSS. Ưu tiên
  cookie `HttpOnly`.
- `MUST` có đường thoát: đăng xuất làm mất hiệu lực phiên ở phía máy chủ, không
  chỉ xóa cookie ở trình duyệt.

### 6.3. Phân quyền

- `MUST` kiểm tra quyền **ở tầng máy chủ, trên từng thao tác**. Ẩn nút trên giao
  diện không phải là phân quyền.
- `MUST` kiểm tra quyền sở hữu đối tượng, không chỉ vai trò. Đây là lỗi
  **IDOR/BOLA** - hạng mục số một trong OWASP API Top 10: người dùng A gọi
  `/api/orders/123` của người dùng B và được trả về.
- `MUST` mặc định từ chối: endpoint mới không khai quyền thì không ai vào được.
- `SHOULD` bật xác thực hai lớp (2FA/TOTP) cho tài khoản quản trị. Với GitHub,
  `MUST` bật 2FA cho mọi thành viên có quyền ghi.

---

## 7. Bảo mật tầng ứng dụng

### 7.1. Đối chiếu OWASP Top 10 (2021)

| Mã | Rủi ro | Biện pháp bắt buộc trong hệ sinh thái tsudev |
| --- | --- | --- |
| A01 | Hỏng kiểm soát truy cập | Mục 6.3. Có test tự động cho ít nhất một ca IDOR |
| A02 | Hỏng mật mã | TLS 1.2+ bắt buộc, băm mật khẩu theo 6.1, mã hóa D2/D3 khi lưu |
| A03 | Tiêm mã (Injection) | Chỉ dùng truy vấn tham số hóa hoặc ORM; sanitize HTML theo `RICH_TEXT_EDITOR.md` mục 5.4 |
| A04 | Thiết kế thiếu an toàn | Rà soát mối đe dọa cho mọi tính năng đụng D2/D3 |
| A05 | Cấu hình sai | Tắt debug ở production, đổi mọi tài khoản mặc định, header theo 7.2 |
| A06 | Thành phần lỗi thời | Dependabot + cổng kiểm mục 4.3 |
| A07 | Hỏng định danh | Mục 6.1, 6.2 |
| A08 | Hỏng toàn vẹn dữ liệu/phần mềm | Ghim SHA cho Action, file khóa phiên bản, mục 5 |
| A09 | Thiếu ghi nhật ký và giám sát | Mục 1.6 và 9.1 |
| A10 | SSRF | Danh sách trắng đích đến cho mọi yêu cầu do người dùng điều khiển; chặn dải IP nội bộ |

### 7.2. Header bảo mật HTTP (bắt buộc cho mọi ứng dụng web)

```
Content-Security-Policy: default-src 'self'; script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
Cross-Origin-Opener-Policy: same-origin
```

- `MUST NOT` dùng `unsafe-inline` hay `unsafe-eval` trong `script-src`. Nếu
  framework cần script nội tuyến, dùng nonce hoặc hash.
- `MUST NOT` đặt `Access-Control-Allow-Origin: *` cho endpoint có xác thực.
- `MUST` kiểm chứng bằng https://securityheaders.com (miễn phí) trước khi ra mắt,
  mục tiêu hạng **A** trở lên.

### 7.3. Đầu vào và đầu ra

- `MUST` kiểm tra đầu vào bằng lược đồ (Zod, Pydantic, FluentValidation, JSON
  Schema) tại **biên** của hệ thống, không rải rác trong logic nghiệp vụ.
- `MUST` mã hóa đầu ra theo đúng ngữ cảnh: HTML, thuộc tính HTML, JavaScript, URL,
  SQL là năm ngữ cảnh khác nhau, không dùng chung một hàm escape.
- `MUST` giới hạn kích thước: độ dài chuỗi, kích thước file tải lên, số phần tử
  mảng, độ sâu JSON. Thiếu giới hạn là cửa cho tấn công cạn tài nguyên.
- `MUST` kiểm tra file tải lên bằng **magic number**, không tin phần mở rộng hay
  `Content-Type` do trình duyệt gửi; lưu ngoài thư mục web root; phục vụ lại qua
  tên do hệ thống sinh, kèm `Content-Disposition: attachment` cho loại không hiển thị.

### 7.4. Giới hạn tần suất và chống lạm dụng

- `MUST` giới hạn tần suất cho: đăng nhập, đăng ký, quên mật khẩu, tìm kiếm, gửi
  biểu mẫu, và mọi endpoint tốn tài nguyên.
- `SHOULD` dùng CAPTCHA không xâm lấn cho biểu mẫu công khai. Lựa chọn 0 đồng:
  **Cloudflare Turnstile**.
- `MUST` trả về `429` kèm `Retry-After` thay vì âm thầm bỏ qua.

---

## 8. Hạ tầng, mạng và triển khai

- `MUST` bắt buộc HTTPS toàn tuyến, chuyển hướng HTTP sang HTTPS, TLS tối thiểu
  1.2. `MUST NOT` tắt kiểm tra chứng chỉ SSL ở bất kỳ đâu, kể cả trong script
  phát triển (`curl -k`, `rejectUnauthorized: false`, `verify=False` đều bị cấm).
- `MUST` chỉ mở cổng thực sự cần. Database `MUST NOT` phơi ra Internet công cộng.
- `MUST` tách môi trường `development`/`staging`/`production` với secret riêng
  biệt hoàn toàn.
- `MUST` sao lưu dữ liệu D2/D3 tự động và **kiểm tra khôi phục ít nhất mỗi quý**.
  Bản sao lưu chưa từng phục hồi thử thì chưa phải bản sao lưu.
- `MUST` bật nhật ký kiểm toán ở tầng nền tảng (ai đăng nhập bảng điều khiển, ai
  đổi cấu hình).
- `MUST` chọn vùng gần Việt Nam: ưu tiên **Singapore**, kế đến **Nhật Bản
  (Tokyo/Osaka)**. Chi tiết: [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md).
- `SHOULD` đặt sau Cloudflare (gói miễn phí) để có WAF cơ bản, chống DDoS lớp
  mạng, và ẩn địa chỉ IP gốc.

---

## 9. Quy trình ứng phó sự cố

### 9.1. Chuẩn bị trước

- `MUST` có nhật ký đủ để trả lời: chuyện gì xảy ra, lúc nào, ai làm, ảnh hưởng
  bản ghi nào. Lưu tối thiểu 90 ngày.
- `MUST` ghi trong `README.md` ai là người liên hệ đầu tiên khi có sự cố.

### 9.2. Khi phát hiện lộ secret

Theo đúng thứ tự này, **không đảo thứ tự**:

1. **Thu hồi/đổi khóa ngay lập tức** tại nhà cung cấp. Xóa lịch sử git trước khi
   thu hồi là sai: khóa vẫn còn hiệu lực trong lúc bạn dọn dẹp.
2. Rà soát nhật ký truy cập của dịch vụ đó tìm dấu hiệu đã bị dùng.
3. Xóa khỏi lịch sử git:
   ```bash
   git filter-repo --path <duong-dan-file> --invert-paths
   git push --force-with-lease --all
   ```
   Sau đó thông báo mọi người xóa bản sao cũ và clone lại.
4. Ghi sự cố vào `logs/STATE.md` mục "Quyết định quan trọng": lộ gì, lộ bao lâu,
   đã làm gì, phòng ngừa ra sao.
5. Nếu secret chạm dữ liệu D2/D3 của người dùng, thực hiện tiếp mục 9.3.

### 9.3. Khi có xâm nhập hoặc rò rỉ dữ liệu

1. **Cô lập**: cắt đường truy cập của kẻ tấn công, giữ nguyên hiện trạng để điều
   tra (chụp ảnh máy/ổ đĩa trước khi xóa bất cứ thứ gì).
2. **Đánh giá**: dữ liệu nào, bao nhiêu bản ghi, mức phân loại nào.
3. **Khắc phục**: vá lỗ hổng, đổi toàn bộ khóa liên quan, buộc đăng xuất tất cả
   phiên.
4. **Thông báo**: nếu có dữ liệu cá nhân, thông báo cho người bị ảnh hưởng và cơ
   quan có thẩm quyền theo Nghị định 13/2023/ND-CP.
5. **Rút kinh nghiệm**: viết báo cáo sau sự cố **không quy trách nhiệm cá nhân**,
   tập trung vào lỗ hổng quy trình. Bổ sung một cổng kiểm tự động để lỗi đó không
   lặp lại - nếu không thêm được cổng kiểm nào, sự cố coi như chưa đóng.

---

## 10. Checklist theo giai đoạn

### 10.1. Khi khởi tạo repo mới

- [ ] Copy `templates/gitignore/base.gitignore` thành `.gitignore`, thêm phần
      dành riêng cho ngôn ngữ.
- [ ] Copy `.env.example`, khai đủ biến, không có giá trị thật.
- [ ] Copy `SECURITY.md` và điều chỉnh địa chỉ liên hệ.
- [ ] Bật Secret Scanning, Push Protection, Dependabot, branch protection cho `main`.
- [ ] Cài hook `gitleaks` theo mục 4.1.
- [ ] Chạy `scripts/sync-standards.sh` để kéo bộ quy ước và ghi `.standards-version`.
- [ ] Phân loại dữ liệu theo mục 2, ghi vào `docs/ARCHITECTURE.md`.

### 10.2. Trước mỗi commit

- [ ] `git status` sạch, không có file lạ ngoài phạm vi task.
- [ ] Không có secret trong diễn biến - kể cả trong comment, log, file test.
- [ ] File nhạy cảm/cache mới tạo đã vào `.gitignore`.
- [ ] Lint và test tối thiểu của task đã qua.
- [ ] Commit message theo `loại(phạm-vi): mô tả ngắn`.

### 10.3. Trước mỗi PR

- [ ] `gitleaks detect --redact` sạch trên toàn nhánh.
- [ ] Không có phụ thuộc mới chưa rà soát theo mục 5.
- [ ] Có test cho phần logic bảo mật đã đụng tới.
- [ ] Nếu thêm endpoint: đã khai quyền, đã giới hạn tần suất, đã kiểm tra quyền
      sở hữu đối tượng.
- [ ] Nếu đụng dữ liệu D2/D3: đã điền phiếu tại
      `docs/templates/SECURITY_REVIEW.md`.

### 10.4. Trước mỗi lần phát hành

- [ ] Không còn lỗ hổng phụ thuộc mức Cao trở lên.
- [ ] Header bảo mật đạt hạng A trên securityheaders.com.
- [ ] Debug/stack trace đã tắt ở production; thông báo lỗi không lộ chi tiết nội bộ.
- [ ] Đã kiểm tra khôi phục từ bản sao lưu trong vòng 90 ngày qua.
- [ ] Đã ghi một dòng vào `CHANGELOG.md`.

---

## 11. Bộ công cụ 0 đồng

Toàn bộ chuẩn trên thực hiện được với chi phí **0 đồng**. Không có mục nào ở đây
yêu cầu gói trả phí.

| Nhu cầu | Công cụ miễn phí | Ghi chú |
| --- | --- | --- |
| Quét secret | **gitleaks** | Mã nguồn mở, chạy cục bộ và trên CI |
| Chặn secret khi push | **GitHub Push Protection** | Miễn phí với repo Public |
| Phân tích tĩnh (SAST) | **CodeQL** (Public) hoặc **Semgrep OSS** | Semgrep OSS chạy được cả với repo Private |
| Lỗ hổng phụ thuộc | **Dependabot**, **OSV-Scanner**, `npm audit`, `pip-audit` | Đều 0 đồng |
| Kiểm tra header | **securityheaders.com**, **Mozilla Observatory** | Miễn phí, không cần tài khoản |
| Kiểm tra TLS | **SSL Labs Server Test** | Miễn phí |
| Quét lỗ hổng web động | **OWASP ZAP** | Mã nguồn mở |
| WAF + chống DDoS | **Cloudflare Free** | Kèm SSL và ẩn IP gốc |
| CAPTCHA | **Cloudflare Turnstile** | Miễn phí, không theo dõi người dùng |
| Mã hóa secret trong repo | **SOPS + age** | Cả hai mã nguồn mở |
| Đối chiếu mật khẩu đã lộ | **Have I Been Pwned Range API** | Miễn phí, dùng k-anonymity nên không gửi mật khẩu đi |
| Giám sát lỗi | **Sentry** gói miễn phí, hoặc **GlitchTip** tự vận hành | 5.000 sự kiện/tháng ở gói miễn phí |

---

## 12. Yêu cầu riêng cho agent AI

Agent AI (Claude, Copilot, Cursor, hay agent tự động khác) khi làm việc trên bất
kỳ repo tsudev nào `MUST`:

1. **Đọc file này trước khi viết dòng mã đầu tiên** đụng tới xác thực, phân
   quyền, tải file lên, truy vấn database, hay xử lý dữ liệu người dùng.
2. **Không bao giờ in secret ra hội thoại**, kể cả khi đọc trúng file `.env`.
   Nếu vô tình đọc phải, báo cho chủ project rằng file đó tồn tại, không trích
   nội dung.
3. **Không tự ý nới lỏng bảo mật để cho code chạy được.** Cấm tuyệt đối các cách
   "chữa cháy": tắt kiểm tra SSL, đặt CORS `*`, thêm `unsafe-inline` vào CSP, bỏ
   qua kiểm tra quyền, `catch` nuốt lỗi ở luồng xác thực. Gặp bế tắc thì báo cáo
   và đề xuất, không tự quyết.
4. **Bổ sung `.gitignore` ngay khi tạo ra file nhạy cảm**, trước khi commit,
   không đợi ai nhắc.
5. **Khai báo rõ khi đề xuất phụ thuộc mới**: tên gói, vì sao cần, có gói tương
   đương trong project chưa, giấy phép gì.
6. **Khi được yêu cầu làm nhanh, vẫn không bỏ bước bảo mật.** "Làm nhanh" rút gọn
   phạm vi tính năng, không rút gọn cổng kiểm.
7. **Khi phát hiện lỗ hổng trong mã có sẵn**, báo cáo ngay cho chủ project kèm
   đánh giá mức độ theo `SECURITY.md` mục 5, kể cả khi nó nằm ngoài task đang làm.

---

## 13. Tài liệu liên quan

- [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md) - kiến trúc tài khoản, ba lối vào, xác minh
- [`ECOSYSTEM_IDENTITY.md`](ECOSYSTEM_IDENTITY.md) mục 6 - xử lý ảnh tải lên an toàn
- [`DATA_TABLE.md`](DATA_TABLE.md) mục 8.4 - trần `page_size` và chống lạm dụng
- [`../SECURITY.md`](../SECURITY.md) - chính sách và kênh báo cáo lỗ hổng
- [`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md) - quy tắc duy trì `.gitignore`
- [`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) - nhánh, commit, PR, phát hành
- [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md) - nhà cung cấp và hạn mức 0 đồng
- [`RICH_TEXT_EDITOR.md`](RICH_TEXT_EDITOR.md) mục 5.4 - sanitize nội dung soạn thảo
- [`templates/SECURITY_REVIEW.md`](templates/SECURITY_REVIEW.md) - phiếu rà soát bảo mật
