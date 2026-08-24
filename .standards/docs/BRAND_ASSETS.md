# TÀI SẢN THƯƠNG HIỆU (v2.4.0)

> Logo, wordmark, favicon và icon ứng dụng của hệ sinh thái tsudev: đặt ở đâu,
> đặt tên thế nào, dùng biến thể nào trên nền nào, nhỏ đến mức nào thì phải đổi
> biến thể.
>
> Màu giao diện nằm ở [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md), không phải ở đây.
> Tài liệu này chỉ nói về **hình ảnh nhận diện** và **màu của chính logo**.

## 1. Vì sao cần quy ước riêng

Logo là thứ duy nhất đi qua mọi bề mặt của hệ sinh thái: web, cửa sổ desktop,
tab trình duyệt, khay hệ thống, cửa hàng ứng dụng, chữ ký email, file cài đặt.
Mỗi bề mặt đòi một kích thước và một nền khác nhau. Không có chuẩn thì mỗi repo
tự cắt một bản, và sau vài tháng không còn ai biết bản nào là gốc.

Trạng thái thực tế trước khi có tài liệu này: `tsudev-cwico` có bộ tài sản đầy
đủ nhất và tự sinh được toàn bộ icon từ một file gốc, còn file gốc chất lượng
cao nhất (`1024x1024`) lại nằm rời trên máy cá nhân, ngoài mọi repo.

## 2. Bộ tài sản tối thiểu

Repo có giao diện `MUST` có đủ bộ này. Repo thư viện hoặc công cụ dòng lệnh
không có giao diện thì `MUST NOT` mang theo.

| File | Nội dung | Nền dùng được |
| --- | --- | --- |
| `assets/brand/tsudev-logo.png` | **Bản gốc.** Chỉ hình cú, nền trong suốt, không kèm chữ | mọi nền |
| `assets/brand/tsudev-logo-square.png` | Bản vuông sinh ra từ bản gốc, dùng làm gốc cho mọi icon | mọi nền |
| `assets/brand/tsudev-wordmark.png` | Logo kèm chữ "tsudev", màu cho nền sáng | Light, Warm |
| `assets/brand/tsudev-wordmark-dark.png` | Cùng bố cục, màu cho nền tối | Dark |

Bản dựng đem đi phát hành (favicon, icon cài đặt, tile cửa hàng) là **sản phẩm
sinh ra**, `MUST` để trong `.gitignore` hoặc sinh lại được bằng một lệnh. Hiện
`tsudev-cwico` commit cả bản sinh vì Tauri đọc chúng lúc đóng gói - chấp nhận
được, nhưng script sinh `MUST` tồn tại và chạy lại ra đúng bộ đó.

## 3. Quy tắc đặt tên

```
tsudev-<loại>[-<biến-thể>][-<cỡ>].<đuôi>
```

- `<loại>`: `logo` | `wordmark` | `icon` | `favicon`.
- `<biến-thể>`: `dark` cho bản dùng trên nền tối, `square` cho bản vuông, `mono`
  cho bản một màu. Không ghi gì nghĩa là bản mặc định cho nền sáng.
- `<cỡ>`: số pixel cạnh ngắn, chỉ dùng cho bản sinh ra (`tsudev-logo-256.png`).
- Toàn bộ chữ thường, phân tách bằng gạch ngang ngắn `-`, không dấu, không
  khoảng trắng, không số phiên bản trong tên file.

`MUST NOT` đặt tên kiểu `logo-tsudev.png`, `logo_final.png`, `logo v2.png`.
Tên đặt sai làm hỏng thứ tự sắp xếp và khiến bản gốc lẫn với bản sinh.

## 4. Biến thể theo nền

Hệ sinh thái có ba chế độ nền (`DESIGN_SYSTEM.md` mục 1). Quy tắc chọn biến thể:

| Chế độ nền | Logo (hình cú) | Wordmark |
| --- | --- | --- |
| Light | bản mặc định | `tsudev-wordmark.png` |
| Warm/Sepia | bản mặc định | `tsudev-wordmark.png` |
| Dark | bản mặc định | `tsudev-wordmark-dark.png` |

Hình cú **dùng chung một bản cho cả ba chế độ**: nó nhiều màu và có nền trong
suốt, đủ tương phản trên cả nền sáng lẫn nền tối. Chỉ phần chữ mới cần đổi màu.

**Wordmark `SHOULD` dựng bằng chữ thật, không phải ảnh.** Hai `span` màu khác
nhau - `tsu` màu xanh, `dev` màu cam - cho ra chữ co giãn theo cỡ chữ hệ thống,
nét sắc ở mọi DPI, chọn và tìm kiếm được, và tự đổi màu theo chế độ nền mà
không cần tải thêm file. Bản cài đặt tham chiếu: `ui/src/components/Brand.tsx`
ở `tsudev-cwico`. Hai file wordmark PNG chỉ dành cho nơi không chạy được chữ:
ảnh mở đầu bản cài đặt, ảnh chia sẻ mạng xã hội, chữ ký email, file in.

## 5. Màu thương hiệu

Bảng màu lấy mẫu từ chính logo: xanh mạch điện của con cú, cam hổ phách của mắt.

| Vai trò | Nền sáng | Nền tối |
| --- | --- | --- |
| Chữ `tsu` | `#1a6aa0` | `#78c0d8` |
| Chữ `dev` | `#d2540e` | `#ffa76b` |

