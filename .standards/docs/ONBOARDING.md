# TRIỂN KHAI BỘ QUY ƯỚC VÀ BỘ CÂU LỆNH CHUẨN CHO AGENT (v2.0.0)

> Tài liệu hướng dẫn thực hành: đưa bộ quy ước vào một repo, và bộ câu lệnh dán
> sẵn để điều khiển agent AI theo đúng quy ước.
>
> Quy trình phiên làm việc chi tiết: [`AGENT_PROTOCOL.md`](AGENT_PROTOCOL.md).
> Cơ chế đồng bộ: [`SYNC.md`](SYNC.md).

---

## Phần 1 - Đưa bộ quy ước vào repo

### 1.1. Nguyên tắc

Repo con **không chép tay** file quy ước. Repo con chạy `sync-standards.sh` để
lấy bản sao chỉ-đọc vào `.standards/`, rồi tự tạo các file của riêng mình dựa
trên mẫu trong đó. Nhờ vậy repo con luôn biết mình đang ở bản quy ước nào và
biết ngay khi lệch.

### 1.2. Cây thư mục trước và sau

**Trước:**

```
<repo-cua-ban>/
├── src/
├── package.json
└── .gitignore          # bản riêng, có thể thiếu nhiều mục
```

**Sau:**

```
<repo-cua-ban>/
├── .standards/                 # BẢN SAO CHỈ-ĐỌC, sinh bởi script, được commit
│   ├── AGENTS.md
│   ├── VERSION
│   ├── SECURITY.md
│   ├── MANIFEST.sha256
│   ├── docs/
│   ├── tokens/
│   └── templates/
├── .standards-version          # ghi đã đồng bộ từ đâu, bản nào, lúc nào
├── AGENTS.md                   # trỏ về .standards/AGENTS.md + phần phân vai riêng
├── SECURITY.md                 # từ .standards/SECURITY.md, sửa địa chỉ liên hệ
├── .gitignore                  # base + phần theo ngôn ngữ + phần riêng
├── .env.example
├── docs/
│   └── ARCHITECTURE.md         # quyết định kiến trúc CỦA RIÊNG repo này
├── logs/
│   ├── STATE.md
│   ├── LOCKS.md
│   └── handover/
├── scripts/
│   ├── sync-standards.sh       # lấy một lần lúc cài đặt, tự cảnh báo khi cũ
│   └── check-standards.sh      # thuộc bộ quy ước, mỗi lần đồng bộ ghi đè lại
└── src/
```

### 1.3. Câu lệnh triển khai

Xem [`SYNC.md`](SYNC.md) mục 3 - toàn bộ các bước đã viết sẵn thành lệnh dán được.

> **Bẫy hay gặp:** file trong `.standards/` là chỉ-đọc, và `cp` giữ nguyên quyền đó.
> Sau khi chép mẫu `logs/STATE.md` và `logs/LOCKS.md` ra, `MUST` chạy
> `chmod u+w logs/STATE.md logs/LOCKS.md` - nếu không, agent sẽ không ghi được
> trạng thái ở cuối phiên và phiên sau mất toàn bộ ngữ cảnh bàn giao.

### 1.4. Xử lý `.gitignore` khi repo đã có sẵn

`MUST NOT` ghi đè. `MUST` **gộp**: giữ nguyên phần cũ, thêm những dòng của bản
chuẩn còn thiếu.

```bash
# Xem những dòng chuẩn mà repo đang thiếu
comm -23 \
  <(grep -vE '^\s*(#|$)' .standards/templates/gitignore/base.gitignore | sort -u) \
  <(grep -vE '^\s*(#|$)' .gitignore | sort -u)
```

Rồi nối phần thiếu vào cuối `.gitignore` dưới một tiêu đề rõ ràng. Sau đó chạy
`./scripts/check-standards.sh` để xác nhận đã đủ.

### 1.5. Hai file cần điều chỉnh riêng

| File | Cách xử lý |
| --- | --- |
| `AGENTS.md` ở gốc repo | `MUST` là file ngắn: trỏ về `.standards/AGENTS.md` cho phần quy ước chung, rồi thêm "Phần B - phân vai và quy tắc riêng của repo này". Mẫu: `.standards/templates/AGENTS.downstream.md` |
| `SECURITY.md` | Chép từ `.standards/SECURITY.md`, chỉ sửa mục 1 (phạm vi) và mục 3 (địa chỉ liên hệ) cho đúng repo |

