# CHUẨN KIỂM THỬ VÀ CHẤT LƯỢNG MÃ (v2.0.0)

## 1. Nguyên tắc

1. **Test tồn tại để bạn dám sửa mã.** Không phải để đạt chỉ tiêu phần trăm.
   Một bộ test không cho bạn tự tin refactor là bộ test hỏng.
2. **Mỗi lỗi sửa xong phải kèm một test tái hiện nó.** Không có test thì lỗi sẽ
   quay lại - chỉ là chưa biết lúc nào.
3. **Test phải chạy nhanh và ổn định.** Test lúc xanh lúc đỏ tệ hơn không có
   test: nó dạy cả nhóm bỏ qua màu đỏ.

## 2. Tháp kiểm thử

| Tầng | Tỉ trọng | Đối tượng | Bắt buộc |
| --- | --- | --- | --- |
| Unit | ~70% | Hàm thuần, logic nghiệp vụ, hàm chuẩn hóa dữ liệu | `MUST` |
| Integration | ~20% | API, truy cập database, luồng nhiều thành phần | `MUST` |
| E2E | ~10% | Kịch bản người dùng quan trọng nhất | `SHOULD` |

Đặt unit test **cạnh mã nguồn** trong `features/`; test tích hợp và E2E đặt ở
`tests/` theo `PROJECT_STRUCTURE.md`.

## 3. Bắt buộc phải có test

Không phụ thuộc tỉ lệ bao phủ, những chỗ sau `MUST` có test:

- Mọi hàm xử lý xác thực, phân quyền, kiểm tra quyền sở hữu đối tượng.
- Mọi hàm sanitize và validate đầu vào.
- Mọi hàm chuẩn hóa tiếng Việt (`viNormalizeText`, `viRemoveDiacritics`,
  `viWordCount`) - xem `SEARCH_AND_FILTER.md`.
- Mọi phép tính tiền, thuế, ngày giờ.
- Mọi lỗi từng xảy ra ở production.

## 4. Bao phủ

- Ngưỡng tối thiểu cho mã mới trong PR: **80%** dòng.
- Ngưỡng cho toàn repo: `SHOULD` không giảm so với lần trước (cổng "không tụt").
- `MUST NOT` đuổi theo 100%. Test viết chỉ để nâng phần trăm là nợ kỹ thuật đội
  lốt chất lượng.

## 5. Chất lượng test

- Tên test mô tả **hành vi**, không mô tả hàm:
  `"trả về kết quả có dấu khi người dùng gõ không dấu"`, không phải `"test
  search()"`.
- Cấu trúc **Sắp xếp - Thực thi - Khẳng định** (Arrange - Act - Assert), tách
  rõ ba khối.
- Một test khẳng định **một hành vi**.
- `MUST NOT` có test phụ thuộc thứ tự chạy, phụ thuộc thời gian thật, hay phụ
  thuộc mạng. Cố định đồng hồ, giả lập mạng.
- `MUST NOT` có `sleep` cố định trong E2E. Chờ theo điều kiện.
- Dữ liệu test `MUST` là dữ liệu giả. Không bao giờ dùng dữ liệu người dùng thật.

## 6. Cổng kiểm tự động

Mọi repo `MUST` có các cổng sau chạy trên PR và chặn merge:

| Cổng | Công cụ gợi ý theo ngôn ngữ |
| --- | --- |
| Định dạng mã | Prettier / `black` / `dotnet format` / `clang-format` |
| Lint | ESLint / Ruff / Roslyn Analyzers / clang-tidy |
| Kiểu tĩnh | `tsc --noEmit` / mypy / bật `TreatWarningsAsErrors` |
| Test + bao phủ | Vitest / pytest / xUnit / GoogleTest |
| Bảo mật | `SECURITY_BASELINE.md` mục 4.3 |
| Quy ước | `./scripts/check-standards.sh` |

TypeScript `MUST` bật `strict: true`. `MUST NOT` dùng `any` để cho qua lỗi kiểu -
dùng `unknown` rồi thu hẹp.

## 7. Chất lượng mã

- Một file **một trách nhiệm**. File vượt **400 dòng** `MUST` được cân nhắc tách.
- Một hàm `SHOULD` dưới 50 dòng và có **một** lý do để tồn tại.
- Hàm dùng chung ở **từ 2 nơi trở lên** `MUST` chuyển vào `src/utils/` hoặc
  `src/services/` (hoặc package dùng chung ở monorepo). Không copy-paste.
- Đặt tên nói rõ việc: `formatDateVN()`, `loadTokens()`. `MUST NOT` viết tắt mơ
  hồ (`d`, `tmp2`, `handleIt`).
- Comment giải thích **vì sao**, không giải thích **cái gì**. Mã đã nói cái gì rồi.
- `MUST NOT` để lại mã chết, mã bị comment, hay `console.log` gỡ lỗi trong PR.
- `MUST NOT` nuốt lỗi: `catch {}` rỗng bị cấm. Hoặc xử lý, hoặc ném lại kèm ngữ cảnh.

## 8. Hiệu năng

Ngưỡng tối thiểu cho ứng dụng web công khai (đo bằng Lighthouse trên máy tầm
trung, mạng 4G mô phỏng):

| Chỉ số | Ngưỡng |
| --- | --- |
| Largest Contentful Paint | < 2.5s |
| Interaction to Next Paint | < 200ms |
| Cumulative Layout Shift | < 0.1 |
| Kích thước JS ban đầu | < 200KB đã nén |

- `MUST` đặt kích thước cố định cho ảnh và khung nhúng để không xô lệch bố cục.
- `MUST NOT` gây lỗi truy vấn N+1. Có test tích hợp đếm số truy vấn cho endpoint
  danh sách.
