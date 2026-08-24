# HẠ TẦNG 0 ĐỒNG - nhà cung cấp và hạn mức được duyệt (v2.0.0)

> Hệ sinh thái tsudev xây dựng và phát triển theo hướng **chi phí 0 đồng**. Mọi
> project/tool/phần mềm `MUST` chọn giải pháp miễn phí phù hợp nhất. Đề xuất trả
> phí chỉ được xem xét khi đã chứng minh gói miễn phí không đủ, **kèm số liệu đo
> được**, và phải qua quy trình tại mục 6.

## 1. Ba nguyên tắc

1. **Miễn phí trước, trả phí sau, và phải chứng minh.** Câu "gói miễn phí chắc
   không đủ" không phải lý do. Lý do hợp lệ là con số: lượt gọi/tháng, dung
   lượng, băng thông, thời gian phản hồi đo được.
2. **Vùng gần Việt Nam.** Ưu tiên **Singapore**, kế đến **Nhật Bản
   (Tokyo/Osaka)** cho mọi dịch vụ cho chọn vùng: máy chủ, database, gốc CDN,
   lưu trữ. Chênh lệch độ trễ Singapore so với châu Âu/Mỹ là 150-250ms - người
   dùng cảm nhận được ngay.
3. **Không khóa chặt vào một nhà cung cấp.** Ưu tiên chuẩn mở (S3 API, Postgres,
   Docker, OCI). Trước khi dùng tính năng độc quyền của một nền tảng, hỏi: nếu
   hạn mức miễn phí bị siết, chuyển đi mất bao lâu?

## 2. Bảng nhà cung cấp được duyệt

Hạn mức ghi ở đây là mốc tham khảo tại 24/08/2026, `MUST` kiểm lại trên trang
chính thức trước khi ra quyết định kiến trúc.

### 2.1. Mã nguồn, CI/CD

| Dịch vụ | Hạn mức miễn phí | Ghi chú |
| --- | --- | --- |
| **GitHub** (repo Public) | Actions không giới hạn phút, CodeQL, Secret Scanning, Push Protection | Lựa chọn mặc định của hệ sinh thái |
| **GitHub** (repo Private) | 2.000 phút Actions/tháng, 500MB Packages | Không có CodeQL/Secret Scanning miễn phí |
| **GitHub Pages** | 1GB dung lượng, 100GB băng thông/tháng | Trang tĩnh, tài liệu |

### 2.2. Chạy web và hàm

| Dịch vụ | Hạn mức miễn phí | Vùng | Khi nào chọn |
| --- | --- | --- | --- |
| **Cloudflare Pages** | Build không giới hạn băng thông, 500 lần build/tháng | Biên toàn cầu | ⭐ Mặc định cho trang tĩnh và SPA |
| **Cloudflare Workers** | 100.000 yêu cầu/ngày | Biên toàn cầu | ⭐ Mặc định cho API nhẹ, hàm biên |
| **Vercel** (Hobby) | 100GB băng thông/tháng, chỉ dùng phi thương mại | `sin1` Singapore | Next.js cần SSR/ISR |
| **Netlify** | 100GB băng thông, 300 phút build/tháng | Biên toàn cầu | Thay thế cho Pages |
| **Oracle Cloud Always Free** | 4 nhân Ampere ARM, 24GB RAM, 200GB đĩa | Singapore, Tokyo, Osaka | ⭐ Khi cần máy chủ thật chạy liên tục |
| **Fly.io** | Hạn mức dùng thử theo tài khoản | `sin` Singapore | Container cần chạy gần người dùng |
| **Render** | Dịch vụ web miễn phí có ngủ đông sau 15 phút rảnh | Singapore | Chỉ cho môi trường thử nghiệm |

### 2.3. Cơ sở dữ liệu

| Dịch vụ | Hạn mức miễn phí | Vùng | Ghi chú |
| --- | --- | --- | --- |
| **Supabase** | 500MB Postgres, 1GB lưu trữ, 50.000 người dùng hoạt động/tháng | Singapore | ⭐ Postgres + Auth + Storage trong một |
| **Neon** | 0.5GB Postgres, tự ngủ khi rảnh, phân nhánh database | Singapore | ⭐ Khi chỉ cần Postgres thuần |
| **Turso** | 500 database, 9GB tổng, 1 tỷ lượt đọc hàng/tháng | Singapore | SQLite phân tán, cực nhẹ |
| **Cloudflare D1** | 5GB, 5 triệu lượt đọc hàng/ngày | Biên | Đi cùng Workers |
| **MongoDB Atlas** | Cụm M0 512MB | Singapore | Khi thực sự cần tài liệu phi quan hệ |
| **Upstash Redis** | 10.000 lệnh/ngày | Singapore | Cache, giới hạn tần suất, hàng đợi |

### 2.4. Lưu trữ tệp và CDN

| Dịch vụ | Hạn mức miễn phí | Ghi chú |
| --- | --- | --- |
| **Cloudflare R2** | 10GB lưu trữ, **0 đồng phí truyền ra** | ⭐ Mặc định. Tương thích API S3 |
| **Cloudflare CDN** | Băng thông không giới hạn | ⭐ Bắt buộc đặt trước mọi trang công khai |
| **Backblaze B2** | 10GB lưu trữ, 1GB tải xuống/ngày | Miễn phí truyền ra khi qua Cloudflare |
| **Cloudinary** | 25 tín dụng/tháng | Chỉ khi cần biến đổi ảnh phức tạp |

