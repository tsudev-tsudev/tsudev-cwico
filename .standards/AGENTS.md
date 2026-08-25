# AGENTS.md - Bộ quy ước dùng chung tsudev (v3.1.1)

> **ĐÂY LÀ NGUỒN CHÂN LÝ.** Bộ quy ước được biên tập tại repo
> `tsudev-tsudev/tsudev-standards`. Repo con mang một BẢN SAO CHỈ-ĐỌC tại
> `.standards/` và đồng bộ xuống từ đây. Sửa quy ước thì sửa ở đây rồi mới đồng
> bộ xuống, **KHÔNG sửa ngược**. Cơ chế: [`docs/SYNC.md`](docs/SYNC.md).
>
> Repo con chèn phần phân vai riêng vào mục "Phần B" trong bản `AGENTS.md` của
> nó; phần đó không thuộc bộ quy ước và không tồn tại ở đây.

---

> **ĐỌC FILE NÀY ĐẦU TIÊN trong mọi phiên làm việc mới.** Áp dụng cho toàn bộ
> project, cho cả lập trình viên và agent AI.
>
> Các file quy ước (`AGENTS.md`, `SECURITY.md`, `docs/*`, `tokens/*`,
> `templates/*`, `.gitignore`) là **BẤT KHẢ XÂM PHẠM**: chỉ đọc-hiểu-tuân thủ,
> KHÔNG sửa/xóa trừ khi chủ project yêu cầu trực tiếp.

## 0. Câu lệnh khởi động phiên

Dán nguyên khối này vào đầu mọi phiên terminal mới:

```
Đọc AGENTS.md, logs/STATE.md và phiếu bàn giao mới nhất trong logs/handover/.
Tuân thủ toàn bộ quy ước. Nhận task tiếp theo trong hàng đợi của STATE.md,
khóa file mình sẽ sửa vào logs/LOCKS.md rồi mới bắt đầu. Trả lời ngắn gọn,
tiết kiệm token, không lặp lại nội dung đã có trong file quy ước.
```

Bộ câu lệnh đầy đủ cho mọi tình huống: [`docs/ONBOARDING.md`](docs/ONBOARDING.md) phần 2.

## 1. Bản đồ quy ước

File này là **điểm vào**, giữ ngắn có chủ đích. Chi tiết nằm ở các tài liệu
chuyên đề. Bản đồ đầy đủ: [`docs/00-INDEX.md`](docs/00-INDEX.md).

| Cần gì | Đọc |
| --- | --- |
| Quy trình phiên, khóa file, bàn giao | [`docs/AGENT_PROTOCOL.md`](docs/AGENT_PROTOCOL.md) |
| **Bảo mật bắt buộc** | [`docs/SECURITY_BASELINE.md`](docs/SECURITY_BASELINE.md) |
| Quy tắc `.gitignore` | [`docs/GITIGNORE_POLICY.md`](docs/GITIGNORE_POLICY.md) |
| Nhánh, commit, PR, phát hành | [`docs/GIT_WORKFLOW.md`](docs/GIT_WORKFLOW.md) |
| Hạ tầng 0 đồng | [`docs/FREE_TIER_STACK.md`](docs/FREE_TIER_STACK.md) |
| Chọn ngôn ngữ, framework | [`docs/LANGUAGE_SELECTION.md`](docs/LANGUAGE_SELECTION.md) |
| **Đăng nhập, đăng ký, xác minh tài khoản** | [`docs/AUTH_AND_ACCOUNT.md`](docs/AUTH_AND_ACCOUNT.md) |
| Giao diện, token, component | [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md) |
| Bảng bản ghi, bộ chọn số bản ghi | [`docs/DATA_TABLE.md`](docs/DATA_TABLE.md) |
| Logo, favicon, icon ứng dụng | [`docs/BRAND_ASSETS.md`](docs/BRAND_ASSETS.md) |
| tsudev.com, ảnh đại diện, trang hồ sơ | [`docs/ECOSYSTEM_IDENTITY.md`](docs/ECOSYSTEM_IDENTITY.md) |
| Cấu trúc thư mục | [`docs/PROJECT_STRUCTURE.md`](docs/PROJECT_STRUCTURE.md) |
| Khả năng truy cập | [`docs/ACCESSIBILITY.md`](docs/ACCESSIBILITY.md) |
| Kiểm thử và chất lượng mã | [`docs/TESTING_QUALITY.md`](docs/TESTING_QUALITY.md) |
| Trình soạn thảo nội dung | [`docs/RICH_TEXT_EDITOR.md`](docs/RICH_TEXT_EDITOR.md) |
| Tìm kiếm và lọc tiếng Việt | [`docs/SEARCH_AND_FILTER.md`](docs/SEARCH_AND_FILTER.md) |
| Phiên bản và CHANGELOG | [`docs/VERSIONING.md`](docs/VERSIONING.md) |
| Đồng bộ quy ước xuống repo con | [`docs/SYNC.md`](docs/SYNC.md) |

