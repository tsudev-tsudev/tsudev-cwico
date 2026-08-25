# TÀI SẢN THƯƠNG HIỆU (v3.0.0)

> Logo, dấu hiệu thu gọn, wordmark, favicon và icon ứng dụng của hệ sinh thái
> tsudev: bản gốc nằm ở đâu, sinh ra bằng gì, dùng dấu hiệu nào ở cỡ nào.
>
> Màu giao diện nằm ở [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md), không phải ở đây.
> Tài liệu này nói về **hình ảnh nhận diện** và **màu của chính dấu hiệu**.

## 1. Bản gốc chính thức và nơi sinh tài sản

Hệ sinh thái đã có sẵn một dây chuyền hoàn chỉnh, đặt tại repo website:

| Thứ | Đường dẫn (repo `tsudev-tsudev/tsudev`) |
| --- | --- |
| **Bản gốc** | `packages/brand/source/logo.jpeg` - `2048x2048`, nền trắng |
| Bộ favicon gốc | `packages/brand/source/favicons/` (bản xuất của RealFaviconGenerator) |
| **Dây chuyền sinh** | `packages/brand/build-assets.js` |
| Tài liệu dây chuyền | `packages/brand/README.md` |

SHA-256 của bản gốc:

```
4bf56738c0b9534a65ec3d6ba4e618a3570ae6ec4133fde1612313d4f60bdb32
```

Chạy lại dây chuyền:

```bash
npm i --no-save sharp                 # sharp không nằm trong dependency của repo
node packages/brand/build-assets.js
```

**Repo quy ước này KHÔNG giữ bản sao của bản gốc.** Hai bản gốc song song là hai
bản gốc sẽ trôi khỏi nhau. Repo nào cần thì lấy từ đường dẫn trên và đối chiếu
chuỗi băm.

`MUST NOT` sửa file trong `apps/*/public/` - chúng là **sản phẩm sinh ra**, lần
chạy lại dây chuyền sẽ ghi đè. Sửa thì sửa ở `packages/brand/source/`.

## 2. Bộ dấu hiệu và dùng ở cỡ nào

| Dấu hiệu | File sinh ra | Là gì | Dùng khi |
| --- | --- | --- | --- |
| **Logo đầy đủ** | `brand/logo-full.png` (`768x1169`) | Cú + chữ "tsudev" + tagline | Trang giới thiệu, bìa tài liệu, ảnh chia sẻ, bản in. Cạnh ngắn **từ 160px** |
| **Dấu hiệu thu gọn** | `brand/logo-mark.png` (`512x626`) | **Chỉ hình cú**, không chữ | Mặc định ở giao diện, và **bắt buộc** cho mọi icon: favicon, icon ứng dụng, khay hệ thống, avatar |
| **Wordmark ảnh** | `brand/logo-wordmark.png` (`640x192`) | Chỉ phần chữ | Chỉ nơi không chạy được chữ thật: ảnh mở đầu bản cài, chữ ký email, bản in |
| **Bản cho nền tối** | `brand/logo-full-dark.png`, `brand/logo-wordmark-dark.png` | Như trên, mực navy đổi sang trắng | Bắt buộc khi đặt trên nền tối (mục 4) |

**Dấu hiệu thu gọn là dấu hiệu mặc định trên giao diện**, không phải logo đầy đủ.
Logo đầy đủ có tagline chữ nhỏ; thu xuống dưới 160px là tagline thành vệt xám.

Bản cài đặt tham chiếu: `packages/ui/src/components/Logo.tsx` ở repo `tsudev` -
dùng `logo-mark.png` cho hình và **render chữ "tsudev" bằng text**, không dùng
ảnh wordmark.

## 3. Quy tắc đặt tên

```
tsudev-<loại>[-<biến-thể>][-<cỡ>].<đuôi>
```