### 1.6. Khi hệ sinh thái có từ 2 repo trở lên

`MUST` dùng cơ chế đồng bộ ở [`SYNC.md`](SYNC.md), không chép tay giữa các repo.
`SHOULD` bật thêm workflow tự mở PR nâng cấp hằng tuần (`SYNC.md` mục 5.2) để
không repo nào bị bỏ lại phía sau.

---

## Phần 2 - Bộ câu lệnh chuẩn cho agent AI

> Cách dùng: chép nguyên khối trong khung, dán vào đầu tin nhắn gửi agent, thay
> phần `<...>` bằng nội dung thật. Các lệnh viết theo tiêu chí ngắn gọn, đủ ý,
> tiết kiệm token; dùng được cho mọi agent (Claude Code, Cursor, Copilot,
> Windsurf và tương đương).

### 2.1. Lệnh mở phiên (bắt buộc, dán đầu tiên trong MỌI phiên terminal mới)

```
Đọc AGENTS.md, logs/STATE.md và phiếu bàn giao mới nhất trong logs/handover/.
Tuân thủ toàn bộ quy ước. Nhận task tiếp theo trong hàng đợi của STATE.md,
khóa file mình sẽ sửa vào logs/LOCKS.md rồi mới bắt đầu. Trả lời ngắn gọn,
tiết kiệm token, không lặp lại nội dung đã có trong file quy ước.
```

Biến thể khi muốn **giao task cụ thể** thay vì để agent tự lấy từ hàng đợi:

```
Đọc AGENTS.md, logs/STATE.md và phiếu bàn giao mới nhất trong logs/handover/.
Tuân thủ toàn bộ quy ước. Task của phiên này: <mô tả task cụ thể>.
Ghi task vào mục "Đang thực hiện" của STATE.md, khóa file sẽ sửa vào
logs/LOCKS.md rồi mới bắt đầu. Trả lời ngắn gọn, tiết kiệm token.
```

### 2.2. Lệnh kết thúc phiên và bàn giao

```
Kết thúc phiên theo AGENT_PROTOCOL.md mục 5:
1. Cập nhật logs/STATE.md: việc đã làm, việc dang dở kèm bước tiếp theo cụ thể.
2. Tạo phiếu bàn giao logs/handover/ theo mẫu docs/templates/HANDOVER.md,
   đủ chi tiết để phiên sau làm tiếp không cần hỏi lại.
3. Nhả toàn bộ khóa của mình trong logs/LOCKS.md.
4. Dọn file tạm và tàn dư, chuẩn hóa cây làm việc sạch.
Xong thì báo "ĐÃ BÀN GIAO" kèm tên phiếu, và dừng - không nhận task mới.
```

### 2.3. Lệnh giữa phiên

**a) Giao thêm task khi agent còn rảnh trong phiên:**

```
Task tiếp theo: <mô tả>. Vẫn tuân thủ AGENTS.md: cập nhật STATE.md,
kiểm tra và khóa file trong LOCKS.md trước khi sửa.
```

**b) Sửa giao diện (chống hard-code):**

```
<mô tả thay đổi giao diện>. Bắt buộc dùng token trong .standards/tokens/tokens.css
(hoặc đọc .standards/tokens/design-tokens.json), theo docs/DESIGN_SYSTEM.md.
Cấm hard-code màu, cỡ chữ, radius. Ngày hiển thị dạng số DD/MM/YYYY.
Kiểm tra tương phản ở cả ba chế độ Sáng, Ấm, Tối.
```

**c) Trước khi cho phép commit hoặc push:**

```
Chạy checklist bảo mật SECURITY_BASELINE.md mục 10.2 và cổng kiểm
./scripts/check-standards.sh. Báo kết quả từng mục rồi mới commit với
message dạng "loại(phạm-vi): mô tả ngắn".
```

**d) Khi cần đụng file agent khác đang khóa:**

```
File <đường dẫn> đang bị khóa trong LOCKS.md. Không sửa trực tiếp.
Tạo phiếu bàn giao logs/handover/ theo mẫu, ghi rõ thay đổi cần thực hiện,
lý do và tiêu chí hoàn thành, gửi đến agent đang giữ khóa.
```

