# TÀI SẢN THƯƠNG HIỆU (v2.6.0)

> Logo, wordmark, favicon và icon ứng dụng của hệ sinh thái tsudev: bản gốc nằm ở
> đâu, dùng dấu hiệu nào ở cỡ nào, đặt tên thế nào, biến thể nào cho nền nào.
>
> Màu giao diện nằm ở [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md), không phải ở đây.
> Tài liệu này nói về **hình ảnh nhận diện** và **màu của chính dấu hiệu**.

## 1. Bản gốc chính thức

```
assets/brand/tsudev-logo.png
```

Nằm ngay trong repo quy ước này. `1024x1024`, PNG, nền trong suốt ngoài vùng huy
hiệu. SHA-256:

```
8527a64d97817732592acb7093f1f42021a9b4e8653ef0774460b2b7dd8a20dc
```

**Vì sao để ở đây:** trước `v2.6.0` bản gốc chỉ tồn tại trên **một máy cá nhân**,
ngoài mọi repo. Máy hỏng là mất vĩnh viễn thứ không dựng lại được - mã nguồn thì
viết lại được, bản gốc thiết kế thì không.

**Vì sao không tốn gì cho repo con:** `scripts/make-manifest.sh` chỉ băm
`AGENTS.md`, `VERSION`, `SECURITY.md`, `docs`, `tokens`, `templates`, `scripts`;
`scripts/sync-standards.sh` cũng chỉ chép đúng bộ đó. `assets/` **nằm ngoài bộ
đồng bộ**, nên repo con không tải file này về cùng bản sao quy ước. Repo nào cần
thì lấy riêng một lần:

```bash
curl -fsSL -o assets/brand/tsudev-logo.png \
  https://raw.githubusercontent.com/tsudev-tsudev/tsudev-standards/main/assets/brand/tsudev-logo.png
sha256sum assets/brand/tsudev-logo.png   # MUST khớp chuỗi băm ở trên
```

Repo con `MAY` giữ một bản sao trong `assets/brand/` để build không cần mạng,
nhưng bản sao đó `MUST` khớp từng byte với bản gốc. Lệch băm nghĩa là ai đó đã
xuất lại ảnh - đó là lúc dấu hiệu bắt đầu trôi khỏi chuẩn.

**`MUST NOT` sửa đè lên bản gốc.** Đổi dấu hiệu là việc của chủ project, làm bằng
một PR riêng, có ảnh trước/sau, và `MUST` cập nhật chuỗi băm ở mục này.

## 2. Hai dấu hiệu, dùng ở hai cỡ khác nhau

Bản gốc là **huy hiệu đầy đủ**: hình cú trên vòng mạch điện, kèm chữ "tsudev"
**nướng sẵn trong ảnh** ở dải dưới.

| Dấu hiệu | Là gì | Dùng khi |
| --- | --- | --- |
| **Huy hiệu đầy đủ** (`tsudev-logo.png`) | Cú + vòng mạch + chữ "tsudev" trong ảnh | Cỡ hiển thị **từ 96px** trở lên: trang chủ, màn hình chờ, ảnh chia sẻ, bìa tài liệu, bản in |
| **Dấu hiệu thu gọn** (`tsudev-mark.png`) | **Chỉ hình, KHÔNG chữ** | Mọi cỡ **dưới 96px** và mọi icon: favicon, icon ứng dụng, khay hệ thống, avatar |

**Ranh giới 96px không phải con số cho đẹp.** Chữ "tsudev" chiếm khoảng một phần
tám chiều cao ảnh. Ở 96px, chữ còn khoảng 12px - đã sát ngưỡng đọc được. Ở 32px
(favicon) chữ còn 4px, tức là một vệt xám. Thu nhỏ huy hiệu đầy đủ xuống cỡ icon
`MUST NOT` làm, vì kết quả vừa không đọc được vừa làm hình cú bết lại.

**Dấu hiệu thu gọn hiện CHƯA có** cho bản nhận diện chính thức - xem mục 10.
Trong lúc chưa có, `MUST NOT` tự cắt hình cú ra khỏi huy hiệu: chữ nằm đè lên
vòng mạch, cắt đi để lại một khoảng trống trong vành. Đây là việc thiết kế, không
phải việc cắt ảnh.

## 3. Quy tắc đặt tên

```
tsudev-<loại>[-<biến-thể>][-<cỡ>].<đuôi>
```

