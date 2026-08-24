# QUY TẮC DUY TRÌ `.gitignore` - mọi repo tsudev (v2.0.0)

> `.gitignore` là hàng rào bảo mật rẻ nhất và bị bỏ quên nhiều nhất. Một dòng
> thiếu ở đây tốn nhiều công hơn mọi thứ khác trong bộ quy ước này.

## 1. Nguồn chân lý

Bản chuẩn nằm tại `templates/gitignore/base.gitignore` của repo
`tsudev-standards`. Mọi repo con `MUST` chứa **toàn bộ** nội dung bản chuẩn.

- **Được phép THÊM** dòng cho nhu cầu riêng của repo.
- **KHÔNG được phép XÓA BỚT** dòng nào của bản chuẩn. Muốn bỏ một dòng thì phải
  đề xuất ngược lên trung tâm qua `proposals/`, không tự bỏ ở repo con.
- Bản chuẩn được kiểm tự động bằng `scripts/check-standards.sh`.

## 2. Cách ghép cho từng loại project

```bash
# Bước 1: lấy phần nền
cat .standards/templates/gitignore/base.gitignore > .gitignore

# Bước 2: nối thêm phần theo ngôn ngữ (chọn đúng file)
cat .standards/templates/gitignore/node.gitignore   >> .gitignore   # Node/TypeScript
cat .standards/templates/gitignore/python.gitignore >> .gitignore   # Python
cat .standards/templates/gitignore/dotnet.gitignore >> .gitignore   # C#/.NET
cat .standards/templates/gitignore/cpp.gitignore    >> .gitignore   # C/C++
cat .standards/templates/gitignore/rust.gitignore   >> .gitignore   # Rust / Tauri
cat .standards/templates/gitignore/go.gitignore     >> .gitignore   # Go
cat .standards/templates/gitignore/java.gitignore  >> .gitignore   # Java (Maven/Gradle)
cat .standards/templates/gitignore/mobile.gitignore >> .gitignore   # Flutter/RN/Android/iOS

# Bước 3: thêm phần riêng của repo dưới tiêu đề rõ ràng
printf '\n# --- Riêng của repo này ---\n' >> .gitignore
```

## 3. Mười nhóm của bản chuẩn

| Nhóm | Bảo vệ khỏi điều gì |
| --- | --- |
| 1. Biến môi trường & cấu hình nhạy cảm | Lộ chuỗi kết nối, khóa dịch vụ |
| 2. Khóa, chứng chỉ, token | Lộ khóa riêng, khóa ký ứng dụng, khóa SSH |
| 3. Hạ tầng dưới dạng mã | `*.tfstate` chứa secret ở dạng văn bản thuần |
| 4. Bản build & bản phát hành | Repo phình to, phát tán bản build cũ có lỗ hổng |
| 5. Phụ thuộc & cache | Repo phình to, xung đột giữa các máy |
| 6. Log & file tạm | Log ứng dụng thường chứa token phiên và dữ liệu cá nhân |
| 7. IDE, công cụ & hệ điều hành | Lộ đường dẫn cục bộ, cấu hình cá nhân |
| 8. Công cụ AI & agent | `.claude/settings.local.json` và tương đương có thể chứa khóa |
| 9. Dữ liệu cục bộ, cá nhân & kết quả kiểm thử | Commit nhầm database thật có dữ liệu người dùng |
| 10. Dấu vết điều tra / gỡ lỗi | `.har` chứa nguyên cookie phiên và header xác thực |

## 4. Quy tắc vận hành

1. **Hễ tạo ra file nhạy cảm thì bổ sung ngay**, trước khi commit, không đợi
   nhắc. Đây là mục 3 của `AGENTS.md`.
2. `.gitignore` **không dọn được thứ đã theo dõi**. Nếu file đã bị commit trước
   đó, thêm dòng ignore là vô nghĩa:
   ```bash
   git rm --cached <đường-dẫn>    # gỡ khỏi chỉ mục, giữ file trên đĩa
   ```
   Và nếu file đó chứa secret, xử lý theo `SECURITY_BASELINE.md` mục 9.2 - thu
   hồi khóa trước, dọn lịch sử sau.
