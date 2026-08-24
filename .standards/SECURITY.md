# Chính sách bảo mật - hệ sinh thái tsudev

> Áp dụng cho repo này và MỌI repo trong hệ sinh thái tsudev (https://tsudev.com).
> Yêu cầu kỹ thuật chi tiết: [`docs/SECURITY_BASELINE.md`](docs/SECURITY_BASELINE.md).

## 1. Phạm vi

Repo `tsudev-standards` chứa quy ước, tài liệu chuẩn, design token và script đồng
bộ. Repo **không chứa** secret, dữ liệu cá nhân, hay mã nguồn sản phẩm. Nếu bạn
tìm thấy bất kỳ secret nào trong repo này, đó là **sự cố**: báo ngay theo mục 3.

## 2. Phiên bản được hỗ trợ

| Phiên bản bộ quy ước | Trạng thái | Nhận cập nhật bảo mật |
| --- | --- | --- |
| `2.x` | Đang dùng | Có |
| `1.x` | Ngừng hỗ trợ từ 24/08/2026 | Không - hãy nâng cấp theo [`docs/SYNC.md`](docs/SYNC.md) |

## 3. Báo cáo lỗ hổng

**KHÔNG mở Issue công khai cho lỗ hổng bảo mật.**

1. Ưu tiên: dùng **GitHub Private Vulnerability Reporting** tại tab `Security` ->
   `Report a vulnerability` của repo liên quan. Kênh này riêng tư và miễn phí.
2. Nếu không dùng được kênh trên: gửi thư tới địa chỉ liên hệ công bố tại
   https://tsudev.com, tiêu đề bắt đầu bằng `[SECURITY]`.

Trong báo cáo xin ghi rõ: thành phần bị ảnh hưởng, phiên bản, bước tái hiện, tác
động đánh giá được, và (nếu có) bản vá đề xuất. **Không đính kèm secret thật** -
hãy che bớt (redact) trước khi gửi.

## 4. Cam kết phản hồi

| Mốc | Thời hạn |
| --- | --- |
| Xác nhận đã nhận báo cáo | 72 giờ |
| Đánh giá sơ bộ + phân mức độ | 7 ngày |
| Bản vá cho lỗ hổng Nghiêm trọng/Cao | 14 ngày |
| Bản vá cho lỗ hổng Trung bình/Thấp | 60 ngày |
| Công bố (sau khi đã có bản vá) | Thống nhất với người báo cáo |

## 5. Phân mức độ

Dùng **CVSS v3.1**. Quy đổi:

| Mức | Điểm CVSS | Ví dụ |
| --- | --- | --- |
| Nghiêm trọng | 9.0 - 10.0 | Thực thi mã từ xa, rò rỉ secret của hệ thống |
| Cao | 7.0 - 8.9 | Vượt quyền, SQL injection, XSS lưu trữ |
| Trung bình | 4.0 - 6.9 | XSS phản xạ, CSRF, rò rỉ thông tin có giới hạn |
| Thấp | 0.1 - 3.9 | Thiếu header bảo mật, lộ phiên bản thành phần |

## 6. Phạm vi được thử nghiệm

**Được phép**: đọc mã nguồn, phân tích tĩnh, thử nghiệm trên bản sao cục bộ của
chính bạn.

**Không được phép**: tấn công từ chối dịch vụ, gửi thư rác, tấn công kỹ nghệ xã
hội nhân sự tsudev, thử nghiệm trên hệ thống đang phục vụ người dùng thật, truy
cập hay làm rò rỉ dữ liệu của người khác.

## 7. Sự cố lộ secret

Một secret đã bị commit = **secret đã lộ**, kể cả khi commit bị xóa ngay sau đó.
Quy trình xử lý bắt buộc theo đúng thứ tự tại
[`docs/SECURITY_BASELINE.md`](docs/SECURITY_BASELINE.md) mục 9. Tóm tắt:

1. **Thu hồi/đổi khóa ngay** ở nhà cung cấp. Đây là bước đầu tiên, không phải
   bước xóa lịch sử git.
2. Rà soát nhật ký truy cập xem khóa đã bị dùng chưa.
3. Xóa khỏi lịch sử git (`git filter-repo`), buộc mọi bản sao clone lại.
4. Ghi sự cố vào `logs/STATE.md` của repo đó và báo cáo theo mục 3.

## 8. Công cụ bảo mật đang bật cho repo này

| Công cụ | Vai trò | Chi phí |
| --- | --- | --- |
| GitHub Secret Scanning + Push Protection | Chặn secret ngay khi push | 0 đồng (repo Public) |
| GitHub Dependabot | Cảnh báo và vá phụ thuộc | 0 đồng |
| GitHub CodeQL | Phân tích tĩnh tìm lỗ hổng | 0 đồng (repo Public) |
| gitleaks | Quét secret ở diễn biến PR và lịch sử | 0 đồng (mã nguồn mở) |
| `scripts/check-standards.sh` | Cổng kiểm quy ước chạy cục bộ và trên CI | 0 đồng |
