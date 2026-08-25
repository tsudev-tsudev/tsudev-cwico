# STATE.md - Trạng thái project

> Agent đọc file này ở đầu mỗi phiên và cập nhật ở cuối mỗi phiên.
> Quy trình: `.standards/docs/AGENT_PROTOCOL.md`.

## Hàng đợi task (làm từ trên xuống)

- [ ] **QU-STD-1** Di trú `tokens/` sang `.standards/tokens/` (nguồn chân lý duy nhất). Hiện có **0 file mã nguồn** đọc token cục bộ. Đây là thay đổi PHÁ VỠ: `text-muted` đổi giá trị ở cả ba chế độ và có thêm `border-control`. Làm theo CHANGELOG mục 2.0.0 "Hướng dẫn nâng cấp", chạy lại ảnh chụp giao diện.
- [ ] **QU-STD-3** Rà chỗ dùng `border-strong` cho viền nút phụ hoặc ô nhập, đổi sang `border-control` (`.standards/docs/DESIGN_SYSTEM.md` mục 1).
- [ ] **QU-STD-AUTH** Rà luồng đăng nhập theo `.standards/docs/AUTH_AND_ACCOUNT.md` mục 17. Repo này là **hạng B - desktop có kết nối**, nên `MUST` đủ mục 3 tới 13 và thêm mục 14.1. App desktop `MUST` mở trình duyệt hệ thống với vòng lặp quay về, không nhúng WebView đăng nhập.
- [ ] **QU-STD-TABLE** Thêm bộ chọn số bản ghi `10/20/50/100/200` (mặc định `10`, góc dưới bên trái) cho mọi bảng và mọi modal có bản ghi. Chuẩn: `.standards/docs/DATA_TABLE.md` mục 12.
- [ ] **QU-STD-BRAND** Bổ sung tài sản nhận diện còn thiếu và siêu dữ liệu nối về `tsudev.com`. Chuẩn: `.standards/docs/BRAND_ASSETS.md` mục 14 và `.standards/docs/ECOSYSTEM_IDENTITY.md` mục 9. Chữ "dev" ở `ui/src/components/Brand.tsx` đang ra màu cam trong khi website ra màu xanh - xem việc treo TS-8 ở `tsudev-standards`, chưa tự đổi.

## Đang thực hiện

| Task | Agent | Bắt đầu |
| --- | --- | --- |

## Đã hoàn thành (mới nhất trên cùng)

- 24/08/2026 - Đưa bộ quy ước tsudev v2.1.0 vào repo, bật cổng kiểm CI.

## Quyết định quan trọng

> Quyết định kiến trúc lớn thì viết ADR riêng theo `docs/templates/ADR.md` và chỉ
> ghi một dòng tham chiếu ở đây.

- 24/08/2026 - Bộ quy ước đồng bộ từ `tsudev-standards`, bản sao chỉ-đọc ở `.standards/`. Không sửa ngược.

## Sự cố bảo mật

> Ghi theo `.standards/docs/SECURITY_BASELINE.md` mục 9. Để trống nếu chưa có.

- (chưa có)
