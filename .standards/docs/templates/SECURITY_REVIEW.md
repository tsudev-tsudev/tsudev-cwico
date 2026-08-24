# PHIẾU RÀ SOÁT BẢO MẬT - <tên thay đổi>

> Bắt buộc điền khi thay đổi đụng tới: xác thực, phân quyền, tải tệp lên, thực
> thi truy vấn, hoặc dữ liệu mức D2/D3 theo `SECURITY_BASELINE.md` mục 2.
> Đính kèm phiếu này vào PR.

- **PR / nhánh**:
- **Người thực hiện**: - **Người rà soát**:
- **Ngày**: DD/MM/YYYY

## 1. Thay đổi này đụng vào đâu

- [ ] Xác thực hoặc quản lý phiên
- [ ] Phân quyền hoặc kiểm tra quyền sở hữu đối tượng
- [ ] Tải tệp lên hoặc phục vụ tệp
- [ ] Truy vấn cơ sở dữ liệu có dữ liệu do người dùng nhập
- [ ] Hiển thị nội dung do người dùng tạo ra HTML
- [ ] Gọi ra dịch vụ ngoài
- [ ] Xử lý dữ liệu cá nhân (D2) hoặc bí mật (D3)

## 2. Phân loại dữ liệu

| Dữ liệu chạm tới | Mức | Đã mã hóa khi truyền | Đã mã hóa khi lưu | Thời hạn lưu |
| --- | --- | --- | --- | --- |
| | | | | |

## 3. Đối chiếu OWASP

| Rủi ro | Có liên quan | Biện pháp đã áp dụng |
| --- | --- | --- |
| A01 Kiểm soát truy cập | | |
| A02 Mật mã | | |
| A03 Tiêm mã | | |
| A05 Cấu hình sai | | |
| A07 Định danh | | |
| A10 SSRF | | |

## 4. Ca tấn công đã cân nhắc

Với mỗi ca: mô tả kẻ tấn công làm gì, hệ thống chặn ở đâu, và **test nào chứng
minh việc chặn đó**.

1. **Ca**: - **Chặn ở**: - **Test**:
2. **Ca**: - **Chặn ở**: - **Test**:

## 5. Kiểm tra bắt buộc

- [ ] Đầu vào được kiểm tra bằng lược đồ tại biên hệ thống.
- [ ] Đầu ra được mã hóa đúng ngữ cảnh (HTML / thuộc tính / JS / URL / SQL).
- [ ] Quyền được kiểm tra ở phía máy chủ, trên từng thao tác.
- [ ] Kiểm tra quyền **sở hữu đối tượng**, không chỉ vai trò.
- [ ] Có giới hạn tần suất cho endpoint mới.
- [ ] Có giới hạn kích thước (chuỗi, tệp, mảng, độ sâu JSON).
- [ ] Không secret nào trong mã, log, hay thông báo lỗi.
- [ ] Thông báo lỗi không lộ chi tiết nội bộ.
- [ ] Hành vi nhạy cảm được ghi nhật ký, nhưng không ghi giá trị nhạy cảm.
- [ ] `gitleaks detect --redact` sạch.
- [ ] Không thêm phụ thuộc mới chưa rà soát theo `SECURITY_BASELINE.md` mục 5.

## 6. Rủi ro còn lại

Điều gì vẫn chưa được xử lý, vì sao chấp nhận được, và khi nào sẽ xử lý:

## 7. Kết luận của người rà soát

- [ ] Đạt - được merge
- [ ] Đạt có điều kiện - phải sửa các điểm ghi ở mục 6 trước khi phát hành
- [ ] Không đạt - lý do:
