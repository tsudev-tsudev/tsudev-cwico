# XÁC THỰC VÀ KIẾN TRÚC TÀI KHOẢN - mọi project/tool/phần mềm tsudev (v3.0.0)

> Đăng nhập, đăng ký, liên kết nhà cung cấp, trạng thái tài khoản, và cơ chế
> **Xác minh tài khoản**. Đây là chuẩn bắt buộc, không phải gợi ý.
>
> Phần mật mã học và ngưỡng bảo mật nền nằm ở
> [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6; tài liệu này **không lặp
> lại** mà chỉ tham chiếu. Giao diện lấy token từ
> [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md). Nhận diện và trang hồ sơ ở
> [`ECOSYSTEM_IDENTITY.md`](ECOSYSTEM_IDENTITY.md).

## Mục lục

1. [Phạm vi và ba hạng sản phẩm](#1-phạm-vi-và-ba-hạng-sản-phẩm)
2. [Kiến trúc danh tính của hệ sinh thái](#2-kiến-trúc-danh-tính-của-hệ-sinh-thái)
3. [Ba lối vào bắt buộc](#3-ba-lối-vào-bắt-buộc)
4. [Luồng OIDC và OAuth](#4-luồng-oidc-và-oauth)
5. [Gộp và liên kết tài khoản](#5-gộp-và-liên-kết-tài-khoản)
6. [Đăng ký](#6-đăng-ký)
7. [Phiên đăng nhập](#7-phiên-đăng-nhập)
8. [Cơ chế Xác minh tài khoản](#8-cơ-chế-xác-minh-tài-khoản)
9. [Vòng đời và trạng thái tài khoản](#9-vòng-đời-và-trạng-thái-tài-khoản)
10. [Ma trận quyền theo trạng thái](#10-ma-trận-quyền-theo-trạng-thái)
11. [Xác thực hai lớp và khôi phục](#11-xác-thực-hai-lớp-và-khôi-phục)
12. [Đổi mật khẩu, đổi email, xóa tài khoản](#12-đổi-mật-khẩu-đổi-email-xóa-tài-khoản)
13. [Giao diện bắt buộc](#13-giao-diện-bắt-buộc)
14. [Sản phẩm chạy ngoại tuyến](#14-sản-phẩm-chạy-ngoại-tuyến)
15. [Chuẩn API](#15-chuẩn-api)
16. [Kiểm thử bắt buộc](#16-kiểm-thử-bắt-buộc)
17. [Checklist nghiệm thu](#17-checklist-nghiệm-thu)

---

## 1. Phạm vi và ba hạng sản phẩm

Không phải sản phẩm nào cũng gánh cùng một khối lượng. Xác định hạng **trước
khi** viết dòng mã xác thực đầu tiên, rồi ghi hạng đó vào `README.md` của repo.

| Hạng | Mô tả | Xác thực |
| --- | --- | --- |
| **A - Web có tài khoản** | Website, ứng dụng web, bảng quản trị. Ví dụ: `tsudev`, `tsudev-contact` | `MUST` đủ mục 3 tới 13 |
| **B - Desktop/app có kết nối** | App cài đặt nhưng có tính năng cần máy chủ (đồng bộ, giấy phép, báo cáo) | `MUST` đủ mục 3 tới 13, thêm mục 14.1 |
| **C - Ngoại tuyến** | Chạy trọn vẹn không cần internet. Ví dụ: `tsudev-swico` rà quét máy tính | Đăng nhập là **tùy chọn**, theo mục 14 |

**Quy tắc cứng:** hạng C `MUST NOT` chặn bất kỳ chức năng lõi nào sau một màn
hình đăng nhập. Người dùng mở phần mềm ra là dùng được ngay. Đăng nhập ở hạng C
chỉ mở thêm phần cộng thêm (đồng bộ báo cáo, lưu hồ sơ máy, cấu hình theo tài
khoản), và `MUST` nêu rõ mở thêm cái gì ngay tại nút đăng nhập.

Sản phẩm hạng A và B `MUST NOT` tự dựng kho người dùng riêng. Danh tính của hệ
sinh thái là **một**, ở `tsudev.com` (mục 2).

---

## 2. Kiến trúc danh tính của hệ sinh thái

```
                    Google            GitHub
                       |                 |
                       +--------+--------+
                                |   (nhà cung cấp liên kết)
                                v
                     +----------------------+
                     |   tsudev.com (IdP)   |  <- nguồn chân lý về danh tính
                     |  OpenID Connect      |     mật khẩu, hồ sơ, xác minh, 2FA
                     +----------+-----------+
                                |   (OIDC Authorization Code + PKCE)
        +------------+----------+----------+-------------+
        v            v                     v             v
     tsudev     tsudev-contact        tsudev-cwico    tsudev-swico
    (hạng A)      (hạng A)             (hạng B)        (hạng C)
```

- `tsudev.com` là **Nhà cung cấp danh tính** (IdP) duy nhất của hệ sinh thái.
  Mọi project khác là **bên tin cậy** (Relying Party, RP).
- Google và GitHub là **nhà cung cấp liên kết**: chúng nối vào IdP, **không** nối
  thẳng vào từng project. Nhờ vậy một người dùng đăng nhập bằng Google ở
  `tsudev.com` và ở `tsudev-cwico` là **cùng một tài khoản**, cùng một trạng thái
  xác minh, cùng một hồ sơ.
- `MUST NOT` khai báo OAuth app của Google/GitHub riêng cho từng project. Mỗi
  `client_id` thừa là một bề mặt tấn công thừa và một danh tính bị tách đôi.
- Điểm cuối chuẩn `MUST` công bố tại
  `https://tsudev.com/.well-known/openid-configuration`. RP `MUST` đọc cấu hình
  từ đó thay vì ghi cứng đường dẫn.
- Khóa ký `MUST` xoay vòng được qua JWKS (`/.well-known/jwks.json`); RP `MUST`
  cache theo `kid` và tự tải lại khi gặp `kid` lạ, `MUST NOT` cache vĩnh viễn.

### 2.1. Trường hợp chưa dựng xong IdP

Trong lúc IdP trung tâm chưa sẵn sàng, project mới `MAY` tạm dùng thư viện xác
thực cục bộ (`Auth.js`, `Supabase Auth`, ...) với **ba điều kiện bắt buộc**:

1. Lược đồ người dùng `MUST` đúng mục 9.1, để di trú sau này không mất dữ liệu.
2. `MUST` ghi một mục nợ kỹ thuật vào `logs/STATE.md` của repo đó.
3. `MUST NOT` phát hành ra người dùng ngoài trước khi có đường di trú, vì gộp hai
   kho người dùng sau khi đã có người thật là việc rất dễ làm hỏng.

---

## 3. Ba lối vào bắt buộc

Mọi màn hình đăng nhập của hạng A và B `MUST` có đủ và **đúng thứ tự** này:

| Thứ tự | Lối vào | Ghi chú |
| --- | --- | --- |
| 1 | **Tiếp tục với Google** | Nút có logo Google chính chủ, nền trắng, viền `border-strong` |
| 2 | **Tiếp tục với GitHub** | Nút có logo GitHub chính chủ, nền đen ở chế độ Sáng, nền `bg-surface` viền sáng ở chế độ Tối |
| 3 | **Tài khoản tsudev** | Email + mật khẩu; tài khoản đã đăng ký tại `tsudev.com` |

- Ba lối vào `MUST` cùng dẫn tới **một** không gian tài khoản. Cùng một email đã
  xác minh thì là cùng một người (mục 5).
- `MUST` ghi nhớ lối vào lần trước và đánh dấu "Lần trước bạn dùng cách này" -
  đây là biện pháp chống việc người dùng vô tình tạo tài khoản thứ hai, hiệu quả
  hơn mọi lời cảnh báo.
- `MUST NOT` bắt buộc liên kết Google/GitHub. Tài khoản tsudev thuần email luôn
  `MUST` là lối vào đủ dùng.
- `MUST NOT` thêm nhà cung cấp thứ tư (Facebook, X, Zalo, ...) mà không có đề xuất
  được duyệt theo [`SYNC.md`](SYNC.md) và cập nhật chính tài liệu này.

### 3.1. Phạm vi quyền xin của nhà cung cấp liên kết

Xin **ít nhất có thể**. Mỗi quyền thừa là một lần người dùng do dự và một nghĩa
vụ bảo mật thừa.

| Nhà cung cấp | Phạm vi được phép | Cấm |
| --- | --- | --- |
| Google | `openid`, `email`, `profile` | `MUST NOT` xin Drive, Calendar, Contacts |
| GitHub | `read:user`, `user:email` | `MUST NOT` xin `repo`, `admin:*`, `write:*` |

Cần quyền rộng hơn cho một tính năng cụ thể thì `MUST` xin **tăng dần** ngay tại
lúc dùng tính năng đó, không gộp vào lúc đăng nhập.

---

## 4. Luồng OIDC và OAuth

- `MUST` dùng **Authorization Code Flow kèm PKCE** (`S256`) cho mọi loại client,
  kể cả web có máy chủ. `MUST NOT` dùng Implicit Flow hay Password Grant.
- `MUST` sinh `state` ngẫu nhiên tối thiểu 128 bit, gắn với phiên, và **đối chiếu
  khi quay về**. Đây là lớp chống CSRF của luồng đăng nhập.
- `MUST` sinh `nonce` và đối chiếu nó trong `id_token`. Bỏ `nonce` là mở cửa cho
  tấn công phát lại token.
- `MUST` đối chiếu `iss`, `aud`, `exp`, `iat` của `id_token` và kiểm chữ ký bằng
  JWKS. `MUST NOT` tin `id_token` chỉ vì nó đến từ đường dẫn quay về.
- `redirect_uri` `MUST` nằm trong danh sách cho phép **so khớp chuỗi tuyệt đối**.
  `MUST NOT` so khớp theo tiền tố, theo ký tự đại diện, hay theo tên miền phụ mở.
- Sau `redirect_uri`, đích đến nội bộ (`?next=`) `MUST` được kiểm là đường dẫn
  tương đối cùng gốc. Đây là lỗ hổng **open redirect** kinh điển, và nó biến trang
  đăng nhập thật thành bàn đạp cho trang lừa đảo.
- Mã ủy quyền `MUST` dùng một lần, sống tối đa **60 giây**.
- App desktop (hạng B, C) `MUST` mở trình duyệt hệ thống với vòng lặp quay về
  `http://127.0.0.1:<cổng ngẫu nhiên>/callback`. `MUST NOT` nhúng WebView để hứng
  mật khẩu - đó là chống chỉ định của chính OAuth, và người dùng không thể kiểm
  chứng thanh địa chỉ.

---

## 5. Gộp và liên kết tài khoản

Đây là chỗ dễ mắc lỗi bảo mật nghiêm trọng nhất của cả tài liệu này. Đọc kỹ.

**Quy tắc cứng:** chỉ được tự động gộp hai danh tính theo email khi nhà cung cấp
**khẳng định email đã được xác minh** (`email_verified: true` với Google; email
đã xác minh và ở trạng thái chính với GitHub).

- Nhà cung cấp trả email **chưa xác minh**: `MUST` coi như một danh tính mới,
  `MUST` bắt xác minh email trước khi được nối vào tài khoản đang có.
  Bỏ bước này là lỗ hổng **chiếm tài khoản trước khi nạn nhân đăng ký**: kẻ tấn
  công tạo tài khoản ở nhà cung cấp bên thứ ba bằng email của nạn nhân, đăng nhập
  vào hệ thống trước, rồi ngồi đợi.
- Tài khoản tsudev thuần email muốn nối thêm Google/GitHub: `MUST` yêu cầu người
  dùng **đang đăng nhập** rồi mới nối, ở trang Bảo mật của hồ sơ.
- `MUST` cho phép **gỡ liên kết**, nhưng `MUST NOT` cho gỡ lối vào cuối cùng. Còn
  đúng một cách vào tài khoản thì nút gỡ `MUST` bị vô hiệu kèm giải thích: "Đặt
  mật khẩu hoặc nối một nhà cung cấp khác trước đã".
- Mọi lần nối hoặc gỡ liên kết `MUST` gửi email thông báo tới địa chỉ chính và
  ghi vào nhật ký kiểm toán (mục 9.2).
- Đổi email chính `MUST` không kéo theo việc gộp ngầm với một tài khoản đang tồn
  tại. Trùng email thì `MUST` báo lỗi và dừng, để con người xử lý.

---

## 6. Đăng ký

- Trường bắt buộc lúc đăng ký: **email** và **mật khẩu**. Hết. Tên hiển thị, ảnh
  đại diện, ngày sinh `MUST` để lại cho trang hồ sơ, điền sau.
- Mật khẩu theo [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6.1: tối thiểu
  12 ký tự, đối chiếu danh sách đã lộ, `MUST NOT` ép quy tắc ký tự đặc biệt kèm
  giới hạn độ dài tối đa thấp.
- `MUST` có thanh đo độ mạnh mật khẩu theo `zxcvbn` hoặc tương đương, và `MUST` có
  nút hiện/ẩn mật khẩu.
- `MUST NOT` để lộ tài khoản có tồn tại hay không (**user enumeration**): thông
  báo ở đăng ký, đăng nhập và quên mật khẩu `MUST` giống nhau về nội dung và
  **gần nhau về thời gian trả lời**. Ví dụ đúng: "Nếu email này có tài khoản,
  chúng tôi đã gửi hướng dẫn tới đó".
- `MUST` có `Turnstile` (Cloudflare, miễn phí) hoặc tương đương ở đăng ký và quên
  mật khẩu. `MUST NOT` dùng reCAPTCHA v2 dạng chọn ảnh - nó là rào cản khả năng
  truy cập.
- `MUST` giới hạn tần suất theo IP **và** theo email đích, có độ trễ tăng dần.
- Người dùng `MUST` chủ động đồng ý Điều khoản và Chính sách riêng tư bằng một ô
  đánh dấu **không tick sẵn**, có liên kết mở được ra tab mới.

---

## 7. Phiên đăng nhập

Nền tảng ở [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6.2. Phần riêng của
hệ sinh thái:

- Access token sống tối đa **15 phút**; refresh token sống tối đa **30 ngày**,
  **xoay vòng mỗi lần dùng**, và `MUST` có phát hiện tái sử dụng: một refresh
  token đã dùng lại lần hai nghĩa là nó đã bị đánh cắp, `MUST` thu hồi **cả cây
  phiên** của tài khoản đó và gửi email cảnh báo.
- Cookie phiên `MUST` đặt trên `tsudev.com` với `Domain=.tsudev.com` để dùng chung
  giữa các tên miền phụ, kèm đủ `HttpOnly`, `Secure`, `SameSite=Lax`.
- `MUST` có trang **"Thiết bị và phiên đang đăng nhập"** trong hồ sơ, liệt kê:
  thiết bị, trình duyệt, vị trí gần đúng theo IP, lần hoạt động cuối
  (`HH:mm DD/MM/YYYY`), và nút **Đăng xuất khỏi thiết bị này** cùng **Đăng xuất
  khỏi mọi thiết bị khác**.
- "Ghi nhớ đăng nhập" `MUST` là lựa chọn của người dùng, mặc định **tắt** trên
  thiết bị dùng chung không nhận biết được, nghĩa là mặc định tắt.
- Đăng xuất `MUST` vô hiệu phiên ở phía máy chủ, không chỉ xóa cookie.

---

## 8. Cơ chế Xác minh tài khoản

Mục tiêu: người dùng luôn biết **tài khoản mình đang ở trạng thái nào** và
**cần làm gì tiếp theo**, không phải đoán.

### 8.1. Định nghĩa

**Xác minh tài khoản** = đã chứng minh quyền kiểm soát địa chỉ email chính.

| Trạng thái | Điều kiện đạt |
| --- | --- |
| **Chưa xác minh** | Mới đăng ký, hoặc vừa đổi email chính và chưa xác nhận địa chỉ mới |
| **Đã xác minh** | Đã bấm liên kết xác minh còn hạn, hoặc đăng nhập bằng Google/GitHub trả về `email_verified: true` |

Đăng nhập bằng Google hoặc GitHub với email đã xác minh phía nhà cung cấp thì
`MUST` đánh dấu **Đã xác minh** ngay, `MUST NOT` bắt xác minh lại. Bắt người dùng
làm lại việc đã làm là cách nhanh nhất khiến họ bỏ đi.

### 8.2. Liên kết xác minh

- Token xác minh `MUST` ngẫu nhiên tối thiểu **256 bit**, sinh bằng nguồn ngẫu
  nhiên mật mã, **lưu dạng băm** trong cơ sở dữ liệu - lộ bản sao lưu cơ sở dữ
  liệu thì token vẫn không dùng được.
- Hạn **24 giờ**, **dùng một lần**, gắn cứng với đúng một email đích.
- `MUST` có nút **Gửi lại** với đếm ngược, sớm nhất **60 giây** một lần, tối đa
  **5 lần trong 24 giờ**.
- Trang xác minh `MUST` xử lý được cả ba ca và nói rõ bước tiếp theo: token hợp
  lệ, token hết hạn, token đã dùng rồi.
- `MUST NOT` để việc bấm liên kết xác minh **tự đăng nhập** người dùng vào phiên.
  Hộp thư có thể đã bị xem trộm, và trình quét liên kết của máy chủ email hay tự
  bấm trước cả người thật.

### 8.3. Thời hạn 7 ngày và các mức hạn chế

Đồng hồ chạy từ **thời điểm tạo tài khoản**.

| Giai đoạn | Trạng thái | Người dùng làm được gì |
| --- | --- | --- |
| **Ngày 0 tới hết ngày 7** - ân hạn | Chưa xác minh | Đăng nhập, đọc, sửa hồ sơ của mình, xác minh. **Không** bình luận, **không** đăng nội dung, **không** tải tệp lên, **không** gọi API ghi công khai |
| **Từ ngày 8** - hạn chế | Chưa xác minh, quá hạn | Chỉ còn: xem hồ sơ của mình, đổi email chính, gửi lại thư xác minh, xuất dữ liệu, xóa tài khoản. Mọi trang khác dành cho tài khoản đăng nhập đều `MUST` chuyển hướng về trang Xác minh |
| Bất kỳ lúc nào sau khi xác minh | Đã xác minh | Đầy đủ quyền theo vai trò (mục 10) |

- Mốc 7 ngày `MUST` là một hằng số cấu hình được (`ACCOUNT_VERIFY_GRACE_DAYS`),
  `MUST NOT` rải số 7 khắp mã nguồn.
- `MUST` nhắc bằng email vào **ngày 3** và **ngày 6**, và **một lần nữa** khi hết
  hạn kèm hướng dẫn khôi phục quyền.
- Tài khoản chưa xác minh quá **90 ngày** và chưa từng có hoạt động nào `SHOULD`
  bị xóa tự động, sau một email báo trước 7 ngày. Đây là cách dọn tài khoản rác
  và giảm bề mặt dữ liệu cá nhân phải giữ.
- Việc chặn `MUST` được thi hành **ở tầng máy chủ, trên từng thao tác ghi**. Ẩn
  nút "Bình luận" trên giao diện không phải là hạn chế, đúng theo
  [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6.3.

### 8.4. Hiển thị trạng thái

Trang hồ sơ và trang quản lý tài khoản `MUST` hiển thị huy hiệu trạng thái ngay
cạnh email chính, dùng component **Badge status** của
[`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) mục 5:

| Trạng thái | Nền | Chữ | Icon 16px | Nhãn |
| --- | --- | --- | --- | --- |
| Đã xác minh | `success` nhạt 12% | `success` đậm | dấu kiểm trong vòng tròn | `Đã xác minh` |
| Chưa xác minh, còn ân hạn | `warning` nhạt 12% | `warning` đậm | dấu chấm than trong tam giác | `Chưa xác minh` |
| Chưa xác minh, quá hạn | `danger` nhạt 12% | `danger` đậm | dấu chấm than trong vòng tròn | `Chưa xác minh` |

- Huy hiệu `MUST NOT` chỉ dùng màu để phân biệt: luôn kèm **chữ và icon**, theo
  [`ACCESSIBILITY.md`](ACCESSIBILITY.md).
- Trạng thái "Chưa xác minh" `MUST` đi kèm **một dòng nói rõ hậu quả và một nút
  hành động duy nhất**. Đúng: `Chưa xác minh - còn 4 ngày trước khi tài khoản bị
  hạn chế.` kèm nút `Gửi lại thư xác minh`. Sai: một dấu chấm than không giải
  thích gì.
- `MUST` có dải thông báo (banner) cố định trên đầu mọi trang khi tài khoản chưa
  xác minh, đóng tạm được nhưng `MUST` hiện lại ở phiên sau.
- Số ngày còn lại `MUST` đếm theo ngày tròn và nói bằng tiếng Việt tự nhiên:
  `còn 4 ngày`, `còn 1 ngày`, `hết hạn hôm nay`.

### 8.5. Xác minh nâng cao (tùy sản phẩm)

Email là mức nền. Sản phẩm có thao tác rủi ro cao `MAY` yêu cầu thêm, và khi đó
`MUST` ghi rõ trong tài liệu của chính sản phẩm đó:

| Mức | Chứng minh cái gì | Dùng khi |
| --- | --- | --- |
| Mức 1 - Email | Kiểm soát hộp thư | Mặc định toàn hệ sinh thái |
| Mức 2 - 2FA (TOTP) | Giữ một thiết bị | Tài khoản quản trị (`MUST`), tài khoản thường (`SHOULD`) |
| Mức 3 - Xác thực lại | Còn đang ngồi trước máy | Đổi mật khẩu, đổi email, xóa tài khoản, xem mã dự phòng |

Mức 3 `MUST` áp cho mọi thao tác trong mục 12, kể cả khi phiên còn hạn.

---

## 9. Vòng đời và trạng thái tài khoản

### 9.1. Lược đồ tối thiểu

Mọi sản phẩm `MUST` giữ đủ các trường sau, đúng tên, để di trú và đồng bộ được:

| Trường | Kiểu | Ghi chú |
| --- | --- | --- |
| `id` | UUID v7 | `MUST NOT` dùng số tự tăng làm định danh công khai |
| `email` | text, duy nhất, thường hóa | Lưu bản thường hóa để so khớp; giữ bản gốc để hiển thị |
| `email_verified_at` | timestamp, cho phép rỗng | Rỗng nghĩa là chưa xác minh. `MUST NOT` dùng cờ boolean - mất mốc thời gian là mất khả năng tính hạn |
| `status` | enum | Theo bảng 9.2 |
| `display_name` | text | Điền sau, có thể rỗng |
| `avatar_url` | text | Theo [`ECOSYSTEM_IDENTITY.md`](ECOSYSTEM_IDENTITY.md) |
| `created_at` | timestamp | Mốc bắt đầu đếm 7 ngày |
| `last_seen_at` | timestamp | Phục vụ dọn tài khoản rác |
| `deletion_requested_at` | timestamp, cho phép rỗng | Mốc bắt đầu ân hạn 30 ngày |

Danh tính liên kết nằm ở bảng riêng `account_identities`
(`user_id`, `provider`, `provider_user_id`, `email`, `email_verified`,
`linked_at`), khóa duy nhất trên cặp (`provider`, `provider_user_id`).

### 9.2. Các trạng thái

| `status` | Nghĩa | Vào bằng cách nào | Ra bằng cách nào |
| --- | --- | --- | --- |
| `active` | Bình thường | Mặc định khi tạo | - |
| `restricted` | Quá hạn xác minh 7 ngày | Tự động | Xác minh xong thì tự về `active` |
| `suspended` | Tạm khóa do vi phạm hoặc dấu hiệu bất thường | Quản trị viên, hoặc tự động khi phát hiện tái dùng refresh token | Quản trị viên gỡ |
| `locked` | Khóa do đăng nhập sai quá nhiều | Tự động | Tự mở sau thời gian chờ, hoặc qua đặt lại mật khẩu |
| `pending_deletion` | Người dùng đã yêu cầu xóa | Người dùng | Hủy yêu cầu trong 30 ngày |
| `deleted` | Đã ẩn danh hóa | Hết 30 ngày | Không |

Mọi lần đổi `status` `MUST` ghi **nhật ký kiểm toán** không sửa được: ai đổi, từ
gì sang gì, lý do, thời điểm, IP. Nhật ký này `MUST` giữ tối thiểu **12 tháng**.

---

## 10. Ma trận quyền theo trạng thái

`MUST` cài đặt đúng bảng này ở tầng máy chủ. Cột trái là điều kiện, không phải
gợi ý giao diện.

| Thao tác | Khách | Chưa xác minh (ân hạn) | Chưa xác minh (quá hạn) | Đã xác minh | Quản trị |
| --- | --- | --- | --- | --- | --- |
| Xem nội dung công khai | có | có | có | có | có |
| Đăng nhập | có | có | có | có | có |
| Xem hồ sơ của mình | không | có | có | có | có |
| Sửa hồ sơ, đổi ảnh đại diện | không | có | không | có | có |
| Gửi lại thư xác minh, đổi email chính | không | có | có | có | có |
| Bình luận, đánh giá | không | **không** | không | có | có |
| Đăng nội dung, tải tệp lên | không | **không** | không | có | có |
| Truy cập trang chỉ dành cho tài khoản | không | có | **không** | có | có |
| Gọi API ghi công khai | không | **không** | không | có | có |
| Xuất dữ liệu, xóa tài khoản | không | có | có | có | có |
| Trang quản trị | không | không | không | không | có, kèm 2FA |

- Bị chặn `MUST` trả `HTTP 403` kèm thân phản hồi nêu **mã lý do máy đọc được**
  (`account_unverified`, `account_restricted`, `account_suspended`) để giao diện
  hiện đúng thông điệp. `MUST NOT` trả `404` hay lỗi chung chung.
- Giao diện `MUST NOT` chỉ ẩn nút. Nút vẫn hiện, ở trạng thái `Disabled` theo
  [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) mục 5, kèm tooltip nói vì sao và cách gỡ.
  Ẩn đi thì người dùng tưởng tính năng không tồn tại.

---

## 11. Xác thực hai lớp và khôi phục

- 2FA `MUST` dùng **TOTP** (RFC 6238, 30 giây, 6 chữ số), chấp nhận lệch một cửa
  sổ trước và sau. `MUST NOT` dùng SMS làm lớp thứ hai mặc định - SIM swap là rủi
  ro thật và SMS quốc tế còn tốn tiền.
- Bật 2FA `MUST` sinh **10 mã dự phòng** dùng một lần, hiển thị đúng một lần, có
  nút tải về và nút sao chép. Mã `MUST` lưu dạng băm.
- `MUST` bắt buộc 2FA cho mọi tài khoản có quyền quản trị.
- `SHOULD` hỗ trợ **Passkey** (WebAuthn) như lối vào không mật khẩu. Khi có
  passkey, nó `MUST` đủ tư cách thay cho cả mật khẩu lẫn TOTP.
- Tắt 2FA `MUST` yêu cầu xác thực lại (mức 3) và `MUST` gửi email thông báo.
- Khôi phục khi mất cả thiết bị lẫn mã dự phòng `MUST` là quy trình có con người
  duyệt, `MUST NOT` tự động hóa. Đây chính là cửa hậu mà kẻ tấn công nhắm vào.

---

## 12. Đổi mật khẩu, đổi email, xóa tài khoản

Cả ba `MUST` yêu cầu xác thực lại (mục 8.5 mức 3) và `MUST` gửi email thông báo
tới **địa chỉ cũ** kèm câu "Không phải bạn? Bấm vào đây" dẫn tới luồng khóa khẩn.

**Đổi mật khẩu**
- `MUST` hỏi mật khẩu hiện tại (trừ luồng đặt lại qua email).
- `MUST` thu hồi mọi phiên khác sau khi đổi, giữ lại phiên đang thao tác.

**Đặt lại mật khẩu (quên mật khẩu)**
- Token `MUST` ngẫu nhiên tối thiểu 256 bit, lưu dạng băm, hạn **60 phút**, dùng
  một lần, và `MUST` mất hiệu lực khi mật khẩu đổi bằng đường khác.
- Phản hồi `MUST` như nhau dù email có tồn tại hay không (mục 6).

**Đổi email chính**
- Địa chỉ mới `MUST` được xác minh **trước khi** thay thế địa chỉ cũ. Trong lúc
  chờ, địa chỉ cũ vẫn là địa chỉ chính.
- Sau khi đổi, `email_verified_at` `MUST` được đặt lại theo mốc xác minh của địa
  chỉ mới, `MUST NOT` giữ mốc cũ.

**Xóa tài khoản**
- `MUST` có ân hạn **30 ngày** ở trạng thái `pending_deletion`, hủy được bằng một
  lần đăng nhập kèm xác nhận.
- Hết hạn thì `MUST` **ẩn danh hóa**, không xóa cứng dòng dữ liệu nếu nó còn được
  tham chiếu: email và tên thay bằng giá trị vô danh, ảnh đại diện xóa hẳn khỏi
  kho lưu trữ, nội dung công khai xử lý theo chính sách của sản phẩm và `MUST`
  nói rõ trong hộp thoại xác nhận.
- `MUST` cho **xuất dữ liệu** (JSON) trước khi xóa, tải về ngay hoặc gửi liên kết
  có hạn qua email.

---

## 13. Giao diện bắt buộc

### 13.1. Đường dẫn chuẩn

Mọi sản phẩm web `MUST` dùng đúng bộ đường dẫn này. Đồng nhất đường dẫn là điều
kiện để chia sẻ liên kết giữa các sản phẩm mà không hỏng.

```
/dang-nhap                     đăng nhập
/dang-ky                       đăng ký
/quen-mat-khau                 yêu cầu đặt lại mật khẩu
/dat-lai-mat-khau              đặt mật khẩu mới (có token)
/xac-minh                      trang trạng thái xác minh, gửi lại thư
/xac-minh/xac-nhan             đích của liên kết trong email
/tai-khoan                     tổng quan tài khoản
/tai-khoan/ho-so               hồ sơ cá nhân, ảnh đại diện
/tai-khoan/bao-mat             mật khẩu, 2FA, liên kết Google/GitHub
/tai-khoan/phien               thiết bị và phiên đang đăng nhập
/tai-khoan/du-lieu             xuất dữ liệu, xóa tài khoản
```

- Đường dẫn `MUST` không dấu, chữ thường, nối bằng gạch ngang ngắn `-`.
- Sản phẩm tiếng Anh `MAY` dùng bản dịch tương ứng nhưng `MUST` giữ nguyên cấu
  trúc cây và `MUST` chuyển hướng `301` từ bản tiếng Việt.

### 13.2. Bố cục màn hình đăng nhập và đăng ký

```
+---------------------------------------------------+
|                  [ dấu hiệu tsudev ]              |  logo-mark 48px + chữ text
|                Đăng nhập vào tsudev               |  24px / 600
|          Chưa có tài khoản? Đăng ký ngay          |  14px, liên kết
|                                                   |
|  [ (G)  Tiếp tục với Google              ]        |  cao 40px, full width
|  [ (GH) Tiếp tục với GitHub              ]        |  cao 40px, full width
|                                                   |
|  ------------------ hoặc ------------------       |  12px, text-muted
|                                                   |
|  Email                                            |  13px nhãn
|  [                                       ]        |  input 36px
|  Mật khẩu                        Quên mật khẩu?   |
|  [                                  (mắt) ]       |
|  [ ] Ghi nhớ đăng nhập                            |
|                                                   |
|  [           Đăng nhập            ]               |  Primary, full width
|                                                   |
|  Bằng việc tiếp tục, bạn đồng ý với Điều khoản    |  12px, text-muted
|  và Chính sách riêng tư của tsudev.               |
+---------------------------------------------------+
```

- Khung `MUST` rộng tối đa **400px**, căn giữa, nền `bg-surface`, `radius-lg`,
  `shadow-md`, đệm trong 32px (24px ở màn hình dưới 480px).
- Nút nhà cung cấp `MUST` nằm **trên** biểu mẫu email. Đa số người dùng chọn
  đường đó; bắt họ cuộn qua biểu mẫu là ma sát vô ích.
- Lỗi `MUST` hiện ngay dưới ô sai, chữ 13px màu `danger`, kèm `aria-describedby`.
  Lỗi toàn biểu mẫu hiện ở một dải phía trên nút gửi, `role="alert"`.
- `MUST` có trạng thái đang xử lý cho nút gửi: vô hiệu nút, đổi nhãn thành
  `Đang đăng nhập...`, `MUST NOT` để bấm hai lần.
- Ô email `MUST` có `autocomplete="username"`, ô mật khẩu `autocomplete="current-password"`
  (đăng ký: `new-password`), `inputmode="email"`. Trình quản lý mật khẩu là đồng
  minh, không phải kẻ địch.
- Toàn bộ luồng `MUST` dùng được **chỉ bằng bàn phím**, thứ tự `Tab` đúng theo thứ
  tự nhìn thấy, vòng focus theo `DESIGN_SYSTEM.md` mục 5.
- `MUST` giữ hoạt động ở cả ba chế độ nền, dùng bản logo cho nền tối theo
  [`BRAND_ASSETS.md`](BRAND_ASSETS.md) mục 4.

### 13.3. Thông điệp

- `MUST` viết bằng tiếng Việt tự nhiên, nói người dùng **làm gì tiếp theo**.
- `MUST NOT` in mã lỗi kỹ thuật hay dấu vết ngăn xếp ra giao diện.
- Bảng thông điệp chuẩn, dùng nguyên văn:

| Tình huống | Nội dung |
| --- | --- |
| Sai email hoặc mật khẩu | `Email hoặc mật khẩu không đúng.` |
| Tài khoản bị khóa tạm | `Tài khoản tạm khóa do đăng nhập sai nhiều lần. Thử lại sau {n} phút.` |
| Chưa xác minh, còn hạn | `Tài khoản chưa xác minh. Còn {n} ngày trước khi bị hạn chế.` |
| Chưa xác minh, quá hạn | `Tài khoản đã bị hạn chế do chưa xác minh email. Xác minh để dùng lại đầy đủ.` |
| Gửi lại thư xác minh | `Đã gửi. Kiểm tra hộp thư của {email}, cả mục Spam.` |
| Quên mật khẩu | `Nếu email này có tài khoản, chúng tôi đã gửi hướng dẫn tới đó.` |

---

## 14. Sản phẩm chạy ngoại tuyến

Áp cho hạng C, và cho hạng B khi mất mạng.

### 14.1. Nguyên tắc

- Chức năng lõi `MUST` chạy đủ ở **chế độ khách**, không tài khoản, không mạng.
  Với `tsudev-swico`, rà quét và xuất báo cáo là chức năng lõi.
- Nút đăng nhập `MUST` nói rõ nó mở thêm gì. Ví dụ đúng: `Đăng nhập tài khoản
  tsudev để đồng bộ báo cáo và lưu hồ sơ máy`. Ví dụ sai: một nút `Đăng nhập`
  trống nghĩa.
- `MUST NOT` gọi mạng ngầm khi người dùng chưa đăng nhập. Phần mềm rà quét máy
  tính mà tự gọi ra ngoài là điều đầu tiên bị công cụ bảo mật gắn cờ, và đúng ra
  phải như vậy.
- `MUST` có công tắc **Chế độ ngoại tuyến hoàn toàn** trong Cài đặt, khi bật thì
  chặn mọi kết nối ra, kể cả kiểm tra bản cập nhật.

### 14.2. Token khi ngoại tuyến

- Refresh token `MUST` lưu trong kho bí mật của hệ điều hành: Windows Credential
  Manager, macOS Keychain, Secret Service trên Linux. `MUST NOT` lưu vào tệp
  cấu hình, kể cả có mã hóa bằng khóa nằm cạnh nó.
- Ngoại tuyến, phiên `MAY` được coi là còn hiệu lực tối đa **7 ngày** kể từ lần
  xác thực trực tuyến gần nhất. Quá hạn thì `MUST` quay về chế độ khách, `MUST
  NOT` chặn chức năng lõi.
- Tài nguyên đã tải về `MUST` xóa được bằng một nút **Đăng xuất và xóa dữ liệu
  cục bộ**.

---

## 15. Chuẩn API

Mọi endpoint gắn phiên bản theo [`VERSIONING.md`](VERSIONING.md) mục 4.

```
POST   /api/v1/auth/register            đăng ký bằng email
POST   /api/v1/auth/login               đăng nhập bằng email
POST   /api/v1/auth/logout              thu hồi phiên hiện tại
POST   /api/v1/auth/refresh             xoay vòng token
GET    /api/v1/auth/providers/{p}       bắt đầu luồng OIDC (p = google|github)
GET    /api/v1/auth/providers/{p}/callback
POST   /api/v1/auth/verify/resend       gửi lại thư xác minh
POST   /api/v1/auth/verify/confirm      xác nhận token xác minh
POST   /api/v1/auth/password/forgot     yêu cầu đặt lại
POST   /api/v1/auth/password/reset      đặt mật khẩu mới
GET    /api/v1/me                       hồ sơ + trạng thái xác minh
PATCH  /api/v1/me                       sửa hồ sơ
GET    /api/v1/me/sessions              danh sách phiên
DELETE /api/v1/me/sessions/{id}         thu hồi một phiên
GET    /api/v1/me/identities            danh sách liên kết
DELETE /api/v1/me/identities/{id}       gỡ liên kết
POST   /api/v1/me/deletion              yêu cầu xóa tài khoản
DELETE /api/v1/me/deletion              hủy yêu cầu xóa
```

`GET /api/v1/me` `MUST` trả đủ khối trạng thái sau - giao diện dựa hoàn toàn vào
nó, `MUST NOT` để giao diện tự tính hạn:

```json
{
  "id": "018f...",
  "email": "nguoi.dung@vi.du",
  "display_name": "Người dùng",
  "avatar_url": "https://cdn.tsudev.com/avatars/018f....webp",
  "status": "active",
  "verification": {
    "state": "unverified",
    "verified_at": null,
    "grace_ends_at": "<ISO8601 UTC>",
    "days_remaining": 4,
    "restricted": false
  },
  "identities": ["google"],
  "two_factor_enabled": false
}
```

- `days_remaining` `MUST` do máy chủ tính. Đồng hồ máy khách sai là chuyện thường
  và nó sẽ sinh ra hai con số khác nhau ở hai chỗ.
- Mã lỗi `MUST` là chuỗi máy đọc được trong trường `code`, kèm `message` tiếng
  Việt để hiện thẳng ra giao diện.

---

## 16. Kiểm thử bắt buộc

Ngoài ngưỡng chung ở [`TESTING_QUALITY.md`](TESTING_QUALITY.md), luồng xác thực
`MUST` có test tự động cho đủ các ca sau. Đây là danh sách tối thiểu, không phải
danh sách đầy đủ.

1. Đăng ký, xác minh, đăng nhập - luồng thuận.
2. Token xác minh hết hạn, và token dùng lại lần hai.
3. Tài khoản chưa xác minh **trong** ân hạn: bị chặn đúng các thao tác ghi ở
   bảng mục 10.
4. Tài khoản chưa xác minh **quá** ân hạn: bị đẩy về trang xác minh.
5. Xác minh xong thì quyền khôi phục ngay, không cần đăng nhập lại.
6. Đăng nhập Google với `email_verified: false` **không** được gộp tự động.
7. Gỡ liên kết cuối cùng bị từ chối.
8. `state` sai hoặc thiếu ở đường quay về OIDC thì bị từ chối.
9. `?next=` trỏ ra ngoài tên miền bị từ chối.
10. Refresh token dùng lại lần hai thì cả cây phiên bị thu hồi.
11. Đăng nhập sai `n` lần thì bị khóa tạm và có độ trễ tăng dần.
12. Ít nhất một ca IDOR: người dùng A gọi `/api/v1/me/sessions/{id}` của B.
13. Thời gian phản hồi của `quên mật khẩu` không phân biệt email tồn tại hay
    không (chống dò tài khoản).
14. Toàn bộ luồng đăng nhập đi hết được chỉ bằng bàn phím.

---

## 17. Checklist nghiệm thu

- [ ] Đã ghi hạng sản phẩm (A/B/C) vào `README.md`.
- [ ] Ba lối vào đủ và đúng thứ tự (hạng A, B).
- [ ] Authorization Code + PKCE, có `state` và `nonce`, `redirect_uri` khớp tuyệt đối.
- [ ] Gộp tài khoản chỉ khi `email_verified` đúng.
- [ ] Không lộ tài khoản có tồn tại hay không, ở cả ba luồng.
- [ ] Huy hiệu trạng thái có đủ màu, chữ và icon; có dải thông báo khi chưa xác minh.
- [ ] Hạn 7 ngày là hằng số cấu hình được; có nhắc ngày 3, ngày 6 và khi hết hạn.
- [ ] Hạn chế được thi hành ở tầng máy chủ, trả `403` kèm mã lý do.
- [ ] Có trang phiên đăng nhập và nút đăng xuất khỏi mọi thiết bị khác.
- [ ] Đổi mật khẩu, đổi email, xóa tài khoản đều yêu cầu xác thực lại và gửi email.
- [ ] Có xuất dữ liệu và ân hạn xóa 30 ngày.
- [ ] Hạng C chạy đủ chức năng lõi ở chế độ khách, không gọi mạng ngầm.
- [ ] Token ngoại tuyến nằm trong kho bí mật của hệ điều hành.
- [ ] Đủ 14 ca kiểm thử ở mục 16.
- [ ] Đường dẫn đúng mục 13.1.
- [ ] `./scripts/check-standards.sh` đạt.

---

## 18. Tài liệu liên quan

- [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6 - mật khẩu, phiên, phân quyền
- [`ECOSYSTEM_IDENTITY.md`](ECOSYSTEM_IDENTITY.md) - trang hồ sơ, ảnh đại diện, tsudev.com
- [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) - token, component, trạng thái
- [`ACCESSIBILITY.md`](ACCESSIBILITY.md) - WCAG 2.1 AA
- [`DATA_TABLE.md`](DATA_TABLE.md) - bảng phiên đăng nhập, nhật ký kiểm toán
