# CHUẨN VÙNG BẢN GHI VÀ BỘ CHỌN SỐ BẢN GHI (v3.0.0)

> Mọi trang, mọi modal, mọi bảng có danh sách bản ghi trong hệ sinh thái tsudev
> đều dùng **cùng một bộ chọn số bản ghi, cùng các mốc, cùng vị trí**. Người dùng
> đi từ sản phẩm này sang sản phẩm khác không phải học lại.
>
> Kiểu dáng bảng, màu, trạng thái tương tác lấy từ
> [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) mục 5. Tham số truy vấn và trần phía máy
> chủ ở [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md) mục 7. Tài liệu này quy
> định **hành vi và vị trí** của bộ chọn.

## Mục lục

1. [Phạm vi bắt buộc](#1-phạm-vi-bắt-buộc)
2. [Các mốc và giá trị mặc định](#2-các-mốc-và-giá-trị-mặc-định)
3. [Vị trí và bố cục](#3-vị-trí-và-bố-cục)
4. [Hành vi](#4-hành-vi)
5. [Ghi nhớ lựa chọn](#5-ghi-nhớ-lựa-chọn)
6. [Dòng tóm tắt và phân trang](#6-dòng-tóm-tắt-và-phân-trang)
7. [Khả năng truy cập](#7-khả-năng-truy-cập)
8. [Hợp đồng với máy chủ](#8-hợp-đồng-với-máy-chủ)
9. [Trạng thái rỗng, đang tải, lỗi](#9-trạng-thái-rỗng-đang-tải-lỗi)
10. [Ứng dụng desktop](#10-ứng-dụng-desktop)
11. [Bản cài đặt tham chiếu](#11-bản-cài-đặt-tham-chiếu)
12. [Checklist nghiệm thu](#12-checklist-nghiệm-thu)

---

## 1. Phạm vi bắt buộc

`MUST` có bộ chọn số bản ghi ở **mọi vùng hiển thị danh sách bản ghi có thể vượt
quá mốc mặc định**, bất kể vùng đó là:

- bảng ở trang đầy đủ,
- bảng hoặc danh sách bên trong **modal**,
- ngăn kéo (drawer), thẻ (tab), khung bên (side panel),
- bảng trong ứng dụng desktop.

`MUST NOT` bắt buộc với:

- danh sách có **trần cứng theo bản chất** dưới 10 mục (ví dụ 5 thông báo gần
  nhất trên bảng điều khiển),
- vùng cuộn vô hạn thuần túy như dòng thời gian mạng xã hội. Vùng đó `MUST` dùng
  phân trang con trỏ (mục 8.3) và `MUST NOT` giả vờ có bộ chọn không tác dụng.

Nghi ngờ thì thêm bộ chọn. Một bộ chọn thừa gây phiền nhẹ; một bảng 3000 dòng
không có bộ chọn làm treo trình duyệt.

---

## 2. Các mốc và giá trị mặc định

**Bộ mốc chuẩn, không đổi, không thêm bớt:**

```
10  |  20  |  50  |  100  |  200
```

- Mặc định khi mở trang lần đầu: **`10`**.
- `MUST NOT` thêm mốc (25, 500, 1000) hay bỏ mốc. Mốc lệch chuẩn ở một sản phẩm
  phá đúng thứ mà tài liệu này tồn tại để giữ.
- `MUST NOT` có lựa chọn **"Tất cả"**. Nó là một truy vấn không có trần, và trần
  chính là thứ giữ cho máy chủ không sập vì một cú bấm.
- Trang chắc chắn chỉ có ít bản ghi thì `MAY` cắt bớt các mốc **từ trên xuống**
  (ví dụ chỉ còn `10 | 20 | 50`), nhưng `MUST` giữ nguyên thứ tự và luôn giữ mốc
  `10`. `MUST NOT` cắt từ dưới lên.
- Mốc lớn hơn tổng số bản ghi hiện có `MUST` vẫn hiện và vẫn bấm được, chỉ là
  không đổi kết quả. Ẩn đi làm bộ chọn nhảy múa mỗi lần lọc.

**Vì sao mặc định là 10:** trang tải nhanh hơn, thiết bị di động không phải cuộn
dài, và truy vấn đầu tiên của mọi trang là truy vấn rẻ nhất. Người cần nhiều hơn
sẽ tự chọn, và lựa chọn đó được nhớ (mục 5).

---

## 3. Vị trí và bố cục

**Quy tắc cứng:** bộ chọn `MUST` nằm ở **góc dưới bên trái** của vùng chứa bản
ghi. Trong modal, đó là góc dưới bên trái của **vùng bản ghi**, không phải của
chân modal chứa nút hành động.

```
+-----------------------------------------------------------+
|  Danh sách người dùng                              [ x ]   |  header modal
+-----------------------------------------------------------+
|  [ Tìm kiếm...            ]   [ Bộ lọc v ]                 |  thanh công cụ
+-----------------------------------------------------------+
|  Tên              | Email            | Trạng thái | Ngày   |  header bảng
|  ---------------------------------------------------------|
|  ...              | ...              | ...        | ...    |
|  ...              | ...              | ...        | ...    |  vùng bản ghi
|  ---------------------------------------------------------|
|  [ 10 v ] / trang        1-10 / 128     [< 1 2 3 ... 13 >] |  chân vùng bản ghi
+-----------------------------------------------------------+
|                              [ Đóng ]   [ Lưu thay đổi ]   |  chân modal
+-----------------------------------------------------------+
   ^                          ^                            ^
   TRÁI: bộ chọn          GIỮA: tóm tắt          PHẢI: phân trang
```

- Chân vùng bản ghi `MUST` chia ba: **trái** bộ chọn, **giữa** dòng tóm tắt,
  **phải** điều khiển phân trang.
- Chân vùng bản ghi `MUST` cao `48px`, đệm ngang `16px`, nền `bg-subtle`, viền
  trên `1px` màu `border`, nằm **trong** khung bo góc của bảng.
- Bộ chọn `MUST` là **Dropdown** chuẩn theo `DESIGN_SYSTEM.md` mục 5: cao `32px`
  (biến thể compact), radius-md, nền `bg-surface`, viền `border`.
- Nhãn kèm theo `MUST` là chuỗi `/ trang`, chữ `13px` màu `text-muted`, đặt
  **bên phải** ô chọn, cách `8px`. Đủ ngắn để không chiếm chỗ, đủ rõ để không
  phải đoán con số kia nghĩa là gì.
- `MUST NOT` đặt bộ chọn ở đầu bảng, ở thanh công cụ, hay trong menu ba chấm.
  Vị trí duy nhất là góc dưới bên trái.

### 3.1. Màn hình hẹp

Dưới `640px`, chân vùng bản ghi `MUST` xếp thành hai hàng:

```
+---------------------------------+
|  [ 10 v ] / trang   1-10 / 128  |   hàng 1: bộ chọn TRÁI, tóm tắt PHẢI
|  [ <  Trước ]      [ Sau  > ]   |   hàng 2: phân trang, chia đôi
+---------------------------------+
```

- Bộ chọn `MUST` vẫn nằm bên trái. Nó là thứ **không** được đổi vị trí ở mọi
  điểm ngắt.
- Phân trang dạng số `MAY` thu về hai nút `Trước` / `Sau`.

---

## 4. Hành vi

- Đổi mốc `MUST` **giữ nguyên bản ghi đầu tiên đang nhìn thấy**, rồi tính lại số
  trang. Cách tính: `trang_mới = floor(chỉ_số_bản_ghi_đầu / mốc_mới) + 1`.
  `MUST NOT` nhảy về trang 1 - người dùng đang xem dòng 340 mà bị ném về đầu
  danh sách là mất dấu công việc.
- Đổi mốc `MUST` tải lại dữ liệu **từ máy chủ**. `MUST NOT` tải sẵn 200 bản ghi
  rồi cắt ở máy khách; như vậy mốc `10` không tiết kiệm được gì.
- Đổi mốc `MUST` cập nhật tham số URL `page_size` và `page` (mục 8.1) để chia sẻ
  liên kết và nút quay lại của trình duyệt hoạt động đúng.
- Trong lúc tải lại, bảng `MUST` giữ chiều cao cũ và hiện khung xương (skeleton).
  `MUST NOT` để bảng co lại rồi giãn ra - trang nhảy là lỗi bố cục, không phải
  hiệu ứng.
- Đang chọn nhiều dòng mà đổi mốc: lựa chọn `MUST` được giữ theo `id`, và dòng
  tóm tắt `MUST` ghi `Đã chọn n mục` kể cả khi các mục đó không còn trên trang
  hiện tại.
- Bộ lọc hoặc từ khóa thay đổi `MUST` đưa về **trang 1** nhưng `MUST` **giữ
  nguyên mốc** đang chọn.
- Bộ chọn `MUST` vào trạng thái `Disabled` trong lúc đang tải, `MUST NOT` biến
  mất.

---

## 5. Ghi nhớ lựa chọn

Thứ tự ưu tiên khi quyết định hiển thị bao nhiêu bản ghi:

1. `page_size` trong **URL** - cao nhất, vì liên kết chia sẻ phải tái lập đúng.
2. Lựa chọn đã ghi nhớ cho **đúng bảng đó**.
3. `10` - mặc định.

- Ghi nhớ `MUST` theo **từng bảng**, khóa dạng `tsudev.pagesize.<mã-bảng>`.
  `MUST NOT` dùng một giá trị chung cho cả ứng dụng: người ta muốn 200 dòng ở
  bảng nhật ký nhưng vẫn muốn 10 ở bảng người dùng.
- Web `MUST` lưu bằng `localStorage`. Đây là tiện nghi hiển thị, không phải dữ
  liệu; mất là chấp nhận được, và mã `MUST` chạy đúng khi đọc ra rỗng hoặc khi
  `localStorage` ném lỗi (cửa sổ ẩn danh, trình duyệt chặn).
- Giá trị đọc ra `MUST` được kiểm nằm trong bộ mốc chuẩn. Không hợp lệ thì
  `MUST` quay về `10`, `MUST NOT` tin thẳng.
- Desktop `MUST` lưu vào tệp cấu hình người dùng của ứng dụng, không phải vào
  registry hay thư mục cài đặt.
- `MUST NOT` lưu lựa chọn này lên máy chủ theo tài khoản. Đó là tiện nghi cục bộ,
  không đáng một vòng gọi mạng và một bảng dữ liệu.

---

## 6. Dòng tóm tắt và phân trang

**Dòng tóm tắt** (giữa chân vùng bản ghi), chữ `13px` màu `text-muted`:

| Tình huống | Nội dung |
| --- | --- |
| Bình thường | `1-10 / 128` |
| Trang cuối lẻ | `121-128 / 128` |
| Chỉ một trang | `8 mục` |
| Rỗng | không hiện dòng tóm tắt |
| Có lọc | `1-10 / 42 (lọc từ 128)` |
| Đang chọn | `Đã chọn 3 mục` thay cho dòng trên, kèm nút `Bỏ chọn` |

- Tổng số `MUST` là số thật từ máy chủ. Đếm được tốn kém thì `MUST` dùng phân
  trang con trỏ (mục 8.3) và bỏ hẳn tổng số, `MUST NOT` hiện con số ước lượng
  không ghi chú.
- Số `MUST` có dấu phân cách hàng nghìn theo tiếng Việt: `1.128`.

**Phân trang** (phải chân vùng bản ghi):

- Dạng số `MUST` hiện tối đa **7 ô**: `< 1 ... 5 6 7 ... 13 >`, trang hiện tại
  nền `primary` chữ `on-primary`.
- Ô `MUST` cao `32px`, rộng tối thiểu `32px`, radius-md. Nút `<` `>` ở trang đầu
  và cuối `MUST` ở trạng thái `Disabled`, `MUST NOT` bị ẩn - ẩn làm hàng nút xê
  dịch.
- `MAY` thêm ô nhập `Tới trang [ ]` khi tổng số trang vượt **20**.

---

## 7. Khả năng truy cập

Theo [`ACCESSIBILITY.md`](ACCESSIBILITY.md), phần riêng của vùng bản ghi:

- Bộ chọn `MUST` là `<select>` thật, hoặc một `combobox` cài đủ vai trò ARIA. Nếu
  dựng tay: `role="combobox"`, `aria-expanded`, `aria-controls`, điều hướng bằng
  `Mũi tên lên/xuống`, `Home`, `End`, `Esc` đóng, `Enter` chọn.
- Bộ chọn `MUST` có nhãn liên kết: `<label for>` hoặc `aria-label="Số bản ghi
  mỗi trang"`. Chuỗi `/ trang` một mình **không** đủ làm nhãn cho trình đọc màn
  hình.
- Sau khi đổi mốc và dữ liệu đã về, `MUST` thông báo qua vùng `aria-live="polite"`:
  `Đang hiển thị 1 đến 20 trong 128 mục`.
- Tiêu điểm `MUST` ở lại trên bộ chọn sau khi đổi. `MUST NOT` nhảy về đầu bảng.
- Điều khiển phân trang `MUST` nằm trong `<nav aria-label="Phân trang">`, trang
  hiện tại có `aria-current="page"`.
- Vùng bảng cuộn ngang `MUST` có `tabindex="0"` và nhãn, để cuộn được bằng bàn
  phím.
- Mọi ô chọn, nút phân trang `MUST` đạt vùng chạm tối thiểu `44x44px` trên thiết
  bị cảm ứng, kể cả khi phần nhìn thấy chỉ `32px`.

---

## 8. Hợp đồng với máy chủ

### 8.1. Tham số

Dùng đúng bộ tham số của [`SEARCH_AND_FILTER.md`](SEARCH_AND_FILTER.md) mục 7:

```
GET /api/v1/{resource}?page=1&page_size=10&sort=newest
```

- `page_size` `MUST` được máy chủ kiểm nằm trong tập `{10, 20, 50, 100, 200}`.
  Giá trị lạ `MUST` bị **quy về mốc hợp lệ gần nhất không lớn hơn nó**, `MUST NOT`
  trả lỗi - một tham số hiển thị sai không đáng làm hỏng cả trang.
- Trần cứng phía máy chủ là **200**. `MUST NOT` phục vụ `page_size` lớn hơn với
  bất kỳ lý do gì, kể cả cho quản trị viên hay cho tác vụ nội bộ. Xuất dữ liệu
  lớn đi đường riêng (mục 8.4).
- `page` bắt đầu từ `1`. Vượt quá trang cuối `MUST` trả mảng rỗng kèm `meta`
  đúng, `MUST NOT` trả `404`.

### 8.2. Phản hồi

```json
{
  "data": [],
  "meta": {
    "total": 128,
    "page": 1,
    "page_size": 10,
    "total_pages": 13
  }
}
```

### 8.3. Phân trang con trỏ

Tập dữ liệu vượt **100.000 bản ghi**, hoặc bảng có ghi thường xuyên, `MUST`
chuyển sang phân trang con trỏ. `OFFSET` lớn buộc cơ sở dữ liệu quét và bỏ đi
từng dòng một, và bản ghi mới chèn vào giữa lúc lật trang làm lặp hoặc mất dòng.

```
GET /api/v1/{resource}?page_size=10&cursor=<mã>
```

```json
{ "data": [], "meta": { "page_size": 10, "next_cursor": "...", "has_more": true } }
```

- Bộ chọn số bản ghi `MUST` giữ nguyên, cùng mốc, cùng vị trí. Chỉ phần phân
  trang bên phải đổi thành `Trước` / `Sau`.
- Dòng tóm tắt `MUST` chuyển thành `Đang hiển thị 10 mục`, bỏ tổng số.

### 8.4. Chống lạm dụng

- `page_size` từ `100` trở lên `MUST` có giới hạn tần suất riêng, chặt hơn: đề
  xuất **10 yêu cầu / phút / tài khoản**. Đây là cái giá của việc nâng trần từ
  100 lên 200 và nó là điều kiện của việc nâng đó.
- Truy vấn `page_size` lớn `MUST` có thời gian chờ (timeout) ở tầng máy chủ và
  `MUST` được ghi log khi vượt ngưỡng chậm theo
  [`TESTING_QUALITY.md`](TESTING_QUALITY.md).
- Cần nhiều hơn 200 bản ghi cùng lúc thì `MUST` dùng chức năng **xuất tệp** chạy
  nền, trả về qua liên kết tải có hạn, `MUST NOT` nới trần API.
- Mọi truy vấn `MUST` kiểm quyền trên từng bản ghi, không chỉ trên endpoint, theo
  [`SECURITY_BASELINE.md`](SECURITY_BASELINE.md) mục 6.3.

---

## 9. Trạng thái rỗng, đang tải, lỗi

| Trạng thái | Vùng bản ghi | Bộ chọn |
| --- | --- | --- |
| Đang tải lần đầu | Khung xương đúng số dòng theo mốc đang chọn | Hiện, `Disabled` |
| Đang tải lại | Giữ dữ liệu cũ mờ đi `0.5`, phủ vòng quay | Hiện, `Disabled` |
| Rỗng (chưa có gì) | Icon + `Chưa có bản ghi nào` + nút hành động chính | Hiện, `Disabled` |
| Rỗng (do lọc) | `Không có kết quả khớp bộ lọc` + nút `Xóa bộ lọc` | Hiện, bật |
| Lỗi | `Không tải được dữ liệu` + nút `Thử lại` + mã lỗi nhỏ | Hiện, `Disabled` |

- Khung xương `MUST` đúng số dòng của mốc đang chọn. Vẽ 3 dòng xương rồi hiện ra
  200 dòng là một cú giật bố cục.
- Vùng rỗng `MUST` cao tối thiểu `240px` để chân vùng bản ghi không dính lên
  header.

---

## 10. Ứng dụng desktop

Quy ước này áp cho cả desktop. Điểm khác biệt duy nhất được phép:

- Bảng desktop `MAY` dùng cuộn ảo (virtual scroll) thay phân trang khi dữ liệu ở
  ngay trên máy (ví dụ kết quả rà quét của `tsudev-swico`). Khi đó bộ chọn `MUST`
  vẫn tồn tại, vẫn ở góc dưới bên trái, và điều khiển **số dòng nạp mỗi lô**.
- Chiều cao chân vùng bản ghi `MAY` giảm còn `40px` theo mật độ compact của
  `DESIGN_SYSTEM.md` mục 3.
- Phím tắt `Ctrl+G` `MAY` mở ô nhảy tới trang. Đây là gợi ý, không bắt buộc.

---

## 11. Bản cài đặt tham chiếu

Repo `tsudev` giữ bản cài đặt tham chiếu dùng chung:

| Thứ | Đường dẫn (repo `tsudev-tsudev/tsudev`) |
| --- | --- |
| Component | `packages/ui/src/components/DataTable/` |
| Bộ chọn | `packages/ui/src/components/DataTable/PageSizeSelect.tsx` |
| Hằng số mốc | `packages/ui/src/components/DataTable/constants.ts` |

- Sản phẩm web trong hệ sinh thái `MUST` dùng component này thay vì tự dựng lại.
  Tự dựng là cách các mốc bắt đầu trôi khỏi nhau.
- Nền tảng không dùng được React `MUST` cài lại đúng hành vi ở mục 2 tới 9 và
  `MUST` ghi một dòng vào `README.md` của repo nói rõ chỗ nào lệch, vì sao.

---

## 12. Checklist nghiệm thu

- [ ] Đủ năm mốc `10 | 20 | 50 | 100 | 200`, không thêm, không bớt, không có "Tất cả".
- [ ] Mặc định lần đầu là `10`.
- [ ] Bộ chọn ở **góc dưới bên trái** vùng bản ghi, kể cả trong modal, kể cả ở
      màn hình hẹp.
- [ ] Chân vùng bản ghi chia ba: bộ chọn - tóm tắt - phân trang.
- [ ] Đổi mốc giữ nguyên bản ghi đầu đang nhìn thấy, không nhảy về trang 1.
- [ ] Đổi mốc tải lại từ máy chủ và cập nhật `page_size` trong URL.
- [ ] Lựa chọn được nhớ theo từng bảng, đọc ra được kiểm hợp lệ.
- [ ] Máy chủ chặn cứng ở `200` và có giới hạn tần suất riêng cho mốc lớn.
- [ ] Bộ chọn có nhãn cho trình đọc màn hình và thông báo `aria-live` sau khi đổi.
- [ ] Tiêu điểm ở lại trên bộ chọn sau khi đổi.
- [ ] Đủ năm trạng thái ở mục 9, khung xương đúng số dòng.
- [ ] `./scripts/check-standards.sh` đạt.