## 2. Tiết kiệm token và ngữ cảnh

- Không đọc lại file đã nắm nội dung trong cùng phiên; không in toàn bộ file dài
  ra hội thoại - chỉ trích phần liên quan.
- Trả lời và ghi log **ngắn gọn, gạch đầu dòng, không văn mẫu**. Không lặp lại
  quy ước đã có sẵn trong tài liệu.
- Tri thức dùng lại được (quyết định kiến trúc, cách chạy build, lỗi đã gặp) ghi
  vào file markdown tương ứng **một lần duy nhất**; các phiên sau chỉ tham chiếu
  đường dẫn.
- Task lớn phải chia nhỏ; làm xong phần nào chốt phần đó vào log ngay, để mất
  phiên không mất công.

## 3. Đội ngũ agent và chống giẫm chân

- Mỗi agent nhận **một nhiệm vụ duy nhất** tại một thời điểm, ghi rõ trong
  `logs/STATE.md` mục "Đang thực hiện".
- **Trước khi sửa bất kỳ file nào**: kiểm tra `logs/LOCKS.md`. File chưa ai khóa
  thì thêm dòng `<đường dẫn> | <agent/nhiệm vụ> | <HH:mm DD/MM/YYYY>` rồi mới
  sửa. Sửa xong thì xóa dòng khóa ngay.
- File **đang bị agent khác khóa**: TUYỆT ĐỐI không sửa. Tạo phiếu bàn giao tại
  `logs/handover/` theo mẫu [`docs/templates/HANDOVER.md`](docs/templates/HANDOVER.md).
- Agent đang giữ khóa **có trách nhiệm đọc phiếu gửi đến mình trước khi nhả
  khóa** và ghi kết quả vào chính phiếu đó.
- Không bao giờ can thiệp nhiệm vụ, nhánh git, hay file của agent khác.

Chi tiết: [`docs/AGENT_PROTOCOL.md`](docs/AGENT_PROTOCOL.md).

## 4. Bảo mật - checklist trước mọi commit / PR / merge / deploy / push

> Chuẩn đầy đủ: [`docs/SECURITY_BASELINE.md`](docs/SECURITY_BASELINE.md).
> Đây chỉ là phần phải thuộc lòng.

1. `git status` sạch - không có file lạ ngoài phạm vi task.
2. Không có secret, khóa API, token, mật khẩu, chuỗi kết nối trong diễn biến -
   kể cả trong comment, log, và file test.
3. File mới thuộc nhóm nhạy cảm hoặc cache đã vào `.gitignore`.
4. Build và test tối thiểu của task đã qua.
5. Commit message dạng `loại(phạm-vi): mô tả ngắn`, ví dụ `fix(auth): sửa hết
   hạn token`.

**Quy tắc cứng:**

- `.gitignore` gốc của repo là chuẩn tối thiểu. Hễ **tạo ra** file/thư mục chứa
  secret, credential, cache, build output, hay dữ liệu cá nhân thì **bổ sung
  ngay vào `.gitignore` trước khi commit**.
