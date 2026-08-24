# AGENTS.md - <tên-repo>

> **ĐỌC FILE NÀY ĐẦU TIÊN trong mọi phiên làm việc mới.**

## Phần A - Quy ước chung của hệ sinh thái (KHÔNG SỬA Ở ĐÂY)

Toàn bộ quy ước chung nằm trong bản sao chỉ-đọc tại
[`.standards/AGENTS.md`](.standards/AGENTS.md). `MUST` đọc file đó trước.

Bản quy ước repo này đang dùng: xem [`.standards-version`](.standards-version).

| Cần gì | Đọc file nào |
| --- | --- |
| Điểm vào, nguyên tắc chung | `.standards/AGENTS.md` |
| Quy trình phiên, khóa file, bàn giao | `.standards/docs/AGENT_PROTOCOL.md` |
| Bảo mật bắt buộc | `.standards/docs/SECURITY_BASELINE.md` |
| Quy tắc `.gitignore` | `.standards/docs/GITIGNORE_POLICY.md` |
| Nhánh, commit, PR, phát hành | `.standards/docs/GIT_WORKFLOW.md` |
| Giao diện và token | `.standards/docs/DESIGN_SYSTEM.md` |
| Cấu trúc thư mục | `.standards/docs/PROJECT_STRUCTURE.md` |
| Chọn ngôn ngữ, framework | `.standards/docs/LANGUAGE_SELECTION.md` |
| Hạ tầng 0 đồng | `.standards/docs/FREE_TIER_STACK.md` |
| Trình soạn thảo nội dung | `.standards/docs/RICH_TEXT_EDITOR.md` |
| Tìm kiếm và lọc tiếng Việt | `.standards/docs/SEARCH_AND_FILTER.md` |
| Kiểm thử và chất lượng mã | `.standards/docs/TESTING_QUALITY.md` |
| Khả năng truy cập | `.standards/docs/ACCESSIBILITY.md` |

`MUST NOT` sửa bất kỳ file nào trong `.standards/`. Cần đổi quy ước thì mở đề
xuất tại repo `tsudev-standards` theo `.standards/docs/SYNC.md` mục 1.

## Phần B - Riêng của repo này

> Phần này KHÔNG thuộc bộ quy ước chung. Điền theo thực tế của repo.

### B.1. Repo này là gì

- **Loại**: <website | app di động | phần mềm desktop | thư viện | dịch vụ>
- **Stack**: <ghi ngắn, chi tiết ở `docs/ARCHITECTURE.md`>
- **Mức phân loại dữ liệu cao nhất**: <D0 | D1 | D2 | D3>
- **Người liên hệ khi có sự cố**: <tên>

### B.2. Chạy trong 5 phút

```bash
# <lệnh cài đặt>
# <lệnh chạy môi trường phát triển>
# <lệnh chạy test>
```

### B.3. Ranh giới sở hữu file

Ai hoặc agent nào được sửa vùng nào. Giúp chia việc song song mà không giẫm chân.

| Vùng | Chủ sở hữu |
| --- | --- |
| `src/features/<...>` | |
| `src/services/<...>` | |

### B.4. Điểm lệch chuẩn đã biết

> Mỗi dòng `MUST` có lý do và kế hoạch xử lý. Không có kế hoạch thì không phải
> điểm lệch, mà là nợ chưa ghi nhận.

| Lệch ở đâu | Vì sao | Kế hoạch |
| --- | --- | --- |
| | | |

### B.5. Việc riêng của repo cần biết trước khi sửa

-
