# CHUẨN KHẢ NĂNG TRUY CẬP (v2.0.0)

> Mục tiêu: **WCAG 2.1 mức AA** cho mọi giao diện tsudev, web lẫn desktop.
> Đây là yêu cầu tối thiểu, không phải tính năng thêm vào sau.

## 1. Vì sao bắt buộc

Không phải vì tuân thủ hình thức. Ba lý do vận hành:

1. Người dùng thật có suy giảm thị lực, vận động, hoặc đang dùng thiết bị trong
   điều kiện tệ (nắng gắt, một tay, mạng chậm).
2. Giao diện đi được bằng bàn phím là giao diện có cấu trúc rõ ràng. Sửa a11y
   thường là sửa luôn cả kiến trúc component.
3. Sửa sau tốn gấp nhiều lần. Vòng focus và nhãn ARIA thêm lúc viết mất vài
   phút; thêm sau khi có 200 màn hình thì thành một dự án riêng.

## 2. Màu và tương phản

- Mọi cặp chữ/nền `MUST` đạt ngưỡng ghi tại `DESIGN_SYSTEM.md` mục 1. Cổng canh:
  `scripts/check-contrast.mjs` ở repo trung tâm.
- Ranh giới vùng tương tác (viền nút phụ, viền ô nhập) dùng token
  `border-control`, đạt tối thiểu **3:1** theo WCAG 1.4.11. Token `border-strong`
  chỉ dành cho ranh giới **trang trí** và không phải đạt ngưỡng này.
- `MUST NOT` truyền đạt thông tin **chỉ** bằng màu. Trạng thái luôn kèm icon
  hoặc chữ - người mù màu chiếm khoảng 8% nam giới.
- `MUST NOT` dùng trắng tuyệt đối `#FFFFFF` làm nền chính hay đen tuyệt đối
  `#000000` ở bất kỳ chế độ nào.

## 3. Bàn phím

- **Mọi** chức năng `MUST` dùng được bằng bàn phím. Không có chức năng nào chỉ
  bấm chuột mới tới được.
- Thứ tự Tab `MUST` đi theo thứ tự đọc trên màn hình. `MUST NOT` dùng
  `tabindex` dương.
- Vòng focus `MUST` luôn nhìn thấy được: `2px solid var(--focus-ring)`, offset
  `2px`. `MUST NOT` viết `outline: none` mà không thay bằng chỉ dấu khác rõ hơn.
- Modal `MUST` giữ focus bên trong khi mở, đóng được bằng `Esc`, và **trả focus
  về đúng phần tử đã mở nó** khi đóng.
- Trang có vùng nội dung dài `MUST` có liên kết "Bỏ qua tới nội dung chính" xuất
  hiện khi nhận focus.

Phím tắt chuẩn cho danh sách kết quả và menu:

| Phím | Hành vi |
| --- | --- |
| `↓` / `↑` | Di chuyển giữa các mục, cuộn mục đang chọn vào vùng nhìn thấy |
| `Enter` | Kích hoạt mục đang chọn |
| `Esc` | Đóng, giữ nguyên nội dung đã gõ |
| `Tab` | Rời khỏi nhóm, không di chuyển trong nhóm |

## 4. Ngữ nghĩa và ARIA

- `MUST` dùng đúng thẻ HTML gốc trước khi nghĩ tới ARIA. `<button>` thật luôn tốt
  hơn `<div role="button">`.
- Một trang có đúng **một** `<h1>`. Cấp tiêu đề `MUST NOT` nhảy cóc (h2 rồi h4).
- Mọi ô nhập `MUST` có `<label>` gắn đúng, không chỉ có `placeholder`.
  Placeholder biến mất khi gõ - nó không phải nhãn.
- Mọi ảnh `MUST` có `alt`. Ảnh trang trí dùng `alt=""` để trình đọc bỏ qua. Với
  nội dung do người dùng tạo, `alt` `MUST` bắt buộc trước khi cho xuất bản.
- Nút chỉ có icon `MUST` có `aria-label` bằng tiếng Việt mô tả hành động.
- Nội dung thay đổi động (kết quả tìm kiếm, thông báo) `MUST` nằm trong vùng
  `aria-live="polite"`; lỗi dùng `aria-live="assertive"`.
- Ô tìm kiếm có gợi ý `MUST` dùng đúng bộ: `role="combobox"`, `aria-expanded`,
  `aria-controls`, `aria-activedescendant`, danh sách `role="listbox"`, mục
  `role="option"`.

## 5. Chuyển động và thời gian

- `MUST` tôn trọng `prefers-reduced-motion` - đã có sẵn trong `tokens.css`.
- `MUST NOT` có nội dung nhấp nháy quá 3 lần/giây (nguy cơ gây co giật).
- `MUST NOT` tự động phát video có tiếng.
- Nội dung tự chuyển (băng chuyền, thông báo tự đóng) `MUST` có cách tạm dừng.
  Thông báo lỗi `MUST NOT` tự đóng.

## 6. Tiếng Việt

- `MUST` đặt `lang="vi"` trên `<html>`. Trình đọc màn hình đọc sai hoàn toàn nếu
  tưởng là tiếng Anh.
- `MUST NOT` dùng ALL CAPS cho tiếng Việt có dấu ở đoạn dài - dấu bị cắt và trình
  đọc màn hình đọc từng chữ cái. Chỉ dùng cho nhãn ngắn từ 2 từ trở xuống.
- `MUST NOT` dùng font weight 300 cho tiếng Việt có dấu - nét mảnh làm dấu khó đọc.
- Cỡ chữ `MUST NOT` nhỏ hơn 12px, kể cả chú thích.

## 7. Kiểm chứng

| Cách | Công cụ | Bắt buộc |
| --- | --- | --- |
| Tự động trong CI | `axe-core` (`@axe-core/playwright` hoặc `jest-axe`) | `MUST` |
| Tương phản token | `scripts/check-contrast.mjs` | `MUST` |
| Kiểm nhanh trên trình duyệt | Lighthouse, axe DevTools | `SHOULD` |
| Đi hết màn hình **chỉ bằng bàn phím** | Thủ công | `MUST` trước mỗi lần phát hành |
| Trình đọc màn hình | NVDA (Windows) hoặc VoiceOver (macOS) | `SHOULD` cho luồng chính |

Công cụ tự động chỉ bắt được khoảng **30-40%** vấn đề thật. Bước đi hết màn hình
bằng bàn phím là bước không thay thế được.

## 8. Tiêu chí nghiệm thu

Một màn hình chỉ được coi là xong khi:

- [ ] Đi hết được mọi chức năng bằng bàn phím, focus luôn nhìn thấy.
- [ ] `axe-core` không báo vi phạm mức nghiêm trọng.
- [ ] Mọi ô nhập có nhãn, mọi ảnh có `alt`, mọi nút icon có `aria-label`.
- [ ] Tương phản đạt ngưỡng ở **cả ba** chế độ Sáng / Ấm / Tối.
- [ ] Phóng to trình duyệt lên 200% không mất nội dung và không xuất hiện cuộn ngang.
- [ ] Thông báo lỗi được trình đọc màn hình đọc lên, không chỉ đổi màu viền.