**e) Chạy nhiều agent song song:**

Mở mỗi agent một terminal riêng, mỗi terminal dán lệnh 2.1 (biến thể giao task)
với task **khác nhau**, và thêm dòng định danh:

```
Tên định danh của bạn trong phiên này: agent-<số hoặc tên nhiệm vụ>.
Dùng tên này khi ghi STATE.md, LOCKS.md và phiếu bàn giao.
Làm trên nhánh riêng. Không đụng file ngoài phạm vi task của mình.
```

**f) Khôi phục sau khi phiên trước chết giữa chừng:**

```
Phiên trước kết thúc đột ngột. Đọc AGENTS.md, logs/STATE.md, phiếu bàn giao
mới nhất và logs/LOCKS.md. Đối chiếu git status với STATE.md để xác định
việc dang dở, nhả các khóa mồ côi của phiên trước, rồi tiếp tục đúng
"bước tiếp theo" đã ghi. Không làm lại việc đã hoàn thành.
```

**g) Phát hành phiên bản mới (app/tool, không áp dụng cho website):**

```
Phát hành bản mới cho <tên-app> theo docs/VERSIONING.md mục 2:
đặt tên file dạng {ten-app}_{YY}.{M}.{DD}{NN}_{arch}-setup.{ext} với NN là
số thứ tự trong ngày hôm nay (kiểm CHANGELOG.md để lấy NN kế tiếp),
đồng bộ chuỗi version vào manifest, công bố SHA-256 của file cài,
ghi một dòng vào CHANGELOG.md.
```

**h) Rà soát bảo mật cho một thay đổi:**

```
Thay đổi này đụng <xác thực / phân quyền / tải file lên / dữ liệu cá nhân>.
Điền phiếu docs/templates/SECURITY_REVIEW.md, đối chiếu SECURITY_BASELINE.md
mục 6 và 7. Nêu rõ ca tấn công đã cân nhắc và cách chặn.
```

**i) Nâng bộ quy ước lên bản mới:**

```
Chạy ./scripts/sync-standards.sh, đọc CHANGELOG của bản mới, liệt kê những
thay đổi ảnh hưởng tới repo này vào logs/STATE.md mục hàng đợi. Nếu là thay
đổi phá vỡ, làm phần "Hướng dẫn nâng cấp" trước khi commit.
```

### 2.4. Lệnh dùng một lần khi mới triển khai bộ quy ước

Dán ngay sau khi chạy `sync-standards.sh` lần đầu:

```
Đọc AGENTS.md và toàn bộ .standards/docs/. Đây là bộ quy ước bất khả xâm phạm.
Việc cần làm một lần:
1. Gộp .gitignore: thêm các dòng còn thiếu từ
   .standards/templates/gitignore/base.gitignore, KHÔNG xóa phần cũ.
2. Đối chiếu cấu trúc repo với docs/PROJECT_STRUCTURE.md, liệt kê điểm lệch
   chuẩn vào logs/STATE.md mục hàng đợi (chỉ liệt kê, chưa sửa).
3. Rà soát mã nguồn, liệt kê các vị trí hard-code màu và cỡ chữ vào hàng đợi.
4. Phân loại dữ liệu project theo SECURITY_BASELINE.md mục 2, ghi vào
   docs/ARCHITECTURE.md.
5. Chạy ./scripts/check-standards.sh và báo cáo từng mục chưa đạt.
Xong báo cáo ngắn gọn từng mục, chưa sửa gì.
```

---

## Phần 3 - Một phiên làm việc chuẩn

```
Mở terminal mới
   |
   v
Dán LỆNH MỞ PHIÊN (2.1)   ->  agent đọc quy ước, nhận task, khóa file
   |
   v
Agent làm việc            ->  giao thêm việc: lệnh 2.3a
   |                      ->  sửa giao diện:  lệnh 2.3b
   |                      ->  trước commit:   lệnh 2.3c
   |                      ->  file bị khóa:   lệnh 2.3d
   v
Hết task, hoặc muốn dừng, hoặc ngữ cảnh sắp cạn
   |
   v
Dán LỆNH KẾT THÚC PHIÊN (2.2)  ->  ghi STATE + phiếu bàn giao + nhả khóa + dọn dẹp
   |
   v
Đóng terminal  ->  phiên sau mở terminal mới, quay lại bước đầu
```