**Màu thương hiệu KHÔNG phải token giao diện.** Chúng `MUST NOT` ghi đè token
ngữ nghĩa trong `tokens/design-tokens.json` (`primary`, `text-link`, `danger`,
...). Repo cần dải màu thương hiệu thì khai riêng, tên khác hẳn tên token ngữ
nghĩa, và chỉ dùng cho phần nhận diện. Nút bấm, liên kết, trạng thái vẫn `MUST`
đọc token như mọi nơi khác.

Cặp màu trên chỉ dùng cho **chữ wordmark cỡ lớn** (từ 18px hoặc 14px đậm trở
lên), nên áp ngưỡng WCAG 1.4.3 cho chữ lớn là **3:1**. Đặt wordmark ở cỡ nhỏ
hơn thì `MUST` dùng `text-primary` thay vì màu thương hiệu.

## 6. Kích thước tối thiểu và vùng an toàn

**Kích thước tối thiểu** - dưới ngưỡng này chi tiết mạch điện trong logo bết
lại thành một khối:

| Dùng ở đâu | Tối thiểu |
| --- | --- |
| Logo kèm wordmark | cạnh ngắn 24px |
| Chỉ logo, trên màn hình | cạnh ngắn 16px |
| Chỉ logo, khi in | 8mm |
| Favicon | dùng bản `.ico` nhiều lớp 16/32/48, không co ảnh PNG |

**Vùng an toàn**: chừa quanh logo một khoảng trống tối thiểu bằng **1/4 cạnh
ngắn** của chính nó. Không đặt chữ, viền, hay ảnh khác vào vùng này. Bản vuông
đã chừa sẵn `6%` mỗi cạnh khi sinh ra - đó là khoảng đệm để icon không chạm mép
khung bo góc của hệ điều hành, **không** thay thế vùng an toàn ở trên.

## 7. Sinh bộ icon từ bản gốc

Một bản gốc, một script, mọi bản sinh ra từ đó. `MUST NOT` cắt tay từng cỡ.

- Bản gốc `MUST` là PNG nền trong suốt, cạnh ngắn **tối thiểu 1024px**. Phóng to
  ảnh nhỏ hơn rồi lưu lại không tính là bản gốc.
- Script sinh `MUST` nội suy bằng LANCZOS (hoặc tương đương), giữ kênh alpha.
- `.ico` `MUST` là file nhiều lớp `16/24/32/48/64/128/256`, không phải một ảnh
  256 đổi đuôi.
- Bản cài đặt tham chiếu: `tools/gen_icons.py` ở `tsudev-cwico` - sinh bộ Tauri,
  tile Microsoft Store, và bộ web từ đúng một file gốc.

## 8. Điều cấm

`MUST NOT`:

- Kéo méo, xoay, lật, cắt cụt, hay đổi tỉ lệ giữa logo và chữ.
- Đổi màu logo, thêm đổ bóng nặng, viền, hay hiệu ứng chuyển màu.
- Đặt logo lên ảnh nền rối hoặc lên nền có tương phản dưới 3:1 với thân logo.
  Cần thiết thì đặt logo trên một khối nền phẳng lấy từ token `bg-surface`.
- Đặt logo trong một khối màu thương hiệu tự chế mà token không có.
- Dùng logo của tsudev cho sản phẩm không thuộc hệ sinh thái tsudev.

## 9. Khả năng truy cập

- Logo đi kèm wordmark bằng chữ: ảnh `MUST` khai `alt=""` và `aria-hidden="true"`
  - chữ bên cạnh đã mang nghĩa rồi, đọc hai lần là nhiễu.
- Logo đứng một mình mà có chức năng (bấm được, dẫn về trang chủ): `MUST` có
  nhãn văn bản hoặc `aria-label` mô tả **hành động**, không phải mô tả hình.
- Logo thuần trang trí: `alt=""`, không thêm gì.

## 10. Checklist trước khi thêm hoặc đổi tài sản thương hiệu

1. File đặt trong `assets/brand/`, tên theo mục 3.
2. Bản gốc đạt tối thiểu 1024px, nền trong suốt.
3. Bản sinh ra tạo bằng script, không cắt tay; script chạy lại được.
4. Có đủ biến thể cho nền sáng và nền tối theo mục 4.
5. Không hard-code màu thương hiệu vào chỗ lẽ ra phải dùng token.
6. Ảnh có `alt` đúng vai trò theo mục 9.
7. File ảnh lớn không cần cho lúc chạy `MUST` vào `.gitignore`
   ([`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md)).

## 11. Nợ đã biết

| Việc | Ở đâu | Ghi chú |
| --- | --- | --- |
| Bản gốc dưới chuẩn | `tsudev-cwico/assets/brand/tsudev-logo.png` | Chỉ `222x280`, đang bị phóng to lên 512 để sinh icon. Cần thay bằng bản `1024x1024` rồi chạy lại `tools/gen_icons.py` |
| Bản gốc nằm ngoài repo | máy cá nhân, tên `logo-tsudev.png` | `1024x1024`, đúng chất lượng cần, nhưng tên sai quy ước mục 3 và không repo nào giữ. Cần đưa vào `assets/brand/` của repo giữ bản gốc |

Bộ quy ước trung tâm `MUST NOT` giữ file ảnh: `MANIFEST.sha256` băm từng byte
mọi file quy ước và repo con tải toàn bộ về, nên thêm ảnh vào đây là bắt bốn
repo tải một thứ chúng không dùng.
