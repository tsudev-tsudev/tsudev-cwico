# QUY TRÌNH GIT - nhánh, commit, PR, phát hành (v2.0.0)

## 1. Nhánh

| Nhánh | Vai trò | Quy tắc |
| --- | --- | --- |
| `main` | Luôn ở trạng thái phát hành được | Cấm push thẳng. Chỉ vào bằng PR đã qua cổng kiểm |
| `feat/<mô-tả>` | Tính năng mới | Tách từ `main`, sống ngắn |
| `fix/<mô-tả>` | Sửa lỗi | |
| `chore/<mô-tả>` | Việc hạ tầng, phụ thuộc, quy ước | |
| `docs/<mô-tả>` | Chỉ tài liệu | |
| `hotfix/<mô-tả>` | Sửa khẩn cho production | Được rút gọn quy trình duyệt, nhưng không bỏ cổng kiểm |

- Tên nhánh dùng `kebab-case`, tiếng Việt không dấu hoặc tiếng Anh:
  `feat/tim-kiem-khong-dau`, `fix/mat-dau-khi-go-nhanh`.
- Nhánh `SHOULD` sống dưới **3 ngày**. Nhánh sống lâu là nhánh sẽ xung đột.
- Xóa nhánh ngay sau khi merge.

## 2. Cấu hình bảo vệ nhánh `main` (bắt buộc)

Tại `Settings -> Branches -> Add rule` cho `main`:

- [x] Require a pull request before merging - **số người duyệt theo mục 4.4**
- [x] Require status checks to pass - chọn cổng kiểm quy ước và test
- [x] Require branches to be up to date before merging
- [x] Require conversation resolution before merging
- [x] Do not allow bypassing the above settings
- [ ] Allow force pushes - **để tắt**
- [ ] Allow deletions - **để tắt**

## 3. Commit

Định dạng: `loại(phạm-vi): mô tả ngắn`

```
feat(search): tìm không dấu ra kết quả có dấu
fix(auth): sửa hết hạn token khi đổi múi giờ
chore(deps): nâng vite lên 5.4.11
docs(security): bổ sung quy trình ứng phó lộ secret
```

| Loại | Dùng khi |
| --- | --- |
| `feat` | Thêm tính năng người dùng thấy được |
| `fix` | Sửa lỗi |
| `docs` | Chỉ tài liệu |
| `style` | Định dạng mã, không đổi hành vi |
| `refactor` | Đổi cấu trúc mã, không đổi hành vi và không sửa lỗi |
| `perf` | Cải thiện hiệu năng |
| `test` | Thêm hoặc sửa test |
| `build` | Hệ thống build, phụ thuộc |
| `ci` | Cấu hình CI |
| `chore` | Việc còn lại |
| `revert` | Hoàn nguyên commit trước |

**Quy tắc:**

- Dòng đầu tối đa **72 ký tự**, viết thường sau dấu hai chấm, không chấm cuối câu.
- Dùng thể mệnh lệnh: "sửa", "thêm", "bỏ" - không dùng "đã sửa", "đang thêm".
- Thay đổi phá vỡ: thêm `!` sau phạm vi và một đoạn `BREAKING CHANGE:` trong thân:
  ```
  feat(tokens)!: đổi giá trị text-muted cho đạt WCAG AA

  BREAKING CHANGE: ảnh chụp giao diện và test tương phản của mọi repo con
  cần chạy lại. Xem hướng dẫn nâng cấp trong CHANGELOG.
  ```
- Một commit làm **một việc**. Commit trộn 3 việc thì không hoàn nguyên riêng
  được việc nào.
- Chỉ dùng gạch ngang ngắn `-` trong mọi phần của commit message.

## 4. Pull Request

### 4.1. Trước khi mở

- [ ] Đã chạy `./scripts/check-standards.sh` và đạt.
- [ ] Đã chạy `gitleaks detect --redact` và sạch.
- [ ] Đã chạy lint và test.
- [ ] Đã tự đọc lại toàn bộ diễn biến của chính mình.
- [ ] Đã nhả khóa của mình trong `logs/LOCKS.md`.

### 4.2. Kích thước