- `<loại>`: `logo` (đầy đủ) | `mark` (thu gọn, không chữ) | `wordmark` | `favicon`.
- `<biến-thể>`: `dark` cho nền tối, `square` cho bản vuông, `mono` cho một màu.
- `<cỡ>`: số pixel cạnh ngắn, chỉ dùng cho bản sinh ra (`tsudev-mark-256.png`).
- Chữ thường, phân tách bằng gạch ngang ngắn `-`, không dấu, không khoảng trắng,
  không số phiên bản trong tên file.

Hai repo đang chạy dùng lược đồ tên riêng có trước quy ước này: `tsudev` dùng
`logo-full` / `logo-mark` / `logo-wordmark`, `tsudev-cwico` dùng
`tsudev-logo.png` cho chính **dấu hiệu thu gọn**. `MUST NOT` đổi tên chúng chỉ để
hợp quy ước - tên file trong `public/` là URL công khai, đổi là hỏng liên kết và
hỏng cache. Quy tắc trên áp cho **file mới**.

## 4. Biến thể theo nền

Hình cú có nền trong suốt và tự đủ tương phản trên cả ba chế độ nền
(`DESIGN_SYSTEM.md` mục 1) - dùng chung một bản, không cần bản `dark`.

Chữ thì khác. Chữ trong `logo-full.png` và `logo-wordmark.png` là **navy
`#11355A`**, chỉ đạt **1.38:1** trên nền Tối - không đọc được. Vì vậy:

- Trên nền tối, `MUST NOT` dùng `logo-full.png` hay `logo-wordmark.png`. Dùng
  `logo-full-dark.png` / `logo-wordmark-dark.png` - mực navy đã đổi sang trắng,
  đạt **17.28:1**, còn chữ cam giữ nguyên vì nó vốn đã đạt **6.66:1**.
- Nơi dựng được chữ bằng text thì vẫn `SHOULD` dùng `logo-mark.png` kèm chữ text
  theo mục 5 - cách đó tự đúng ở cả ba chế độ nền, không phải chọn file.
- Giới hạn đã biết: quầng sáng của con cú được vẽ để tan vào nền trắng, nên ở cỡ
  rất lớn trên nền tối phần đáy thân cú vẫn hơi lộ vệt sáng. Dưới 300px không nhận
  ra. Sửa hết hẳn là việc của khâu thiết kế, không phải của ngưỡng xoá nền.

## 5. Màu của dấu hiệu

Đo trực tiếp từ bản gốc:

| Vai trò trong ảnh | Mã màu |
| --- | --- |
| Chữ `tsu`, thân cú | `#11355A` |
| Chữ `dev` | `#FE7B2E` |
| Tagline | `#0F2947` |
| Mắt cú, điểm nhấn | Cam hổ phách quanh `#F3813F` |

**Wordmark `SHOULD` dựng bằng chữ thật, không phải ảnh.** Chữ co giãn theo cỡ chữ
hệ thống, sắc ở mọi DPI, chọn và tìm kiếm được, và đổi màu theo chế độ nền mà
không cần file thứ hai.

Chữ dựng bằng text `MUST` lấy màu từ token, không hard-code:

| Phần | Token |
| --- | --- |
| `tsu` | `text-primary` |
| `dev` | `text-link` |

Cả hai token đã qua cổng canh tương phản ở cả ba chế độ nền, nên cách này luôn
đạt chuẩn mà không phải đo lại.

**Chỉ khi buộc phải dùng đúng sắc cam thương hiệu** (ví dụ khớp với ảnh in đặt
cạnh), dùng bảng dưới - cam lấy thẳng từ ảnh **không** dùng được trên nền sáng:

