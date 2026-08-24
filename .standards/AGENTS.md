# AGENTS.md - Bộ quy ước dùng chung tsudev (v2.0.0)

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
| Giao diện, token, component | [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md) |
| Logo, favicon, icon ứng dụng | [`docs/BRAND_ASSETS.md`](docs/BRAND_ASSETS.md) |
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

- Mọi thay đổi giao diện phải dùng token trong `tokens/` - **cấm hard-code màu,
  cỡ chữ, radius**. Chi tiết: [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md).
- **Chỉ dùng gạch ngang ngắn `-` (hyphen, U+002D)** trong MỌI văn bản: mã,
  comment, chuỗi hiển thị, tài liệu, log, commit message. **Không dùng em-dash
  (U+2014)** - nó phá tính thống nhất của giao diện và khó gõ. En-dash (U+2013)
  chỉ chấp nhận cho khoảng số (`3-5`, `14-15px`) và cũng nên ưu tiên `-`.
  - Ngoại lệ 1: file migration đã áp dụng là bất biến, không sửa kể cả gạch
    ngang trong comment (lệch checksum).
  - Ngoại lệ 2: dòng nào trích dẫn chính ký tự này để định nghĩa quy tắc thì
    phải ghi kèm mã điểm `U+2014` trên cùng dòng - cổng kiểm nhận diện theo dấu này.
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