PR `SHOULD` dưới **400 dòng thay đổi**. Số liệu ngành cho thấy chất lượng rà
soát rơi rất nhanh sau ngưỡng này - người duyệt bắt đầu bấm "Approve" thay vì
đọc. PR lớn hơn `MUST` được tách, hoặc `MUST` ghi rõ trong phần mô tả vì sao
không tách được.

### 4.3. Mô tả PR

Theo mẫu `.github/pull_request_template.md`. Tối thiểu phải trả lời được:

1. **Vì sao** cần thay đổi này (vấn đề, không phải giải pháp).
2. **Làm gì** (tóm tắt cách tiếp cận, không liệt kê từng file).
3. **Kiểm chứng ra sao** (bước tái hiện, ảnh chụp trước/sau nếu đụng giao diện).
4. **Rủi ro và đường lùi**.

### 4.4. Rà soát

**Số người duyệt bắt buộc, theo quy mô đội:**

| Repo có bao nhiêu người ghi được | `required_approving_review_count` | Vì sao |
| --- | --- | --- |
| Từ 2 người trở lên | **1** | Luôn có người khác đọc được diễn biến |
| Một người | **0** | GitHub `MUST NOT` cho tự duyệt PR của chính mình. Đặt 1 ở repo một người là tự khóa hoàn toàn: PR không bao giờ đủ điều kiện merge |

Đặt `0` **KHÔNG** phải bỏ rà soát. Ở repo một người, những thứ sau vẫn `MUST`
giữ nguyên - chúng mới là phần chặn thật:

- Bắt buộc mở PR, `MUST NOT` đẩy thẳng vào `main`.
- Bắt buộc cổng kiểm xanh (`Require status checks to pass`).
- Bắt buộc giải quyết hết thảo luận.
- Bắt buộc áp cả với admin (`Do not allow bypassing the above settings`).
- Người mở PR tự đọc lại toàn bộ diễn biến của mình theo mục 4.1.

Khi repo có người thứ hai ghi được, `MUST` nâng lại thành `1` **ngay trong ngày
cấp quyền** - đây là bước dễ quên nhất khi mở rộng đội.

**Cách rà soát:**

- Người duyệt `MUST` thực sự chạy thử khi PR đụng luồng người dùng.
- Góp ý `MUST` nói rõ mức: `Bắt buộc sửa` / `Nên sửa` / `Góp ý thêm`. Người mở
  PR không phải đoán ý.
- Góp ý nhắm vào mã, không nhắm vào người viết.
- PR đụng xác thực, phân quyền, tải file lên, hoặc dữ liệu D2/D3 `MUST` có thêm
  một lượt rà soát bảo mật theo `templates/SECURITY_REVIEW.md`.

### 4.5. Cách merge

- **Squash and merge** là mặc định. Lịch sử `main` sạch, mỗi PR một commit.
- **Rebase and merge** khi chuỗi commit thực sự có ý nghĩa riêng và đã được dọn.
- **Merge commit**: chỉ cho nhánh phát hành.

## 5. Phát hành

1. Cập nhật `CHANGELOG.md`.
2. Cập nhật chuỗi phiên bản theo `VERSIONING.md`.
3. Chạy toàn bộ cổng kiểm.
4. Tạo nhãn có chú thích, đẩy nhãn lên.
5. Tạo GitHub Release kèm phần CHANGELOG tương ứng.
6. Với app desktop: đính kèm file cài đặt đặt tên đúng khuôn của `VERSIONING.md`
   mục 2, kèm SHA-256 của từng file để người tải kiểm chứng.

## 6. Việc cấm

- `MUST NOT` push thẳng vào `main`.
- `MUST NOT` `git push --force` vào nhánh chung. Khi thật sự cần, dùng
  `--force-with-lease` và báo trước cho những người đang làm trên nhánh đó.
- `MUST NOT` commit bản build, `node_modules/`, file `.env`.
- `MUST NOT` sửa lịch sử đã merge vào `main`, trừ trường hợp dọn secret bị lộ
  theo `SECURITY_BASELINE.md` mục 9.2.
- `MUST NOT` merge PR của chính mình khi repo có từ 2 người trở lên.
- `MUST NOT` tắt cổng kiểm CI để merge cho kịp. Cổng kiểm đỏ nghĩa là chưa xong.