| Nền | `dev` | Tương phản | Dùng được cho |
| --- | --- | --- | --- |
| Light / Warm | `#C2410C` | 4.68 / 4.60 | mọi cỡ chữ |
| Light / Warm | `#D2540E` | 3.78 / 3.72 | chỉ chữ lớn (từ 18px, hoặc 14px đậm) |
| Light / Warm | `#FE7B2E` (màu trong ảnh) | **2.35** | `MUST NOT` dùng cho chữ |
| Dark | `#FE7B2E` | 6.66 | mọi cỡ chữ |
| Dark | `#FFA76B` | 9.05 | mọi cỡ chữ |

**Màu dấu hiệu KHÔNG phải token giao diện.** Chúng `MUST NOT` ghi đè token ngữ
nghĩa trong `tokens/design-tokens.json`. Repo cần dải màu thương hiệu thì khai
riêng, tên khác hẳn tên token ngữ nghĩa, và chỉ dùng cho phần nhận diện.

## 6. Kích thước tối thiểu và vùng an toàn

| Dùng ở đâu | Tối thiểu |
| --- | --- |
| Logo đầy đủ (có tagline) | cạnh ngắn **160px**; dưới đó dùng dấu hiệu thu gọn |
| Logo đầy đủ khi in | **30mm** |
| Dấu hiệu thu gọn, trên màn hình | cạnh ngắn **16px** |
| Favicon | `.ico` nhiều lớp 16/32/48, `MUST NOT` co một ảnh PNG duy nhất |

**Vùng an toàn**: chừa quanh dấu hiệu khoảng trống tối thiểu bằng **1/4 cạnh
ngắn** của chính nó. Không đặt chữ, viền, hay ảnh khác vào vùng này.

## 7. Sinh bộ icon

Một bản gốc, một script, mọi bản sinh ra từ đó. `MUST NOT` cắt tay từng cỡ.

- Nguồn để sinh icon `MUST` là **dấu hiệu thu gọn**, không phải logo đầy đủ.
- Nguồn `MUST` có cạnh ngắn **lớn hơn hoặc bằng cỡ icon lớn nhất sẽ sinh ra**.
  Phóng to ảnh nhỏ rồi lưu lại không tạo thêm chi tiết nào, chỉ tạo cảm giác mờ.
- `.ico` `MUST` là file nhiều lớp `16/24/32/48/64/128/256`, không phải một ảnh
  256 đổi đuôi.
- Nội suy `MUST` dùng LANCZOS hoặc tương đương, giữ kênh alpha.

Hai bản cài đặt tham chiếu, dùng cho hai loại sản phẩm:

| Dây chuyền | Repo | Sinh ra |
| --- | --- | --- |
| `packages/brand/build-assets.js` | `tsudev` | Tài sản web: logo, favicon, avatar, ảnh chia sẻ |
| `tools/gen_icons.py` | `tsudev-cwico` | Icon desktop: bộ Tauri, tile Microsoft Store, bộ web |

## 8. Điều cấm

`MUST NOT`:

- Kéo méo, xoay, lật, cắt cụt, hay đổi tỉ lệ giữa các phần của dấu hiệu.
- Đổi màu dấu hiệu, thêm đổ bóng, viền, hay hiệu ứng chuyển màu.
- Dùng logo đầy đủ ở cỡ icon, hoặc trên nền tối (mục 4).
- Sửa file trong `apps/*/public/` thay vì sửa bản gốc rồi chạy lại dây chuyền.
- Đặt dấu hiệu lên ảnh nền rối hoặc nền tương phản thấp.
- Dùng dấu hiệu tsudev cho sản phẩm không thuộc hệ sinh thái tsudev.

## 9. Khả năng truy cập

- Dấu hiệu đi kèm chữ dựng bằng text: ảnh `MUST` khai `alt=""` và
  `aria-hidden="true"` - chữ bên cạnh đã mang nghĩa, đọc hai lần là nhiễu.
- Dấu hiệu đứng một mình mà có chức năng: `MUST` có nhãn mô tả **hành động**,
  không phải mô tả hình.
- Dấu hiệu thuần trang trí: `alt=""`.