### 2.5. Dịch vụ hỗ trợ

| Nhu cầu | Lựa chọn 0 đồng | Hạn mức |
| --- | --- | --- |
| Gửi thư giao dịch | **Resend** / **Brevo** | 3.000 thư/tháng / 300 thư/ngày |
| Tìm kiếm | **Meilisearch** hoặc **Typesense** tự vận hành trên Oracle Always Free | Không giới hạn |
| Giám sát lỗi | **Sentry** gói miễn phí, hoặc **GlitchTip** tự vận hành | 5.000 sự kiện/tháng |
| Đo lường truy cập | **Cloudflare Web Analytics** / **Umami** tự vận hành | Không giới hạn, không dùng cookie |
| Theo dõi hoạt động | **UptimeRobot** | 50 điểm kiểm tra, chu kỳ 5 phút |
| Trạng thái dịch vụ | **Better Stack** gói miễn phí | 10 điểm kiểm tra |
| CAPTCHA | **Cloudflare Turnstile** | Không giới hạn |
| DNS | **Cloudflare DNS** | Không giới hạn |
| Chứng chỉ TLS | **Let's Encrypt** / Cloudflare | Tự động gia hạn |

## 3. Quy tắc thiết kế để ở lại trong hạn mức

- **Tránh hỏi liên tục (polling).** Dùng webhook, sự kiện do máy chủ đẩy (SSE),
  hoặc WebSocket. Một vòng lặp hỏi mỗi 5 giây tiêu 17.280 yêu cầu/ngày cho một
  người dùng - đủ vượt hạn mức Cloudflare Workers với 6 người dùng.
- **Cache tích cực.** Đặt `Cache-Control` đúng cho tài nguyên tĩnh; dùng
  `stale-while-revalidate`; cache kết quả truy vấn hay lặp lại.
- **Nén mọi thứ.** Bật Brotli/gzip, chuyển ảnh sang WebP/AVIF, tạo nhiều kích
  thước ảnh theo `srcset`.
- **Đẩy việc ra biên.** Xử lý ở CDN/Worker rẻ hơn và nhanh hơn so với gọi về máy
  chủ gốc.
- **Gom yêu cầu.** Một truy vấn lấy 20 bản ghi tốt hơn 20 truy vấn - và tránh
  được cả lỗi N+1.
- **Đặt cảnh báo hạn mức.** `MUST` bật thông báo khi dùng tới 80% hạn mức của bất
  kỳ dịch vụ nào, để không bị dừng dịch vụ bất ngờ.

## 4. Trước khi thêm một dịch vụ mới

`MUST` trả lời đủ 6 câu, ghi vào `docs/ARCHITECTURE.md` hoặc ADR của repo:

1. Project đã có thứ làm được việc này chưa?
2. Hạn mức miễn phí là bao nhiêu, và ước lượng mức dùng của ta là bao nhiêu?
3. Có vùng Singapore hoặc Tokyo không?
4. Nếu ngày mai nhà cung cấp bỏ gói miễn phí, chuyển đi mất bao lâu?
5. Dịch vụ này chạm dữ liệu mức nào theo `SECURITY_BASELINE.md` mục 2?
6. Ai trong nhóm giữ tài khoản, và khóa được xoay vòng ra sao?

## 5. Không được dùng

- `MUST NOT` dùng dịch vụ đòi thẻ tín dụng để kích hoạt gói miễn phí mà **không
  có hạn mức cứng** - rủi ro phát sinh hóa đơn ngoài kiểm soát.
- `MUST NOT` dùng gói dùng thử có hạn cho thành phần chạy thật ở production.
- `MUST NOT` dùng dịch vụ không cho xuất dữ liệu ra (không có đường thoát).
- `MUST NOT` dùng gói miễn phí phi thương mại (ví dụ Vercel Hobby) cho sản phẩm
  có doanh thu. Đây là vi phạm điều khoản, không phải mẹo tiết kiệm.

## 6. Khi cần đề xuất trả phí

Mở Issue tại `tsudev-standards` theo mẫu `Đề xuất quy ước`, nội dung `MUST` có:

1. **Số đo thực tế** cho thấy đã chạm hạn mức: biểu đồ mức dùng, thời điểm, tần suất.
2. **Các phương án 0 đồng đã thử** và vì sao không đạt.
3. **Chi phí dự kiến/tháng** và điểm mà chi phí đó tăng vọt.
4. **Đường lùi**: nếu ngừng trả phí thì hệ thống suy giảm ra sao.

## 7. Ghi lại mức dùng

Mỗi repo `MUST` có mục "Hạn mức đang dùng" trong `docs/ARCHITECTURE.md`, cập
nhật khi thêm hoặc bỏ dịch vụ:

```markdown
## Hạn mức đang dùng

| Dịch vụ | Gói | Hạn mức | Mức dùng gần nhất | Cập nhật |
| --- | --- | --- | --- | --- |
| Cloudflare Workers | Free | 100.000 yêu cầu/ngày | ~12.000/ngày | 24/08/2026 |
| Supabase | Free | 500MB | 180MB | 24/08/2026 |
```