- `<loại>`: `logo` (huy hiệu đầy đủ) | `mark` (thu gọn, không chữ) |
  `wordmark` (chỉ chữ) | `favicon`.
- `<biến-thể>`: `dark` cho bản dùng trên nền tối, `square` cho bản vuông, `mono`
  cho bản một màu. Không ghi gì nghĩa là bản mặc định.
- `<cỡ>`: số pixel cạnh ngắn, chỉ dùng cho bản sinh ra (`tsudev-mark-256.png`).
- Toàn bộ chữ thường, phân tách bằng gạch ngang ngắn `-`, không dấu, không
  khoảng trắng, không số phiên bản trong tên file.

`MUST NOT` đặt tên kiểu `logo-tsudev.png`, `logo_final.png`, `logo v2.png`. Tên
sai làm hỏng thứ tự sắp xếp và khiến bản gốc lẫn với bản sinh. Chính bản gốc từng
mang tên `logo-tsudev.png` và đã được đổi khi đưa vào repo ở `v2.6.0`.

## 4. Biến thể theo nền

Hệ sinh thái có ba chế độ nền (`DESIGN_SYSTEM.md` mục 1).

Huy hiệu đầy đủ **dùng chung một bản cho cả ba chế độ**. Nó tự mang nền riêng -
thân huy hiệu là xanh đen `#000A18` viền quầng sáng cyan - nên nổi trên cả nền
sáng lẫn nền tối mà không cần bản thứ hai. Chỉ **wordmark dựng bằng chữ** mới cần
đổi màu theo nền (mục 5).

`MUST NOT` đặt huy hiệu lên ảnh nền rối. Nền phẳng lấy từ token `bg-surface` hoặc
`bg-base` là đủ.

## 5. Màu của dấu hiệu

Đo trực tiếp từ bản gốc, không phải ước lượng:

| Vai trò trong ảnh | Mã màu |
| --- | --- |
| Thân huy hiệu, nền vòng mạch | `#000A18` |
| Quầng sáng và mắt cú | `#49EBFF` |
| Nét mạch, chữ `dev` | `#14AAFA` |
| Chữ `tsu` | `#FFFFFF` |

**Wordmark dựng bằng chữ thật, không phải ảnh** (`SHOULD`). Hai `span` màu khác
nhau cho ra chữ co giãn theo cỡ chữ hệ thống, nét sắc ở mọi DPI, chọn và tìm kiếm
được, và tự đổi màu theo chế độ nền mà không cần tải thêm file. Màu dùng cho chữ
**không phải** màu lấy thẳng từ ảnh, vì `#14AAFA` chỉ đạt **2.32:1** trên nền
sáng - trượt cả ngưỡng chữ lớn:

| Nền | Chữ `tsu` | Chữ `dev` | Tương phản `dev` |
| --- | --- | --- | --- |
| Light / Warm | token `text-primary` | `#0B6FA8` | 4.92 / 4.84 |
| Dark | `#FFFFFF` | `#14AAFA` | 6.72 |

Cả hai giá trị đều vượt **4.5:1**, tức là dùng được cả ở cỡ chữ thường chứ không
riêng cỡ lớn.

**Màu dấu hiệu KHÔNG phải token giao diện.** Chúng `MUST NOT` ghi đè token ngữ
nghĩa trong `tokens/design-tokens.json` (`primary`, `text-link`, `danger`, ...).
Repo cần dải màu thương hiệu thì khai riêng, tên khác hẳn tên token ngữ nghĩa, và
chỉ dùng cho phần nhận diện. Nút bấm, liên kết, trạng thái vẫn `MUST` đọc token
như mọi nơi khác.

## 6. Kích thước tối thiểu và vùng an toàn

| Dùng ở đâu | Tối thiểu |
| --- | --- |
| Huy hiệu đầy đủ, trên màn hình | cạnh ngắn **96px** (dưới ngưỡng này dùng dấu hiệu thu gọn) |
| Huy hiệu đầy đủ, khi in | **20mm** |
| Dấu hiệu thu gọn, trên màn hình | cạnh ngắn **16px** |
| Favicon | bản `.ico` nhiều lớp 16/32/48, `MUST NOT` co một ảnh PNG duy nhất |

**Vùng an toàn**: chừa quanh dấu hiệu một khoảng trống tối thiểu bằng **1/4 cạnh
ngắn** của chính nó. Không đặt chữ, viền, hay ảnh khác vào vùng này.