## 10. Nợ đã biết

| Việc | Ở đâu | Ghi chú |
| --- | --- | --- |
| (không còn) | | Món nợ nguồn sinh icon dưới chuẩn đã trả ở `v2.7.0`; món nợ bản logo cho nền tối đã trả ở `v2.8.0` |

## 11. Biến thể không dùng cho sản phẩm

```
assets/brand/variants/tsudev-badge-cyan.png
```

Huy hiệu cú xanh cyan trên vòng mạch, `1024x1024`, có chữ "tsudev" nướng sẵn
trong ảnh. Tạo ngày 23/08/2026, **chưa từng được dùng ở repo nào**.

`MUST NOT` dùng file này trong bất kỳ sản phẩm nào. Nó giữ ở đây để không mất
(trước đó chỉ tồn tại trên một máy cá nhân) và để đối chiếu khi bàn về nhận diện.
Nó **không** phải bản độ phân giải cao của dấu hiệu chính thức: khác hẳn cả hình
lẫn màu, và không có bản chỉ-hình để làm icon.

Muốn chuyển hệ sinh thái sang biến thể này thì đó là một cuộc **đổi nhận diện**,
`MUST` làm đồng loạt ở website, app, favicon, avatar và ảnh chia sẻ, không phải
đổi lẻ một repo. Chi phí thật nằm ở chỗ mọi bản in, mọi ảnh chụp màn hình trong
tài liệu, và mọi icon đã phát hành đều phải làm lại.

## 12. Bộ tài sản BẮT BUỘC cho mọi project

> Thêm ở `v3.0.0`. Trước bản này, tài liệu mô tả **có những tài sản nào**; nó
> không nói **project nào phải có cái gì**. Khoảng trống đó là lý do một sản phẩm
> phát hành ra mà tab trình duyệt trống trơn.

**Quy tắc cứng:** không sản phẩm nào của hệ sinh thái được phát hành khi còn
thiếu một dòng nào trong bảng dưới đây. Đây là điều kiện phát hành, không phải
việc để dành làm sau.

### 12.1. Bảng bắt buộc theo loại sản phẩm

| Tài sản | Web | Desktop | Nguồn |
| --- | --- | --- | --- |
| `favicon.ico` nhiều lớp `16/24/32/48/64/128/256` | `MUST` | - | mục 6 |
| `favicon-96x96.png` | `MUST` | - | mục 6 |
| `apple-touch-icon.png` `180x180`, nền đặc | `MUST` | - | mục 6 |
| Icon PWA `192x192`, `512x512`, kèm bản `maskable` | `MUST` khi có `manifest` | - | mục 6 |
| `og-image.png` `1200x630` | `MUST` | - | mục 1 |
| Dấu hiệu thu gọn trong giao diện | `MUST` | `MUST` | mục 2 |
| Bản cho nền tối | `MUST` khi có chế độ Tối | `MUST` khi có chế độ Tối | mục 4 |
| Icon ứng dụng theo nền tảng (`.ico`, `.icns`, PNG bộ Linux) | - | `MUST` | mục 7 |
| Icon khay hệ thống `16/32` | - | `MUST` khi có khay | mục 6 |
| Ảnh mở đầu bản cài | - | `MUST` khi có bộ cài | mục 2 |
| Ảnh đại diện tổ chức/repo trên GitHub | `MUST` | `MUST` | dấu hiệu thu gọn |

- Toàn bộ `MUST` sinh từ **bản gốc chính thức** ở mục 1, qua đúng một trong hai
  dây chuyền ở mục 7. `MUST NOT` cắt tay, `MUST NOT` sao chép tài sản đã sinh từ
  repo khác rồi phóng to.
- Siêu dữ liệu khai báo các tài sản này (`<link rel="icon">`, `manifest`,
  Open Graph) `MUST` theo [`ECOSYSTEM_IDENTITY.md`](ECOSYSTEM_IDENTITY.md) mục 2.