3. **Kiểm tra một đường dẫn có bị bỏ qua không**:
   ```bash
   git check-ignore -v <đường-dẫn>
   ```
4. **Không dùng `.gitignore` toàn cục của máy cá nhân** để che file chuẩn phải có
   trong repo. Người khác clone về sẽ không có lớp che đó.
5. Với file cần giữ trong repo nhưng tên trùng mẫu bị chặn, dùng dấu `!` để mở
   lại và **ghi chú lý do ngay dòng trên**.

## 5. Bốn sai lầm hay gặp

| Sai lầm | Hậu quả | Cách đúng |
| --- | --- | --- |
| Thêm `.gitignore` sau khi đã commit secret | Secret vẫn nằm trong lịch sử, ai clone cũng thấy | Thu hồi khóa, `git filter-repo`, xem `SECURITY_BASELINE.md` 9.2 |
| Chặn cả thư mục `config/` | Chặn luôn file cấu hình mẫu cần commit | Chặn `config/*.local.*`, giữ `config/default.json` |
| Chặn cả thư mục `logs/` cho nhật ký chạy | Chặn luôn `logs/STATE.md` và `logs/LOCKS.md` - **thư mục điều phối phiên bắt buộc phải commit** | Chặn `logs/*.log`, `logs/*.tmp`; nhật ký chạy của ứng dụng nên nằm ở thư mục dữ liệu của hệ điều hành, không nằm trong repo |
| Dùng `*.env` thay vì `.env*` | Bỏ sót `.env.production`, `.env.local` | Dùng đúng bản chuẩn |
| Xóa bớt dòng của bản chuẩn cho "gọn" | Repo lệch chuẩn, lần sau không ai biết vì sao | Đề xuất qua `proposals/` |

## 6. Miễn trừ có ghi chép: `.standards-allow`

Đôi khi một file khớp mẫu nhạy cảm nhưng thật sự không chứa secret - ví dụ
`.env.production` của một app Next.js chỉ mang biến `NEXT_PUBLIC_*`, vốn được
biên dịch thẳng vào bundle trình duyệt nên đã công khai theo thiết kế.

Với trường hợp đó, khai vào `.standards-allow` ở gốc repo, **đủ ba cột**:

```
<đường dẫn> | <lý do> | <hết hiệu lực DD/MM/YYYY hoặc điều kiện>
```

Ví dụ:

```
apps/web/.env.production | Chỉ chứa NEXT_PUBLIC_*, đã công khai theo thiết kế | 31/12/2026
```

**Ba ràng buộc, cố ý làm cho việc miễn trừ khó chịu vừa đủ:**

1. **Thiếu lý do hoặc thiếu hạn thì cổng kiểm vẫn chặn.** Ngoại lệ vô thời hạn
   không phải ngoại lệ, đó là quy ước ngầm chưa được viết ra.
2. **Mỗi lần chạy, dòng miễn trừ được in ra dưới dạng `LƯU Ý`.** Nó không bao
   giờ trở nên vô hình - đó là điểm khác biệt với việc lặng lẽ nới mẫu chặn.
3. **`MUST NOT` dùng để tắt cảnh báo cho file có secret thật.** Việc đó xử lý
   theo [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 9.2: thu hồi khóa
   trước, dọn lịch sử sau.

**Rủi ro phải biết trước khi dùng:** miễn trừ một file `.env*` nghĩa là người
sau có thể thêm một biến thật sự bí mật vào chính file đó và nó sẽ được commit
mà không ai báo. Vì vậy miễn trừ luôn là giải pháp **tạm**, và cột hạn tồn tại
để buộc nhìn lại. Cách sửa dứt điểm luôn là đưa giá trị đó ra khỏi file `.env*`.

Mẫu đầy đủ: `templates/standards-allow.example`.

## 7. Cổng kiểm

`scripts/check-standards.sh` kiểm tra:

- Repo có `.gitignore` không.
- `.gitignore` có chứa đủ mọi dòng của bản chuẩn không.
- Cây làm việc có file nào khớp mẫu nhạy cảm mà **đang được git theo dõi** không
  (trừ những gì đã khai ở `.standards-allow` theo mục 6).

Cổng này `MUST` chạy trong CI của mọi repo con và chặn merge khi thất bại.