Bản gốc đã có sẵn lề trong suốt (vùng ảnh thật nằm trong khung
`111,64 - 918,966` của khung `1024x1024`). Lề đó là một phần của ảnh, **không**
thay thế vùng an toàn ở trên.

## 7. Sinh bộ icon từ bản gốc

Một bản gốc, một script, mọi bản sinh ra từ đó. `MUST NOT` cắt tay từng cỡ.

- Nguồn để sinh icon `MUST` là **dấu hiệu thu gọn**, không phải huy hiệu đầy đủ
  (lý do ở mục 2).
- Script sinh `MUST` nội suy bằng LANCZOS (hoặc tương đương) và giữ kênh alpha.
- `.ico` `MUST` là file nhiều lớp `16/24/32/48/64/128/256`, không phải một ảnh
  256 đổi đuôi.
- Bản cài đặt tham chiếu: `tools/gen_icons.py` ở `tsudev-cwico` - sinh bộ Tauri,
  tile Microsoft Store và bộ web từ đúng một file gốc.

## 8. Điều cấm

`MUST NOT`:

- Kéo méo, xoay, lật, cắt cụt, hay đổi tỉ lệ giữa các phần của dấu hiệu.
- Đổi màu dấu hiệu, thêm đổ bóng, viền, hay hiệu ứng chuyển màu.
- Thu nhỏ huy hiệu đầy đủ xuống dưới 96px, kể cả khi "nhìn vẫn tạm được".
- Tự cắt hình cú ra khỏi huy hiệu để làm icon (mục 2).
- Đặt dấu hiệu lên ảnh nền rối hoặc nền có tương phản dưới 3:1 với thân huy hiệu.
- Dùng dấu hiệu tsudev cho sản phẩm không thuộc hệ sinh thái tsudev.

## 9. Khả năng truy cập

- Dấu hiệu đi kèm wordmark bằng chữ: ảnh `MUST` khai `alt=""` và
  `aria-hidden="true"` - chữ bên cạnh đã mang nghĩa, đọc hai lần là nhiễu.
- Dấu hiệu đứng một mình mà có chức năng (bấm được, dẫn về trang chủ): `MUST` có
  nhãn văn bản hoặc `aria-label` mô tả **hành động**, không phải mô tả hình.
- Dấu hiệu thuần trang trí: `alt=""`, không thêm gì.

## 10. Nợ đã biết

| Việc | Ở đâu | Ghi chú |
| --- | --- | --- |
| Chưa có dấu hiệu thu gọn | bản nhận diện chính thức | Cần một bản **chỉ hình, không chữ**, xuất từ file thiết kế gốc. Không cắt ra từ huy hiệu được (mục 2). Cần chủ project làm hoặc đặt làm |
| Hai bộ nhận diện song song | `tsudev-cwico/assets/brand/` | Repo đó dùng một con cú **khác hẳn**: hình học, mắt cam hổ phách, chip "TSU", `222x280`, không có vòng mạch và không có chữ. Nó **không** phải bản độ phân giải thấp của huy hiệu chính thức mà là một thiết kế riêng |
| Dải màu giao diện lệch theo | `tsudev-cwico/ui/src/index.css` | Khai `--color-dev-*` là **cam** (`#d2540e`), lấy mẫu từ mắt cú của bản cũ. Bản chính thức không có màu cam nào. Đổi là đổi giao diện toàn app, cần ảnh chụp trước/sau |

Ba việc này liên quan nhau và `MUST` giải quyết cùng một lượt, sau khi có dấu
hiệu thu gọn. Hàng đợi: `TS-5` và `TS-6` trong `logs/STATE.md` của repo quy ước.

## 11. Checklist trước khi thêm hoặc đổi tài sản thương hiệu

1. File đặt trong `assets/brand/`, tên theo mục 3.
2. Bản sao của bản gốc khớp SHA-256 ghi ở mục 1.
3. Chọn đúng dấu hiệu theo cỡ hiển thị (mục 2), không thu nhỏ huy hiệu đầy đủ.
4. Bản sinh ra tạo bằng script, không cắt tay; script chạy lại được.
5. Wordmark bằng chữ dùng đúng cặp màu theo nền ở mục 5.
6. Không hard-code màu dấu hiệu vào chỗ lẽ ra phải dùng token.
7. Ảnh có `alt` đúng vai trò theo mục 9.
8. File ảnh lớn không cần lúc chạy `MUST` vào `.gitignore`
   ([`GITIGNORE_POLICY.md`](GITIGNORE_POLICY.md)).