- Lỡ commit secret = **secret đã lộ**. Thu hồi hoặc đổi khóa NGAY (đó là bước
  một, không phải bước xóa lịch sử git), rồi làm tiếp theo
  [`docs/SECURITY_BASELINE.md`](docs/SECURITY_BASELINE.md) mục 9.2.
- Không tắt kiểm tra HTTPS/SSL, không hạ cấp thuật toán mã hóa, không mở cổng
  hay quyền rộng hơn mức task cần.
- Không nới lỏng bảo mật để cho mã chạy được. Gặp bế tắc thì báo cáo, không tự quyết.

## 5. Chi phí 0 đồng

- **Luôn ưu tiên gói miễn phí** trước khi cân nhắc trả phí. Chỉ đề xuất trả phí
  khi đã chứng minh gói miễn phí không đủ, **kèm số liệu đo được**.
- Chọn **vùng gần Việt Nam**: ưu tiên Singapore, kế đến Nhật Bản (Tokyo/Osaka)
  cho mọi dịch vụ cho chọn vùng.
- Tận dụng cache và CDN miễn phí, nén tài nguyên, tránh hỏi liên tục (polling).
- Trước khi thêm phụ thuộc hoặc dịch vụ mới: kiểm tra project đã có thứ tương
  đương chưa; ưu tiên thư viện nhẹ, mã nguồn mở.

Bảng nhà cung cấp và hạn mức: [`docs/FREE_TIER_STACK.md`](docs/FREE_TIER_STACK.md).

## 6. Kết thúc phiên và bàn giao

Khi (a) hàng đợi trong `logs/STATE.md` cạn, (b) được yêu cầu bàn giao, hoặc
(c) ngữ cảnh sắp cạn:

1. Ghi vào `logs/STATE.md`: việc đã làm, việc dang dở **kèm bước tiếp theo cụ
   thể**, quyết định quan trọng.
2. Tạo phiếu bàn giao `logs/handover/YYYYMMDD-NN_<chủ-đề>.md` theo mẫu, đủ để
   phiên sau làm tiếp **không cần hỏi lại**, kể cả khi máy tắt đột ngột.
3. Nhả toàn bộ khóa của mình trong `logs/LOCKS.md`.
4. Dọn tàn dư: xóa file tạm, không để thay đổi chưa commit nằm ngoài phạm vi
   bàn giao.
5. Đề xuất chủ project đóng terminal và mở phiên mới cho task kế tiếp.

## 7. Quy ước giao diện và văn bản

### 7.1. Gạch ngang: chỉ dùng `-`

**Mọi lập trình viên và mọi agent AI bắt buộc đọc-hiểu-tuân thủ-thực hiện mục
này.** Đây là quy tắc mức `MUST`, không phải khuyến nghị.

- **Chỉ dùng gạch ngang ngắn `-` (hyphen, U+002D)** trong MỌI văn bản do người
  hoặc agent sinh ra: mã nguồn, comment, chuỗi hiển thị, tài liệu, log, commit
  message, tiêu đề PR, nội dung issue, phiếu bàn giao, tên file, tên nhánh, tên
  khóa cấu hình, và **cả câu trả lời của agent trong hội thoại**.
- **`MUST NOT` dùng em-dash (U+2014)** ở bất kỳ đâu. Nó phá tính thống nhất của
  giao diện, khó gõ trên bàn phím Việt, và hiển thị sai ở một số phông đơn cách.
  Nơi cần ngắt ý, dùng `-` có khoảng trắng hai bên, hoặc tách thành hai câu.
- **En-dash (U+2013)** `SHOULD NOT` dùng. Khoảng số viết `3-5`, `14-15px`.
- Quy tắc này áp cho cả **agent AI sinh văn bản tự do**. Model quen sinh em-dash
  theo thói quen huấn luyện; ở hệ sinh thái này, thói quen đó `MUST` bị ghi đè.
  Nghi ngờ thì gõ `-`.