- Thiếu tài sản nào thì `MUST` mở việc trong `logs/STATE.md` của repo đó **trước**
  khi phát hành, không phải sau.

### 12.2. Cổng kiểm

Repo con `MUST` để cổng kiểm CI chặn khi thiếu tài sản bắt buộc. Bản kiểm tối
thiểu, thêm vào `.github/workflows/standards.yml`:

```yaml
- name: Kiem tra tai san nhan dien
  run: |
    set -e
    for f in favicon.ico favicon-96x96.png apple-touch-icon.png og-image.png; do
      find . -path ./node_modules -prune -o -name "$f" -print | grep -q . \
        || { echo "Thieu tai san bat buoc: $f (BRAND_ASSETS.md muc 12)"; exit 1; }
    done
```

Sản phẩm desktop thay danh sách trên bằng danh sách icon của nền tảng mình.

### 12.3. Nhận diện là MỘT, không nhân bản

- `MUST NOT` mỗi sản phẩm tự vẽ một logo riêng, một bảng màu riêng, hay một biến
  thể "cho hợp với sản phẩm này".
- Sản phẩm cần dấu hiệu riêng để phân biệt trong khay hệ thống thì `MAY` thêm một
  **huy hiệu phụ nhỏ ở góc** dấu hiệu chung, `MUST NOT` thay dấu hiệu chung.
- Chữ "tsudev" dựng bằng text `MUST` lấy màu từ bảng ở mục 5. Hai sản phẩm ra hai
  màu khác nhau là lỗi, không phải phong cách - xem việc `TS-8` trong
  `logs/STATE.md`.

---

## 13. Kho lưu và kho dùng

Hai kho, đừng lẫn:

| Kho | Đường dẫn | Dùng được trong sản phẩm? |
| --- | --- | --- |
| **Kho dùng** | Tài sản sinh ra bởi dây chuyền ở mục 1 và mục 7 | Có, và chỉ những thứ này |
| **Kho lưu** | `assets/brand/variants/` ở repo quy ước | **Không**, xem mục 11 |

Ai đó đưa cho bạn một file ảnh logo rời và bảo "dùng cái này": việc đầu tiên
`MUST` là đối chiếu chuỗi băm với bản gốc ở mục 1. Không khớp thì nó là **biến
thể**, thuộc kho lưu, và `MUST NOT` đi vào sản phẩm cho tới khi có quyết định đổi
nhận diện được ghi vào `logs/STATE.md` mục "Quyết định quan trọng".

Đã xảy ra thật: file `logo-tsudev.png` đặt ở thư mục làm việc, tên nghe như bản
chính thức, thực chất là **bản sao đúng từng byte** của
`assets/brand/variants/tsudev-badge-cyan.png` - biến thể `MUST NOT` dùng. Tên file
không phải bằng chứng. Chuỗi băm mới là bằng chứng.

---

## 14. Checklist trước khi thêm hoặc đổi tài sản thương hiệu

1. Sửa ở `packages/brand/source/`, không sửa file trong `public/`.
2. Chạy lại dây chuyền, không cắt tay.
3. Chọn đúng dấu hiệu theo cỡ hiển thị (mục 2 và 6).
4. Nguồn sinh icon có cạnh ngắn lớn hơn hoặc bằng cỡ icon lớn nhất.
5. Chữ dựng bằng text lấy màu từ token (mục 5).
6. Không hard-code màu dấu hiệu vào chỗ lẽ ra phải dùng token.
7. Ảnh có `alt` đúng vai trò theo mục 9.
8. File ảnh lớn không cần lúc chạy `MUST` vào `.gitignore`.
9. Bộ tài sản bắt buộc ở mục 12 đã đủ, và cổng kiểm CI của repo đã canh nó.
10. File ảnh nhận được từ bên ngoài đã đối chiếu chuỗi băm theo mục 13.
