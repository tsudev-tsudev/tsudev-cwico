# NHẬN DIỆN HỆ SINH THÁI TSUDEV (v3.0.0)

> `https://tsudev.com` là nhà chung của mọi dịch vụ, sản phẩm và công cụ trong hệ
> sinh thái. Tài liệu này quy định mọi project **tự khai mình thuộc về đâu** như
> thế nào: siêu dữ liệu bắt buộc, cách nối về nhà chung, chuẩn ảnh đại diện tài
> khoản, và bố cục trang hồ sơ cá nhân.
>
> Hình ảnh nhận diện (logo, favicon, icon) ở [`BRAND_ASSETS.md`](BRAND_ASSETS.md).
> Màu và component ở [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md). Tài khoản và xác
> minh ở [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md). Tài liệu này nói về **sự
> thuộc về** và **hồ sơ người dùng**.

## Mục lục

1. [tsudev.com là gì](#1-tsudevcom-là-gì)
2. [Siêu dữ liệu bắt buộc của mọi project](#2-siêu-dữ-liệu-bắt-buộc-của-mọi-project)
3. [Tên gọi và cách viết](#3-tên-gọi-và-cách-viết)
4. [Nối về nhà chung](#4-nối-về-nhà-chung)
5. [Tài sản nhận diện bắt buộc](#5-tài-sản-nhận-diện-bắt-buộc)
6. [Ảnh đại diện tài khoản](#6-ảnh-đại-diện-tài-khoản)
7. [Trang hồ sơ cá nhân](#7-trang-hồ-sơ-cá-nhân)
8. [Giọng điệu và văn bản](#8-giọng-điệu-và-văn-bản)
9. [Checklist nghiệm thu](#9-checklist-nghiệm-thu)

---

## 1. tsudev.com là gì

`tsudev.com` giữ ba vai, và ba vai này `MUST` được nhắc đúng trong mọi tài liệu,
mọi trang giới thiệu, mọi bản mô tả repo:

| Vai | Nghĩa |
| --- | --- |
| **Cổng sản phẩm** | Nơi liệt kê và dẫn tới toàn bộ dịch vụ, sản phẩm, công cụ của hệ sinh thái |
| **Nhà cung cấp danh tính** | Một tài khoản dùng chung cho mọi sản phẩm, theo [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md) mục 2 |
| **Nguồn nhận diện** | Nơi công bố logo, bảng màu, và bản mô tả chính thức của thương hiệu |

Mô tả chuẩn, dùng nguyên văn khi cần một câu giới thiệu:

> **tsudev** là hệ sinh thái sản phẩm và công cụ phần mềm tại
> [tsudev.com](https://tsudev.com).

Bản dài, dùng cho trang giới thiệu và mô tả kho mã:

> **tsudev** là hệ sinh thái sản phẩm và công cụ phần mềm tại
> [tsudev.com](https://tsudev.com). Mọi sản phẩm trong hệ sinh thái dùng chung một
> tài khoản, một bộ quy ước kỹ thuật, và một bộ nhận diện.

`MUST NOT` tự nghĩ ra bản mô tả khác cho từng repo. Một câu giới thiệu lệch ở một
repo là một mảnh nhận diện lệch.

---

## 2. Siêu dữ liệu bắt buộc của mọi project

### 2.1. Ở kho mã

Mọi repo thuộc hệ sinh thái `MUST` có:

- **Mô tả kho mã** (`description` trên GitHub): một câu tiếng Việt nói sản phẩm
  làm gì, `MUST` kết thúc bằng ` | tsudev.com`.
- **Website** (`homepage` trên GitHub): trỏ tới trang sản phẩm trên `tsudev.com`,
  hoặc chính `https://tsudev.com` nếu chưa có trang riêng.
- **Nhãn** (`topics`): `MUST` có `tsudev`.
- `README.md` `MUST` mở đầu bằng dấu hiệu thu gọn, tên sản phẩm, một câu mô tả,
  rồi một dòng: `Một sản phẩm của [tsudev](https://tsudev.com).`
- `README.md` `MUST` ghi **hạng sản phẩm** (A/B/C) theo
  [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md) mục 1.

### 2.2. Ở ứng dụng web

`<head>` của mọi trang `MUST` có đủ:

```html
<title>{Tên trang} - {Tên sản phẩm} | tsudev</title>
<meta name="description" content="{mô tả trang, 120 tới 160 ký tự}" />
<meta name="theme-color" content="{token bg-base theo chế độ nền}" />
<link rel="canonical" href="https://{ten-mien}/{duong-dan}" />

<meta property="og:site_name" content="tsudev" />
<meta property="og:type" content="website" />
<meta property="og:title" content="{Tên trang} - {Tên sản phẩm}" />
<meta property="og:description" content="{mô tả}" />
<meta property="og:image" content="https://{ten-mien}/og-image.png" />
<meta property="og:url" content="https://{ten-mien}/{duong-dan}" />
<meta name="twitter:card" content="summary_large_image" />
```

- `og:site_name` `MUST` là chuỗi `tsudev` ở **mọi** sản phẩm, kể cả sản phẩm có
  tên miền riêng. Đó là thứ nối các sản phẩm lại với nhau khi được chia sẻ.
- Khuôn tiêu đề `MUST` đúng thứ tự trên. Trang chủ của sản phẩm rút gọn thành
  `{Tên sản phẩm} | tsudev`.
- `og:image` `MUST` là `1200x630`, sinh từ dây chuyền ở
  [`BRAND_ASSETS.md`](BRAND_ASSETS.md) mục 1. `MUST NOT` cắt tay từ ảnh chụp
  màn hình.
- `MUST` có `manifest.webmanifest` với `name`, `short_name`, `theme_color`,
  `background_color` lấy từ token, và bộ icon theo `BRAND_ASSETS.md` mục 6.

### 2.3. Ở ứng dụng desktop

Siêu dữ liệu bản cài (`Company`, `Product`, `Copyright`, `About`) `MUST` ghi:

```
Nhà phát hành:  tsudev
Trang chủ:      https://tsudev.com
Bản quyền:      Copyright (c) {năm} tsudev
```

Hộp thoại **Giới thiệu** `MUST` có dấu hiệu, tên sản phẩm, chuỗi phiên bản theo
[`VERSIONING.md`](VERSIONING.md), và liên kết mở được tới `https://tsudev.com`.

---

## 3. Tên gọi và cách viết

| Đúng | Sai |
| --- | --- |
| `tsudev` | `TsuDev`, `TSUDEV`, `Tsudev`, `tsu dev` |
| `tsudev-swico` | `TSUDEV SWICO`, `Swico`, `tsudev swico` |
| `tsudev.com` | `www.tsudev.com`, `Tsudev.com` |

- Tên thương hiệu **luôn viết thường**, kể cả ở đầu câu. Đây là chủ ý, không
  phải lỗi chính tả.
- Tên sản phẩm dạng máy đọc: `tsudev-<san-pham>`, chữ thường, nối bằng gạch ngang
  ngắn `-`. Dùng cho tên repo, tên gói, tên file cài, khóa cấu hình.
- Tên hiển thị cho người đọc `MAY` bỏ tiền tố khi ngữ cảnh đã rõ (`SWICO` trong
  chính giao diện của nó), nhưng tiêu đề trang `MUST` giữ đủ theo mục 2.2.
- Tên miền chính `MUST` là `tsudev.com` không có `www.`. `www.tsudev.com`
  `MUST` chuyển hướng `301` về bản không `www`.
- `MUST` dùng `https://`. `http://` `MUST` chuyển hướng `301`, kèm `HSTS` theo
  [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 7.2.

---

## 4. Nối về nhà chung

- Chân trang của mọi sản phẩm web `MUST` có dấu hiệu tsudev kèm liên kết
  `https://tsudev.com`, và dòng bản quyền `(c) {năm} tsudev`.
- Chân trang `MUST` có liên kết tới **Điều khoản sử dụng** và **Chính sách riêng
  tư**. Hai trang này ở `tsudev.com` và dùng chung cho cả hệ sinh thái; sản phẩm
  `MUST NOT` tự viết bản riêng trừ khi có yêu cầu pháp lý riêng, và khi đó `MUST`
  ghi rõ phần nào khác.
- Sản phẩm có nhiều hơn một trang `SHOULD` có menu **Sản phẩm khác** dẫn về trang
  danh mục ở `tsudev.com`.
- Liên kết ra ngoài `MUST` có `rel="noopener noreferrer"` khi mở tab mới.
- `MUST NOT` nhúng khung `iframe` của `tsudev.com` vào sản phẩm khác. Trang chủ
  `MUST` đặt `X-Frame-Options: DENY` theo `SECURITY_BASELINE.md` mục 7.2.

---

## 5. Tài sản nhận diện bắt buộc

**Quy tắc cứng:** không sản phẩm nào được phát hành mà thiếu bộ tài sản dưới đây.
Thiếu favicon là thiếu nhận diện, và tab trình duyệt trống là thứ người dùng thấy
đầu tiên.

| Tài sản | Bắt buộc với | Nguồn |
| --- | --- | --- |
| `favicon.ico` nhiều lớp `16/24/32/48/64/128/256` | Mọi web | `BRAND_ASSETS.md` mục 6 |
| `favicon-96x96.png`, `apple-touch-icon.png` `180x180` | Mọi web | như trên |
| Icon PWA `192x192` và `512x512` (thường + maskable) | Web có `manifest` | như trên |
| `og-image.png` `1200x630` | Mọi web | `BRAND_ASSETS.md` mục 1 |
| Icon ứng dụng theo nền tảng | Mọi desktop | `BRAND_ASSETS.md` mục 7 |
| Ảnh mở đầu bản cài | Desktop có bộ cài | `BRAND_ASSETS.md` mục 2 |
| Ảnh đại diện tổ chức trên GitHub | Mọi repo | dấu hiệu thu gọn |

- Toàn bộ `MUST` sinh từ **bản gốc chính thức** và **dây chuyền** ghi ở
  [`BRAND_ASSETS.md`](BRAND_ASSETS.md) mục 1. `MUST NOT` cắt tay, `MUST NOT` lấy
  ảnh từ một repo khác rồi phóng to.
- Nền tối `MUST` dùng bản dark theo `BRAND_ASSETS.md` mục 4.
- Biến thể ở `assets/brand/variants/` `MUST NOT` dùng trong sản phẩm. Đó là kho
  lưu, không phải kho dùng.

---

## 6. Ảnh đại diện tài khoản

Mọi sản phẩm có tài khoản `MUST` cài đặt đúng bộ quy ước này. Ảnh đại diện xuất
hiện ở mọi sản phẩm, nên nó `MUST` hành xử giống hệt nhau ở mọi sản phẩm.

### 6.1. Nguồn và thứ tự ưu tiên

1. Ảnh người dùng đã tải lên.
2. Ảnh từ nhà cung cấp liên kết (Google/GitHub) tại lần đăng nhập đầu, **sao chép
   về kho của hệ sinh thái một lần** rồi thôi.
3. **Ảnh chữ cái** sinh tại chỗ (mục 6.4).

- `MUST NOT` nhúng thẳng URL ảnh của Google/GitHub vào giao diện. Nó rò lượt truy
  cập của người dùng sang bên thứ ba trên mọi trang, và nó hỏng khi bên kia đổi
  đường dẫn.
- `MUST NOT` dùng Gravatar. Nó gửi băm email của người dùng ra ngoài.

### 6.2. Tải lên

| Ràng buộc | Giá trị |
| --- | --- |
| Định dạng nhận vào | `.jpg`, `.jpeg`, `.png`, `.webp` |
| Dung lượng tối đa | `5 MB` |
| Cạnh nhỏ nhất | `128px` |
| Cạnh lớn nhất nhận vào | `4096px` |
| Định dạng lưu | `WebP`, chất lượng `82` |
| Các cỡ sinh ra | `512`, `192`, `96`, `48` (vuông) |

- `MUST` kiểm **kiểu thật của tệp bằng chữ ký byte đầu (magic number)**, không
  tin phần mở rộng, không tin `Content-Type` do máy khách gửi.
- `MUST` **giải mã và mã hóa lại** ảnh phía máy chủ. Việc này vừa chuẩn hóa định
  dạng vừa loại bỏ tải trọng độc nhúng trong tệp ảnh.
- `MUST` xóa toàn bộ siêu dữ liệu EXIF, đặc biệt là **tọa độ GPS**. Ảnh chụp bằng
  điện thoại mang theo vị trí nhà của người dùng.
- `MUST` áp dụng phép xoay theo EXIF **trước khi** xóa EXIF, nếu không ảnh chụp
  dọc sẽ nằm ngang.
- `MUST` từ chối ảnh động (`GIF` động, `WebP` động) và `SVG`. `SVG` là tài liệu
  có thể chứa mã, không phải ảnh.
- `MUST` lưu ở kho tách khỏi mã nguồn (đối tượng lưu trữ hoặc CDN), phục vụ dưới
  tên tệp **không đoán được** và `Content-Disposition: inline` kèm
  `Content-Type` cố định là `image/webp`.
- `MUST` giới hạn tần suất tải lên: đề xuất **10 lần / giờ / tài khoản**.
- Tài khoản **chưa xác minh** `MUST NOT` tải ảnh lên được, theo ma trận quyền ở
  `AUTH_AND_ACCOUNT.md` mục 10.

### 6.3. Giao diện cắt ảnh và các thao tác

Ba thao tác `MUST` có đủ, cùng tên gọi, ở mọi sản phẩm: **Tải ảnh lên**,
**Đổi ảnh**, **Xóa ảnh**.

```
+---------------------------------------------------+
|  Ảnh đại diện                                     |
|                                                   |
|    (o o)     [ Tải ảnh lên ]                      |
|   (  96  )   [ Xóa ảnh     ]                      |
|                                                   |
|   JPG, PNG hoặc WebP. Tối đa 5 MB.                |
+---------------------------------------------------+
```

- Vùng xem trước `MUST` là hình **tròn** đường kính `96px`, đúng như ảnh sẽ hiện
  ở nơi khác. Xem trước hình vuông rồi hiện ra hình tròn là cách người dùng bị
  cắt mất đỉnh đầu.
- Sau khi chọn tệp, `MUST` mở hộp thoại **cắt ảnh** với khung tròn tỉ lệ `1:1`,
  có thu phóng và kéo. `MUST NOT` tự cắt giữa rồi lưu luôn.
- Hộp thoại cắt `MUST` dùng component Modal của `DESIGN_SYSTEM.md` mục 5, nút
  `Hủy` và `Lưu ảnh` ở chân, `Lưu ảnh` là Primary ngoài cùng bên phải.
- `MUST` hỗ trợ kéo thả tệp vào vùng xem trước, và `MUST` vẫn có nút bấm cho
  người không dùng chuột.
- `MUST` hiện thanh tiến trình khi tải lên, và cập nhật ảnh **ngay khi xong**,
  không bắt tải lại trang.
- **Xóa ảnh** `MUST` hỏi xác nhận một lần, rồi quay về ảnh chữ cái (mục 6.4).
  `MUST NOT` quay về một hình bóng người xám vô danh.
- Xóa `MUST` xóa **mọi cỡ đã sinh** khỏi kho lưu trữ, không chỉ gỡ tham chiếu
  trong cơ sở dữ liệu.
- Lỗi `MUST` nói rõ nguyên nhân và ngưỡng: `Ảnh vượt quá 5 MB. Chọn ảnh nhỏ hơn.`
  `MUST NOT` báo `Tải lên thất bại` cụt lủn.

### 6.4. Ảnh chữ cái

Khi không có ảnh, `MUST` sinh ảnh chữ cái tại chỗ, không gọi dịch vụ ngoài:

- Chữ: **một** ký tự đầu của tên hiển thị, viết hoa. Tên rỗng thì lấy ký tự đầu
  của email. Ký tự tiếng Việt có dấu `MUST` giữ nguyên dấu (`Đ`, `Ơ`).
- Nền: chọn tất định từ băm của `id` người dùng, trong **bộ 8 màu** lấy từ token
  ngữ nghĩa của `DESIGN_SYSTEM.md`. Cùng một người `MUST` luôn ra cùng một màu ở
  mọi sản phẩm.
- Chữ `MUST` đạt tương phản tối thiểu `4.5:1` với nền đã chọn, kiểm bằng
  `scripts/check-contrast.mjs`.
- `MUST NOT` dùng ảnh hình bóng người mặc định. Ảnh chữ cái nhận diện được, ảnh
  bóng người thì không.

### 6.5. Hiển thị

| Nơi | Đường kính |
| --- | --- |
| Menu tài khoản ở thanh điều hướng | `32px` |
| Danh sách, bảng, bình luận | `40px` |
| Đầu trang hồ sơ | `96px` |
| Trang hồ sơ công khai | `128px` |

- Luôn hình **tròn**, luôn `object-fit: cover`.
- `MUST` có `alt` là tên hiển thị của người dùng khi ảnh mang nghĩa; `alt=""` khi
  nó chỉ đi kèm tên đã hiện bằng chữ ngay cạnh.
- `MUST` có ảnh giữ chỗ cùng kích thước trong lúc tải, để không giật bố cục.
- `MUST` dùng `loading="lazy"` cho ảnh đại diện trong danh sách dài.

---

## 7. Trang hồ sơ cá nhân

Mục tiêu: ngăn nắp, gọn gàng, ai cũng dùng được. Một cột, nhóm rõ ràng, mỗi nhóm
một việc.

### 7.1. Bố cục

Đường dẫn theo [`AUTH_AND_ACCOUNT.md`](AUTH_AND_ACCOUNT.md) mục 13.1.

```
+------------------------------------------------------------------+
|  Tài khoản                                                       |  28px/600
+---------------------+--------------------------------------------+
|  Hồ sơ           >  |   +--------------------------------------+ |
|  Bảo mật            |   |   ( ảnh )  Nguyễn Văn A              | |
|  Phiên đăng nhập    |   |    96px    a@vi.du                   | |
|  Dữ liệu            |   |            [ Đã xác minh ]           | |
|                     |   +--------------------------------------+ |
|                     |                                            |
|                     |   Thông tin cá nhân                        |  18px/600
|                     |   Tên hiển thị   [                  ]      |
|                     |   Giới thiệu     [                  ]      |
|                     |                          [ Lưu thay đổi ]  |
|                     |                                            |
|                     |   Ảnh đại diện                             |
|                     |   ( xem trước )  [ Tải ảnh lên ] [ Xóa ]    |
+---------------------+--------------------------------------------+
```

- Cột trái là menu dọc, rộng `220px`, item cao `40px`, mục đang xem có nền
  `bg-subtle` và viền trái `2px` màu `primary`.
- Cột phải rộng tối đa `720px`. Rộng hơn thì dòng chữ dài quá tầm mắt, đúng theo
  [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) mục 3.
- Dưới `900px`, menu dọc `MUST` thành hàng thẻ (tab) cuộn ngang ở đầu trang.
- Mỗi nhóm là một thẻ nền `bg-surface`, `radius-lg`, viền `border`, đệm trong
  `24px`, cách nhau `16px`.
- Thẻ đầu trang `MUST` có ảnh đại diện `96px`, tên hiển thị `20px/600`, email
  `14px` màu `text-muted`, và **huy hiệu trạng thái xác minh** theo
  `AUTH_AND_ACCOUNT.md` mục 8.4.

### 7.2. Các nhóm và thứ tự

| Trang | Nhóm, đúng thứ tự |
| --- | --- |
| **Hồ sơ** | Thẻ đầu trang; Thông tin cá nhân; Ảnh đại diện; Tùy chọn hiển thị (chế độ nền, ngôn ngữ) |
| **Bảo mật** | Mật khẩu; Xác thực hai lớp; Tài khoản liên kết (Google/GitHub); Hoạt động gần đây |
| **Phiên đăng nhập** | Bảng thiết bị đang đăng nhập; nút Đăng xuất khỏi mọi thiết bị khác |
| **Dữ liệu** | Xuất dữ liệu; Xóa tài khoản |

- Bảng ở trang **Phiên đăng nhập** và **Hoạt động gần đây** `MUST` tuân
  [`DATA_TABLE.md`](DATA_TABLE.md), kể cả bộ chọn số bản ghi ở góc dưới bên trái.
- Nhóm **Xóa tài khoản** `MUST` là nhóm cuối cùng, có viền `danger`, tiêu đề
  `Vùng nguy hiểm`, và nút Danger. `MUST NOT` đặt xen giữa các nhóm thường.

### 7.3. Quy tắc biểu mẫu

- Mỗi nhóm `MUST` có nút **Lưu thay đổi** riêng ở góc dưới bên phải của chính
  nhóm đó. `MUST NOT` có một nút lưu chung cho cả trang: người dùng không biết
  mình vừa lưu những gì.
- Nút `Lưu thay đổi` `MUST` ở trạng thái `Disabled` khi chưa có gì đổi.
- Lưu xong `MUST` hiện Toast `Đã lưu` theo `DESIGN_SYSTEM.md` mục 5, và `MUST`
  giữ tiêu điểm tại chỗ.
- Có thay đổi chưa lưu mà rời trang `MUST` hỏi xác nhận.
- Trường chỉ đọc (email chính, ngày tạo tài khoản) `MUST` hiện dạng chữ kèm nút
  hành động bên cạnh, `MUST NOT` là ô nhập bị vô hiệu - ô xám khiến người dùng
  bấm mãi không hiểu vì sao.
- Mỗi trường `MUST` có nhãn nhìn thấy được. `MUST NOT` chỉ dùng placeholder làm
  nhãn: nó biến mất ngay khi bắt đầu gõ.
- Tên hiển thị: tối đa `50` ký tự. Giới thiệu: tối đa `200` ký tự, có bộ đếm
  hiện khi còn dưới `20` ký tự.
- Toàn trang `MUST` dùng được chỉ bằng bàn phím và `MUST` đạt WCAG 2.1 AA theo
  [`ACCESSIBILITY.md`](ACCESSIBILITY.md).

### 7.4. Hồ sơ công khai

Sản phẩm có hồ sơ công khai (`/nguoi-dung/{ten-dinh-danh}`):

- Mặc định `MUST` chỉ công khai: ảnh đại diện, tên hiển thị, giới thiệu, ngày
  tham gia (`DD/MM/YYYY`), nội dung công khai của người đó.
- Email `MUST NOT` bao giờ công khai, kể cả người dùng bật lên.
- `MUST` có công tắc `Hiện hồ sơ công khai`, mặc định **tắt**.
- Hồ sơ của tài khoản chưa xác minh `MUST NOT` hiện công khai. Đây là lớp chống
  hồ sơ rác dùng để phát tán liên kết.

---

## 8. Giọng điệu và văn bản

- Tiếng Việt, xưng hô trung tính, gọi người dùng là **bạn**.
- Câu ngắn, nói **làm gì tiếp theo**, không dùng văn mẫu.
- Thuật ngữ dùng thống nhất toàn hệ sinh thái:

| Dùng | Không dùng |
| --- | --- |
| Đăng nhập / Đăng xuất | Đăng nhập / Thoát, Sign in |
| Đăng ký | Tạo tài khoản, Đăng ký ngay |
| Ảnh đại diện | Avatar, Hình đại diện |
| Hồ sơ | Profile, Thông tin cá nhân (làm tên trang) |
| Xác minh tài khoản | Kích hoạt tài khoản, Verify |
| Bản ghi | Record, Mục, Dòng dữ liệu |
| Lưu thay đổi | Cập nhật, Submit |

- Ngày `DD/MM/YYYY`, ngày giờ `HH:mm DD/MM/YYYY`, theo `AGENTS.md` mục 7.
- Chỉ dùng gạch ngang ngắn `-` trong mọi chuỗi hiển thị, theo `AGENTS.md` mục 7.

---

## 9. Checklist nghiệm thu

- [ ] Mô tả kho mã kết thúc bằng ` | tsudev.com`, có `homepage` và nhãn `tsudev`.
- [ ] `README.md` có dấu hiệu, câu giới thiệu chuẩn, và hạng sản phẩm A/B/C.
- [ ] `<head>` đủ `title`, `description`, `canonical`, `theme-color`, khối
      Open Graph với `og:site_name` là `tsudev`.
- [ ] Có đủ bộ tài sản nhận diện ở mục 5, sinh từ dây chuyền chính thức.
- [ ] Chân trang có dấu hiệu, liên kết `tsudev.com`, Điều khoản, Chính sách riêng tư.
- [ ] Ảnh đại diện: kiểm magic number, mã hóa lại, xóa EXIF, từ chối SVG và ảnh động.
- [ ] Có đủ Tải ảnh lên / Đổi ảnh / Xóa ảnh, có hộp thoại cắt khung tròn.
- [ ] Không có ảnh thì ra ảnh chữ cái, không phải hình bóng người.
- [ ] Trang hồ sơ đúng bố cục mục 7.1, đúng thứ tự nhóm mục 7.2.
- [ ] Mỗi nhóm có nút lưu riêng; Vùng nguy hiểm nằm cuối.
- [ ] Bảng trong hồ sơ tuân `DATA_TABLE.md`.
- [ ] Thuật ngữ đúng bảng mục 8.
- [ ] `./scripts/check-standards.sh` đạt.