**Hai ngoại lệ, chỉ hai:**

1. **File migration đã áp dụng là bất biến** - không sửa kể cả gạch ngang trong
   comment, vì sửa làm lệch checksum của công cụ di trú.
2. **Dòng trích dẫn chính ký tự này để định nghĩa quy tắc** thì `MUST` ghi kèm mã
   điểm `U+2014` trên cùng dòng - cổng kiểm nhận diện miễn trừ theo đúng dấu này.

Ngoài hai ca trên, không có ngoại lệ nào khác. Comment trong mã **không** là
ngoại lệ: agent đọc comment cũng đọc `-` tốt như đọc bất kỳ ký tự nào khác, nên
lý do "dùng em-dash cho agent dễ đọc" không đứng vững.

**Cổng kiểm:** mục 4 của `scripts/check-standards.sh` quét toàn bộ file được git
theo dõi thuộc mọi đuôi văn bản và mã nguồn. Vi phạm = cổng kiểm đỏ = không merge
được. Sửa nhanh toàn repo:

```bash
# Xem truoc pham vi anh huong
git grep -lP '\x{2014}' -- . | grep -v '^migrations/'
```

### 7.2. Các quy ước khác

- Mọi thay đổi giao diện phải dùng token trong `tokens/` - **cấm hard-code màu,
  cỡ chữ, radius**. Chi tiết: [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md).
- Mọi vùng hiển thị danh sách bản ghi phải có bộ chọn số bản ghi đúng vị trí và
  đúng các mốc `10/20/50/100/200`, mặc định `10`. Chi tiết:
  [`docs/DATA_TABLE.md`](docs/DATA_TABLE.md).
- Mọi sản phẩm có tài khoản phải đủ ba lối vào (Google, GitHub, tài khoản tsudev)
  và cơ chế Xác minh tài khoản. Chi tiết:
  [`docs/AUTH_AND_ACCOUNT.md`](docs/AUTH_AND_ACCOUNT.md).
- Mọi sản phẩm phải đủ bộ logo/favicon/icon và siêu dữ liệu nối về `tsudev.com`.
  Chi tiết: [`docs/BRAND_ASSETS.md`](docs/BRAND_ASSETS.md) mục 12 và
  [`docs/ECOSYSTEM_IDENTITY.md`](docs/ECOSYSTEM_IDENTITY.md).
- Ngày hiển thị dạng số `DD/MM/YYYY`, ngày giờ `HH:mm DD/MM/YYYY`.
- Tên bản phát hành app/tool theo [`docs/VERSIONING.md`](docs/VERSIONING.md) mục 2.
- Tạo file mới phải đặt đúng vị trí theo
  [`docs/PROJECT_STRUCTURE.md`](docs/PROJECT_STRUCTURE.md).

## 8. Cổng kiểm

Mọi repo `MUST` chạy được và chạy đạt:

```bash
./scripts/check-standards.sh      # cổng kiểm quy ước
./scripts/sync-standards.sh --check   # đối chiếu với bản trung tâm (repo con)
```

CI `MUST` chặn merge khi cổng kiểm thất bại. Không tắt cổng kiểm để merge cho
kịp - cổng kiểm đỏ nghĩa là chưa xong.

`scripts/check-standards.sh` **thuộc bộ quy ước**, không phải file riêng của repo
con: mỗi lần đồng bộ nó được ghi đè theo bản trung tâm, và `--check` bắt lỗi nếu
nó đã cũ đi. Sửa cổng kiểm thì sửa ở trung tâm. Chi tiết:
[`docs/SYNC.md`](docs/SYNC.md) mục 1 và mục 5.

`--check` đối chiếu với **nhãn repo đang ghim**, không phải với `main`. Nó trả
lời "bản sao của tôi có bị sửa trộm không", không trả lời "tôi đã nâng cấp
chưa" - câu sau là việc của PR định kỳ ở `docs/SYNC.md` mục 5.2.
