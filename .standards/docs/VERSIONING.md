# QUY ƯỚC PHIÊN BẢN (v2.0.0)

Hệ sinh thái tsudev dùng **ba** khuôn phiên bản cho ba thứ khác nhau. Nhầm lẫn
giữa chúng là nguồn gốc của phần lớn tranh cãi khi phát hành.

| Đối tượng | Khuôn | Ví dụ | Nơi quy định |
| --- | --- | --- | --- |
| Bộ quy ước `tsudev-standards` | SemVer `MAJOR.MINOR.PATCH` | `2.0.0` | Tài liệu này, mục 1 |
| App/tool/phần mềm desktop phát hành thành file cài | `{YY}.{M}.{DD}{NN}` | `26.8.2401` | `DESIGN_SYSTEM.md` mục 6 |
| Website | Không có chuỗi phiên bản | | Mỗi đợt thay đổi là một dòng CHANGELOG theo ngày |

---

## 1. Phiên bản của bộ quy ước

Dùng **Semantic Versioning 2.0.0**, ghi ở file `VERSION` tại gốc repo trung tâm.

| Tăng số | Khi nào | Repo con phải làm gì |
| --- | --- | --- |
| **MAJOR** (`2.0.0` -> `3.0.0`) | Thay đổi phá vỡ: bỏ hoặc đổi tên token, siết một quy tắc từ `SHOULD` lên `MUST`, đổi cấu trúc thư mục bắt buộc | Đọc "Hướng dẫn nâng cấp" trong CHANGELOG, sửa code, rồi mới đồng bộ |
| **MINOR** (`2.0.0` -> `2.1.0`) | Bổ sung thuần: thêm token mới, thêm tài liệu mới, thêm quy tắc mức `SHOULD` | Đồng bộ được ngay, không cần sửa code |
| **PATCH** (`2.0.0` -> `2.0.1`) | Sửa lỗi diễn đạt, sửa liên kết hỏng, làm rõ câu chữ mà không đổi nghĩa | Đồng bộ được ngay |

**Quy tắc cứng:** thay đổi giá trị một token màu đang có là **MAJOR**, kể cả khi
chỉ để sửa lỗi. Lý do: mọi ảnh chụp giao diện, mọi test tương phản, mọi bản in
của mọi repo con đều đổi theo. Đó là định nghĩa của phá vỡ.

### 1.1. Quy trình phát hành bộ quy ước

1. Sửa nội dung trên nhánh riêng.
2. Cập nhật `VERSION`.
3. Ghi vào `CHANGELOG.md`. Nếu là MAJOR, `MUST` có mục **Hướng dẫn nâng cấp** với
   các bước cụ thể, không phải mô tả chung chung.
4. Chạy `node scripts/build-tokens.mjs` nếu có đụng token.
5. Chạy `./scripts/make-manifest.sh`.
6. Chạy `./scripts/check-standards.sh` - phải đạt.
7. Mở PR, được duyệt, merge vào `main`.
8. Tạo nhãn: `git tag -a v2.0.0 -m "..."` rồi `git push --tags`.
9. Tạo GitHub Release, dán phần CHANGELOG tương ứng.

### 1.2. Nhãn git

`MUST` đặt nhãn dạng `vMAJOR.MINOR.PATCH` (có chữ `v`), nhãn có chú thích
(`-a`), trỏ vào commit trên `main`. Repo con ghim bằng chính nhãn này.

---

## 2. Phiên bản của app/tool phát hành thành file cài

Định dạng: `{ten-app}_{YY}.{M}.{DD}{NN}_{arch}-setup.{ext}`

- `YY` = 2 số cuối của năm.
- `M` = tháng, **không** có số 0 ở đầu.
- `DD` = ngày, 2 chữ số.
- `NN` = số thứ tự phát hành **trong ngày**, bắt đầu từ `01`.
- `arch`: `x64` | `x86` | `arm64`.
- `ext`: `.exe` (Windows), `.dmg` (macOS), `.deb`/`.AppImage` (Linux), `.apk` (Android).

Ví dụ ngày 24/08/2026, app `tsudev-swico` cho Windows 64-bit:

```
tsudev-swico_26.8.2401_x64-setup.exe    # bản đầu trong ngày
tsudev-swico_26.8.2402_x64-setup.exe    # bản thứ hai cùng ngày
```

Chuỗi phiên bản trong code và manifest = `26.8.2401`, khớp đúng tên file.

**Vì sao không dùng SemVer cho app:** người dùng cuối không quan tâm số nào là
phá vỡ. Họ cần biết bản mình đang chạy có mới không, và mới bao lâu rồi. Khuôn
theo ngày trả lời được câu đó ngay từ tên file.

---

## 3. CHANGELOG

Mọi repo `MUST` có `CHANGELOG.md`, mới nhất trên cùng.

**App/tool** ghi mỗi bản một dòng:

```
26.8.2401 - 24/08/2026 - Sửa lỗi mất dấu tiếng Việt khi gõ nhanh trong ô tìm kiếm
```

**Website và thư viện** ghi theo đợt thay đổi:

```markdown
## 24/08/2026

- Thêm bộ lọc theo tác giả cho trang danh sách bài viết.
- Sửa lỗi trang tìm kiếm không giữ trạng thái khi bấm quay lại.
```

**Bộ quy ước** ghi theo phiên bản SemVer, xem `CHANGELOG.md` của repo này làm mẫu.

Quy tắc chung: viết cho **người đọc bản phát hành**, không phải cho người viết
code. "Sửa lỗi mất dấu tiếng Việt khi gõ nhanh" là dòng tốt; "refactor
useSearchInput hook" thì không.

---

## 4. Phiên bản của API

API công khai `MUST` gắn phiên bản vào đường dẫn: `/api/v1/...`.

- Thêm trường vào phản hồi, thêm tham số tùy chọn: **không** tăng phiên bản.
- Bỏ trường, đổi kiểu dữ liệu, đổi ngữ nghĩa, đổi mã lỗi: **phải** tăng phiên bản.
- Phiên bản cũ `MUST` còn sống tối thiểu **6 tháng** sau khi phiên bản mới ra,
  và `MUST` trả header `Deprecation` cùng `Sunset` trong thời gian đó.
