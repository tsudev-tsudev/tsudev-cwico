# BẢN ĐỒ BỘ QUY ƯỚC TSUDEV (v2.0.0)

> Đọc theo nhu cầu, không cần đọc tuần tự. Bắt đầu từ
> [`../AGENTS.md`](../AGENTS.md) - đó là điểm vào bắt buộc của mọi phiên.

## Đọc theo vai trò

**Bạn vừa vào dự án, chưa biết gì:**
1. [`../AGENTS.md`](../AGENTS.md) - điểm vào, nguyên tắc chung
2. [`ONBOARDING.md`](ONBOARDING.md) - đưa quy ước vào repo, bộ câu lệnh cho agent
3. [`PROJECT_STRUCTURE.md`](PROJECT_STRUCTURE.md) - file mới đặt ở đâu

**Bạn sắp khởi tạo một project mới:**
1. [`LANGUAGE_SELECTION.md`](LANGUAGE_SELECTION.md) - chọn ngôn ngữ và framework
2. [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md) - chọn hạ tầng 0 đồng
3. [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 10.1 - checklist khởi tạo
4. [`SYNC.md`](SYNC.md) mục 3 - đồng bộ bộ quy ước

**Bạn đang viết mã:**
1. [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) - bắt buộc trước khi đụng xác thực, tải tệp, truy vấn
2. [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) - token, không hard-code giao diện
3. [`TESTING_QUALITY.md`](TESTING_QUALITY.md) - ngưỡng chất lượng và kiểm thử
4. [`ACCESSIBILITY.md`](ACCESSIBILITY.md) - WCAG 2.1 AA
5. [`BRAND_ASSETS.md`](BRAND_ASSETS.md) - khi đụng logo, favicon, icon ứng dụng

**Bạn sắp commit hoặc mở PR:**
1. [`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) - nhánh, commit, PR
2. [`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md) - trước khi commit file mới
3. [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 10.2 và 10.3 - checklist

**Bạn là agent AI:**
1. [`AGENT_PROTOCOL.md`](AGENT_PROTOCOL.md) - toàn bộ quy trình phiên làm việc
2. [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 12 - ranh giới của agent
3. [`LANGUAGE_SELECTION.md`](LANGUAGE_SELECTION.md) mục 8 - ràng buộc khi chọn stack

**Bạn đang làm chức năng nội dung:**
1. [`RICH_TEXT_EDITOR.md`](RICH_TEXT_EDITOR.md) - trình soạn thảo ngang Word
2. [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md) - tìm kiếm và lọc tiếng Việt

## Danh mục đầy đủ

| File | Nội dung | Mức ràng buộc |
| --- | --- | --- |
| [`../AGENTS.md`](../AGENTS.md) | Điểm vào bắt buộc, nguyên tắc chung | Bất khả xâm phạm |
| [`../SECURITY.md`](../SECURITY.md) | Chính sách và kênh báo lỗ hổng | Bất khả xâm phạm |
| [`AGENT_PROTOCOL.md`](AGENT_PROTOCOL.md) | Phiên làm việc, khóa file, bàn giao | Bất khả xâm phạm |
| [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) | Chuẩn bảo mật kỹ thuật | Bất khả xâm phạm |
| [`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md) | Duy trì `.gitignore` | Bất khả xâm phạm |
| [`GIT_WORKFLOW.md`](GIT_WORKFLOW.md) | Nhánh, commit, PR, phát hành | Bất khả xâm phạm |
| [`VERSIONING.md`](VERSIONING.md) | Ba khuôn phiên bản | Bất khả xâm phạm |
| [`SYNC.md`](SYNC.md) | Đồng bộ quy ước xuống repo con | Bất khả xâm phạm |
| [`PROJECT_STRUCTURE.md`](PROJECT_STRUCTURE.md) | Cây thư mục chuẩn | Bất khả xâm phạm |
| [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) | Giao diện, token, component | Bất khả xâm phạm |
| [`BRAND_ASSETS.md`](BRAND_ASSETS.md) | Logo, wordmark, favicon, icon ứng dụng | Bất khả xâm phạm |
| [`ACCESSIBILITY.md`](ACCESSIBILITY.md) | WCAG 2.1 AA | Bất khả xâm phạm |
| [`TESTING_QUALITY.md`](TESTING_QUALITY.md) | Kiểm thử và chất lượng mã | Bất khả xâm phạm |
| [`LANGUAGE_SELECTION.md`](LANGUAGE_SELECTION.md) | Chọn ngôn ngữ theo loại project | Bất khả xâm phạm |
| [`FREE_TIER_STACK.md`](FREE_TIER_STACK.md) | Hạ tầng 0 đồng | Bất khả xâm phạm |
| [`RICH_TEXT_EDITOR.md`](RICH_TEXT_EDITOR.md) | Trình soạn thảo nội dung | Bất khả xâm phạm |
| [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md) | Tìm kiếm và lọc tiếng Việt | Bất khả xâm phạm |
| [`ONBOARDING.md`](ONBOARDING.md) | Triển khai và bộ câu lệnh agent | Hướng dẫn |
| [`templates/HANDOVER.md`](templates/HANDOVER.md) | Mẫu phiếu bàn giao | Mẫu |
| [`templates/ADR.md`](templates/ADR.md) | Mẫu ghi quyết định kiến trúc | Mẫu |
| [`templates/SECURITY_REVIEW.md`](templates/SECURITY_REVIEW.md) | Mẫu phiếu rà soát bảo mật | Mẫu |

## Ngoài thư mục docs

| Đường dẫn | Nội dung |
| --- | --- |
| `tokens/design-tokens.json` | Nguồn chân lý duy nhất của giao diện |
| `tokens/tokens.css` | Bản CSS sinh ra từ JSON, không sửa tay |
| `templates/gitignore/` | Bản `.gitignore` chuẩn theo nền tảng |
| `templates/logs/` | Mẫu `STATE.md`, `LOCKS.md`, `handover/` |
| `templates/AGENTS.downstream.md` | Mẫu `AGENTS.md` cho repo con |
| `scripts/sync-standards.sh` | Đồng bộ quy ước xuống repo con |
| `scripts/check-standards.sh` | Cổng kiểm quy ước |
| `scripts/build-tokens.mjs` | Sinh `tokens.css` từ JSON |
| `scripts/check-contrast.mjs` | Cổng canh tương phản WCAG |
| `proposals/` | Đề xuất đẩy ngược từ repo con |
| `exceptions/` | Ngoại lệ đã được duyệt |
